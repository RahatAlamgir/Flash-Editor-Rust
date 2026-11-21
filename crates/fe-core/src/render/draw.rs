//! Draws shapes, pictures and groups with tiny-skia (C# `ObjectRenderer.Draw`). Every function draws in the
//! object's own box space (0, 0 = top-left of the upright box), mapped to the pixmap by `t`.

use tiny_skia::{
    Color, FillRule, FilterQuality, GradientStop, LineCap, LineJoin, LinearGradient, Mask, Paint, Path, PathBuilder,
    Pattern, Pixmap, Point, RadialGradient, Rect, Shader, SpreadMode, Stroke, StrokeDash, Transform,
};

use super::geometry::{self, Combine, Outline};
use super::{pixmaps, warp};
use crate::imaging::{ColorRgba, PixelRect, PointD};
use crate::objects::{Gradient, ObjectBox, ShapeData, ShapeKind, StoredObject, StrokeStyle};

/// Maps box space (0, 0 = top-left of the upright box) to the canvas: flip, turn, place.
pub fn local_to_canvas(b: &ObjectBox, flips: (bool, bool)) -> Transform {
    let c = b.center();
    Transform::from_translate(c.x as f32, c.y as f32)
        .pre_rotate(b.rotation as f32)
        .pre_scale(if flips.0 { -1.0 } else { 1.0 }, if flips.1 { -1.0 } else { 1.0 })
        .pre_translate(-(b.width / 2.0) as f32, -(b.height / 2.0) as f32)
}

/// How long an arrow head is: it grows with the line width.
fn arrow_length(shape: &ShapeData) -> f64 {
    (shape.stroke_width * 4.0).max(10.0)
}

/// Extra room around the box for borders, lines and arrow heads (C# `MarginFor`).
pub fn margin_for(obj: &StoredObject) -> f64 {
    if let Some(s) = obj.shape() {
        let b = obj.box_().unwrap_or_default();
        let arrows = s.arrow_start || s.arrow_end;
        if s.is_line() {
            let tip = if arrows { arrow_length(&s) / 2.0 } else { s.stroke_width / 2.0 };
            return geometry::line_reach(b.width, s.bend, s.bend_style) + tip + 2.0;
        }
        if s.kind == ShapeKind::Path && !s.closed && arrows {
            return arrow_length(&s) + 2.0 + s.stroke_width / 2.0;
        }
        if s.stroke.is_some() {
            return s.stroke_width / 2.0 + 2.0;
        }
        return 2.0;
    }
    if let Some(t) = obj.text() {
        return if t.outline_width > 0.0 && t.outline_color.is_some() { t.outline_width + 2.0 } else { 2.0 };
    }
    if let (Some(members), Some(gb), Some(ob)) = (obj.group_members(), obj.box_(), obj.original_box()) {
        let scale = (gb.width / ob.width.max(1e-9)).max(gb.height / ob.height.max(1e-9));
        let reach = |m: &crate::objects::GroupMember| {
            let b = m.object.box_().unwrap_or_default();
            margin_for(&m.object) + m.object.looks().reach(b.width, b.height)
        };
        return 2.0 + members.iter().map(reach).fold(0.0, f64::max) * scale;
    }
    2.0
}

/// A position in the object (0..1 across its box) on the canvas, however it is turned or mirrored.
pub fn from_unit(obj: &StoredObject, u: PointD) -> PointD {
    let b = obj.box_().unwrap_or_default();
    let (fx, fy) = obj.flips();
    let x = (u.x - 0.5) * b.width * if fx { -1.0 } else { 1.0 };
    let y = (u.y - 0.5) * b.height * if fy { -1.0 } else { 1.0 };
    b.to_canvas(x, y)
}

/// The canvas area the object covers: its box and, once its corners were dragged, those corners too.
pub fn visual_bounds(obj: &StoredObject, margin: f64) -> PixelRect {
    let Some(b) = obj.box_() else { return PixelRect::EMPTY };
    let mut points: Vec<PointD> = b.corners().to_vec();
    let outline = obj.shape().and_then(|s| s.vertices).or_else(|| obj.warp());
    if let Some(list) = outline {
        points.extend(list.iter().map(|p| from_unit(obj, *p)));
    }
    let (l, t) = points.iter().fold((f64::MAX, f64::MAX), |(l, t), p| (l.min(p.x), t.min(p.y)));
    let (r, bt) = points.iter().fold((f64::MIN, f64::MIN), |(r, b), p| (r.max(p.x), b.max(p.y)));
    PixelRect::from_bounds(l - margin, t - margin, r + margin, bt + margin)
}

pub(super) fn color(c: ColorRgba) -> Color {
    Color::from_rgba8(c.r, c.g, c.b, c.a)
}

pub(super) fn solid(c: ColorRgba) -> Paint<'static> {
    let mut paint = Paint { anti_alias: true, ..Paint::default() };
    paint.set_color(color(c));
    paint
}

/// A gradient over a `w` x `h` area (C# `GradientBrushFor`): a straight one reaches the far corners whatever
/// its angle; a round one grows from the middle to the edges.
pub(super) fn gradient_shader(g: &Gradient, w: f64, h: f64) -> Option<Shader<'static>> {
    let (w, h) = (w.max(1.0), h.max(1.0));
    let stops: Vec<GradientStop> =
        g.sorted().iter().map(|(c, p)| GradientStop::new(p.clamp(0.0, 1.0) as f32, color(*c))).collect();
    let center = Point::from_xy((w / 2.0) as f32, (h / 2.0) as f32);
    if g.radial {
        let t = Transform::from_translate(center.x, center.y)
            .pre_scale(1.0, (h / w) as f32)
            .pre_translate(-center.x, -center.y);
        return RadialGradient::new(center, center, (w / 2.0) as f32, stops, SpreadMode::Pad, t);
    }
    let (dy, dx) = g.angle.to_radians().sin_cos();
    let half = ((w * dx).abs() + (h * dy).abs()) / 2.0;
    let start = Point::from_xy(center.x - (dx * half) as f32, center.y - (dy * half) as f32);
    let end = Point::from_xy(center.x + (dx * half) as f32, center.y + (dy * half) as f32);
    LinearGradient::new(start, end, stops, SpreadMode::Pad, Transform::identity())
}

/// A mask that covers the path (filled).
fn mask_of(pm: &Pixmap, paths: &[&Path], rule: FillRule, t: Transform) -> Option<Mask> {
    let mut mask = Mask::new(pm.width(), pm.height())?;
    for p in paths {
        mask.fill_path(p, rule, true, t);
    }
    Some(mask)
}

/// All parts as one path (clockwise parts joined with the non-zero rule make their union).
fn joined(outline: &Outline) -> Option<Path> {
    let mut b = PathBuilder::new();
    for p in &outline.parts {
        b.push_path(p);
    }
    b.finish()
}

fn fill_outline(pm: &mut Pixmap, outline: &Outline, paint: &Paint, t: Transform, clip: Option<&Mask>) {
    match outline.combine {
        Combine::Single => pm.fill_path(&outline.parts[0], paint, FillRule::Winding, t, clip),
        Combine::EvenOdd => pm.fill_path(&outline.parts[0], paint, FillRule::EvenOdd, t, clip),
        Combine::Union => {
            if let Some(path) = joined(outline) {
                pm.fill_path(&path, paint, FillRule::Winding, t, clip);
            }
        }
        Combine::Exclude => {
            if let Some(mut keep) = mask_of(pm, &[&outline.parts[1]], FillRule::Winding, t) {
                keep.invert();
                pm.fill_path(&outline.parts[0], paint, FillRule::Winding, t, Some(&keep));
            }
        }
    }
}

/// The border along the outside of the combined outline only (C# strokes the combined geometry).
fn stroke_outline(pm: &mut Pixmap, outline: &Outline, paint: &Paint, stroke: &Stroke, t: Transform) {
    match outline.combine {
        Combine::Single | Combine::EvenOdd => pm.stroke_path(&outline.parts[0], paint, stroke, t, None),
        Combine::Union => {
            for (i, part) in outline.parts.iter().enumerate() {
                let others: Vec<&Path> =
                    outline.parts.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, p)| p).collect();
                if let Some(mut outside) = mask_of(pm, &others, FillRule::Winding, t) {
                    outside.invert();
                    pm.stroke_path(part, paint, stroke, t, Some(&outside));
                }
            }
        }
        Combine::Exclude => {
            if let Some(mut outside_cut) = mask_of(pm, &[&outline.parts[1]], FillRule::Winding, t) {
                outside_cut.invert();
                pm.stroke_path(&outline.parts[0], paint, stroke, t, Some(&outside_cut));
            }
            if let Some(inside_moon) = mask_of(pm, &[&outline.parts[0]], FillRule::Winding, t) {
                pm.stroke_path(&outline.parts[1], paint, stroke, t, Some(&inside_moon));
            }
        }
    }
}

/// The pen of a shape's border or line (C#: dashes counted in line widths; dots are round).
fn pen(shape: &ShapeData) -> Stroke {
    let width = shape.stroke_width.max(0.1);
    // C# rounds the two ends of a line, but each dash of a dashed border has flat ends (only dots are round).
    let round = match shape.dash {
        StrokeStyle::Dotted => true,
        StrokeStyle::Dashed => false,
        StrokeStyle::Solid => shape.is_line() || shape.kind == ShapeKind::Path,
    };
    let dash = match shape.dash {
        StrokeStyle::Solid => None,
        StrokeStyle::Dashed => {
            let on = if shape.dash_length > 0.0 { shape.dash_length } else { 3.0 * width };
            let off = if shape.dash_gap > 0.0 { shape.dash_gap } else { 2.0 * width };
            StrokeDash::new(vec![on as f32, off as f32], 0.0)
        }
        StrokeStyle::Dotted => {
            let gap = (if shape.dash_gap > 0.0 { shape.dash_gap / width } else { 1.2 }) + 1.0;
            StrokeDash::new(vec![(0.01 * width) as f32, (gap * width) as f32], 0.0)
        }
    };
    Stroke {
        width: width as f32,
        line_cap: if round { LineCap::Round } else { LineCap::Butt },
        line_join: LineJoin::Round,
        dash,
        ..Stroke::default()
    }
}

fn unit(x: f64, y: f64) -> PointD {
    let d = x.hypot(y);
    if d < 1e-9 { PointD::new(1.0, 0.0) } else { PointD::new(x / d, y / d) }
}

/// Filled triangles at the ends of a line or an open curve, turned to follow the line (C# `DrawArrowHeads`).
fn arrow_heads(pm: &mut Pixmap, shape: &ShapeData, b: &ObjectBox, paint: &Paint, t: Transform) {
    let (w, h) = (b.width.max(1.0), b.height.max(1.0));
    let (a, e, out_a, out_e);
    match shape.nodes.as_deref().filter(|n| shape.kind == ShapeKind::Path && n.len() >= 2) {
        Some(nodes) => {
            let p = |q: PointD| PointD::new(q.x * w, q.y * h);
            let (first, second, last, before) = (nodes[0], nodes[1], nodes[nodes.len() - 1], nodes[nodes.len() - 2]);
            let s = if first.has_out() {
                first.handle_out
            } else if second.has_in() {
                second.handle_in
            } else {
                second.point
            };
            let en = if last.has_in() {
                last.handle_in
            } else if before.has_out() {
                before.handle_out
            } else {
                before.point
            };
            (a, e) = (p(first.point), p(last.point));
            let (s, en) = (p(s), p(en));
            out_a = unit(a.x - s.x, a.y - s.y);
            out_e = unit(e.x - en.x, e.y - en.y);
        }
        None => {
            let [s0, c1, c2, e0] = geometry::line_controls(w, h, shape.bend, shape.bend_style);
            (a, e) = (s0, e0);
            let from_a = if c1 != s0 {
                c1
            } else if c2 != e0 {
                c2
            } else {
                e0
            };
            let from_e = if c2 != e0 {
                c2
            } else if c1 != s0 {
                c1
            } else {
                s0
            };
            out_a = unit(s0.x - from_a.x, s0.y - from_a.y);
            out_e = unit(e0.x - from_e.x, e0.y - from_e.y);
        }
    }
    let (len, reach) = (arrow_length(shape), shape.stroke_width / 2.0);
    let half = len * 0.5;
    let head = |tip: PointD, o: PointD| {
        let tip = PointD::new(tip.x + o.x * reach, tip.y + o.y * reach);
        let back = PointD::new(tip.x - o.x * len, tip.y - o.y * len);
        let mut pb = PathBuilder::new();
        pb.move_to(tip.x as f32, tip.y as f32);
        pb.line_to((back.x - o.y * half) as f32, (back.y + o.x * half) as f32);
        pb.line_to((back.x + o.y * half) as f32, (back.y - o.x * half) as f32);
        pb.close();
        pb.finish()
    };
    if shape.arrow_end
        && let Some(p) = head(e, out_e)
    {
        pm.fill_path(&p, paint, FillRule::Winding, t, None);
    }
    if shape.arrow_start
        && let Some(p) = head(a, out_a)
    {
        pm.fill_path(&p, paint, FillRule::Winding, t, None);
    }
}

/// Draws a picture (PNG bytes) stretched over (x, y, w, h) of box space.
#[allow(clippy::too_many_arguments)]
fn picture(
    pm: &mut Pixmap,
    bytes: &std::sync::Arc<Vec<u8>>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    t: Transform,
    clip: Option<&Mask>,
) {
    let Some(mips) = pixmaps::mips(bytes) else { return };
    let full = &mips.levels[0];
    // How big one picture pixel ends up on the pixmap.
    let scale = (w / full.width() as f64) * ((t.sx * t.sy - t.kx * t.ky).abs() as f64).sqrt();
    let (level, factor) = mips.for_scale(scale);
    let (sx, sy) = (w * factor / full.width() as f64, h * factor / full.height() as f64);
    let pattern = Pattern::new(
        level.as_ref(),
        SpreadMode::Pad,
        FilterQuality::Bicubic,
        1.0,
        Transform::from_translate(x as f32, y as f32).pre_scale(sx as f32, sy as f32),
    );
    let paint = Paint { shader: pattern, anti_alias: true, ..Paint::default() };
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pm.fill_rect(rect, &paint, t, clip);
    }
}

fn draw_shape(pm: &mut Pixmap, obj: &StoredObject, shape: &ShapeData, b: &ObjectBox, t: Transform) {
    let Some(outline) = geometry::outline(shape, b) else { return };
    let gradient = obj.gradient();
    let fill = shape.can_fill().then_some(shape.fill).flatten().map(|f| match &gradient {
        Some(g) => {
            let mut p = Paint { anti_alias: true, ..Paint::default() };
            p.shader = gradient_shader(g, b.width, b.height).unwrap_or(Shader::SolidColor(color(f)));
            p
        }
        None => solid(f),
    });
    let stroke_color = shape.stroke.or(if shape.is_line() { shape.fill } else { None });
    let border = stroke_color.filter(|_| shape.stroke_width > 0.0).map(|c| match (&gradient, shape.gradient_border) {
        (Some(g), true) => {
            let mut p = Paint { anti_alias: true, ..Paint::default() };
            p.shader = gradient_shader(g, b.width, b.height).unwrap_or(Shader::SolidColor(color(c)));
            p
        }
        _ => solid(c),
    });
    if let Some(paint) = &fill {
        fill_outline(pm, &outline, paint, t, None);
    }
    if let (Some(pic), Some(bytes)) = (shape.picture.as_ref().filter(|_| !shape.is_line()), obj.picture())
        && let Some(mips) = pixmaps::mips(bytes)
    {
        // A picture in the frame: clipped to the outline, under the border.
        let (iw, ih) = (mips.levels[0].width() as f64, mips.levels[0].height() as f64);
        let (px, py, pw, ph) = pic.place(iw, ih, b.width, b.height);
        let parts: Vec<&Path> = outline.parts.iter().take(1).collect();
        let rule = if outline.combine == Combine::EvenOdd { FillRule::EvenOdd } else { FillRule::Winding };
        let mut clip = mask_of(pm, &parts, rule, t);
        if outline.combine == Combine::Union {
            let all: Vec<&Path> = outline.parts.iter().collect();
            clip = mask_of(pm, &all, FillRule::Winding, t);
        }
        if let Some(clip) = clip {
            picture(pm, bytes, px, py, pw, ph, t, Some(&clip));
        }
    }
    if let Some(paint) = &border {
        stroke_outline(pm, &outline, paint, &pen(shape), t);
    }
    let open = shape.is_line() || (shape.kind == ShapeKind::Path && !shape.closed);
    if open
        && (shape.arrow_start || shape.arrow_end)
        && let Some(c) = stroke_color
    {
        arrow_heads(pm, shape, b, &solid(c), t);
    }
}

fn draw_image(pm: &mut Pixmap, obj: &StoredObject, b: &ObjectBox, t: Transform) {
    let Some(bytes) = obj.picture() else { return };
    let Some(g) = obj.gradient() else {
        picture(pm, bytes, 0.0, 0.0, b.width, b.height, t, None);
        return;
    };
    // A see-through gradient over the picture, only where the picture itself is visible.
    let Some(mut layer) = Pixmap::new(pm.width(), pm.height()) else { return };
    picture(&mut layer, bytes, 0.0, 0.0, b.width, b.height, t, None);
    let only_picture = Mask::from_pixmap(layer.as_ref(), tiny_skia::MaskType::Alpha);
    if let (Some(shader), Some(rect)) =
        (gradient_shader(&g, b.width, b.height), Rect::from_xywh(0.0, 0.0, b.width as f32, b.height as f32))
    {
        let paint = Paint { shader, anti_alias: true, ..Paint::default() };
        layer.fill_rect(rect, &paint, t, Some(&only_picture));
    }
    pm.draw_pixmap(0, 0, layer.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
}

fn draw_group(pm: &mut Pixmap, obj: &StoredObject, t: Transform) -> Option<()> {
    let (members, gb, ob) = (obj.group_members()?, obj.box_()?, obj.original_box()?);
    let (sx, sy) = (gb.width / ob.width.max(1e-9), gb.height / ob.height.max(1e-9));
    // Members keep the places they had when grouped: map that area onto the group's box.
    let to_box = Transform::from_scale(sx as f32, sy as f32).pre_translate(-ob.x as f32, -ob.y as f32);
    for m in members {
        let mb = m.object.box_()?;
        let mt = t.pre_concat(to_box).pre_concat(local_to_canvas(&mb, m.object.flips()));
        if m.opacity >= 0.999 {
            draw(pm, &m.object, mt)?;
        } else {
            let mut layer = Pixmap::new(pm.width(), pm.height())?;
            draw(&mut layer, &m.object, mt)?;
            let paint = tiny_skia::PixmapPaint { opacity: m.opacity.clamp(0.0, 1.0) as f32, ..Default::default() };
            pm.draw_pixmap(0, 0, layer.as_ref(), &paint, Transform::identity(), None);
        }
    }
    Some(())
}

/// Draws an object in box space with its looks (shadow, glow, 3D, outline, blur). `None` when it contains
/// something the Rust version cannot draw yet.
pub fn draw(pm: &mut Pixmap, obj: &StoredObject, t: Transform) -> Option<()> {
    let looks = obj.looks();
    let tone = obj.tone().filter(|t| !t.is_neutral());
    if looks.is_empty() && tone.is_none() {
        return draw_plain(pm, obj, t);
    }
    let b = obj.box_()?;
    let mut layer = Pixmap::new(pm.width(), pm.height())?;
    draw_plain(&mut layer, obj, t)?;
    // Groups stretch their parts: shadows and the like grow with them.
    let scale = f64::from((t.sx * t.sy - t.kx * t.ky).abs()).sqrt();
    let mut center = tiny_skia::Point::from_xy((b.width / 2.0) as f32, (b.height / 2.0) as f32);
    t.map_point(&mut center);
    let mut straight = super::fx::from_pixmap(&layer);
    // The color adjustment of the object itself comes first, so the looks are made from the adjusted object.
    if let Some(tone) = &tone {
        let tables = tone.tone_tables();
        tone.apply_row(&mut straight.data, 1.0, tables.as_ref());
    }
    let img = super::fx::apply(
        straight,
        &looks.scaled(scale),
        (b.width * scale, b.height * scale),
        (f64::from(center.x), f64::from(center.y)),
    );
    let drawn = super::fx::to_pixmap(&img)?;
    pm.draw_pixmap(0, 0, drawn.as_ref(), &tiny_skia::PixmapPaint::default(), Transform::identity(), None);
    Some(())
}

fn draw_plain(pm: &mut Pixmap, obj: &StoredObject, t: Transform) -> Option<()> {
    let b = obj.box_()?;
    if let Some(corners) = obj.warp() {
        return warp::draw_warped(pm, obj, &b, t, &corners);
    }
    match obj.kind() {
        "shape" => draw_shape(pm, obj, &obj.shape()?, &b, t),
        "image" => draw_image(pm, obj, &b, t),
        "text" => super::text_draw::draw_text(pm, obj, &b, t),
        "group" => draw_group(pm, obj, t)?,
        "chart" => super::chart::draw_chart(pm, &obj.chart()?, &b, t),
        "qr" => super::qr_draw::draw_qr(pm, &obj.qr()?, &b, t),
        "barcode" => super::barcode_draw::draw_barcode(pm, &obj.barcode()?, &b, t),
        _ => return None,
    }
    Some(())
}
