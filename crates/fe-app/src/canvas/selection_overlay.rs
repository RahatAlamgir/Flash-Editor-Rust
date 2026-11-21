//! What the selection tools draw over the picture (C# `CanvasView` selection layers): the selected area with a
//! light blue tint and "marching ants" (animated only while on screen), outlines being drawn, Pen handles, the
//! Quick Select brush, the Move tool's rubber band and the frame around the selected layers.

use std::sync::Arc;

use egui::{Color32, ColorImage, Painter, Pos2, Rect, Shape, Stroke, TextureOptions, pos2, vec2};
use fe_core::document::{Document, LayerId};
use fe_core::imaging::{PixelRect, PointD};
use fe_core::objects::{ObjectBox, handles};
use fe_core::paint::signature::layer_signature;
use fe_core::selection::Selection;
use fe_gpu::ViewTransform;
use fe_ui_kit::theme::colors;

use crate::tools::ToolKind;
use crate::tools::selecting::Preview;

/// The dark line under the ants (`#E6000000`, 3 px).
const DARK: Color32 = Color32::from_rgba_unmultiplied_const(0, 0, 0, 0xE6);
/// Ant dashes: 3 x the 1.6 px line, on and off (C# `StrokeDashArray = 3 3`).
const DASH: f32 = 4.8;
/// How fast the ants walk, in screen points per second (C#: 0.5 x 1.6 px every 120 ms).
const ANTS_SPEED: f64 = 6.67;
/// The largest side of the tint texture (bigger selections get a smaller copy).
const MAX_TINT: usize = 2048;

/// Cached drawings, kept between frames.
#[derive(Default)]
pub struct OverlayCache {
    /// The tint texture of the selection it was made for.
    tint: Option<(Arc<Selection>, egui::TextureHandle)>,
    /// Visible bounds of layers by their pixel fingerprint (finding them reads every pixel).
    bounds: Vec<(LayerId, u64, PixelRect)>,
}

/// Everything the overlay needs from the app for one frame.
pub struct OverlayInput<'a> {
    pub doc: &'a Document,
    pub tool: ToolKind,
    pub preview: Option<Preview>,
    pub pen: &'a [fe_core::selection::bezier::Anchor],
    /// Where the mouse is (screen), for the brush circle.
    pub hover: Option<Pos2>,
    /// The brush size of the active tool, if it has a brush (canvas pixels).
    pub brush: Option<f64>,
    /// The selection drawn moved by this much (while its pixels are dragged).
    pub offset: (i32, i32),
    pub band: Option<PixelRect>,
    /// The Move tool is dragging something (the layer frame is hidden meanwhile).
    pub moving: bool,
    /// Reshape corner points of the selected object (empty when Reshape is off).
    pub reshape: Vec<PointD>,
    pub curve: Option<crate::tools::curve::CurveOverlay>,
    pub crop: Option<crate::tools::crop::CropOverlay>,
    pub retouch: Option<crate::tools::retouch::RetouchOverlay>,
    /// The part of a group being edited (its frame and handles are shown instead of the group's).
    pub part: Option<usize>,
    /// Text is being typed: the box shows, the handles do not.
    pub editing: bool,
}

/// Draws the overlay. Asks for the next frame while ants are on screen.
pub fn paint(ctx: &egui::Context, painter: &Painter, cache: &mut OverlayCache, t: ViewTransform, input: &OverlayInput) {
    let viewport = painter.clip_rect();
    let phase = (ctx.input(|i| i.time) * ANTS_SPEED).rem_euclid((DASH * 2.0) as f64) as f32;
    let mut animate = false;

    if input.tool == ToolKind::Move && !input.moving {
        layer_frames(painter, cache, t, input.doc, &input.reshape, input.part, input.editing);
    }
    if let Some(selection) = input.doc.selected_area_arc() {
        let off = vec2(input.offset.0 as f32, input.offset.1 as f32) * t.zoom;
        let t = ViewTransform { origin: t.origin + off, ..t };
        let screen = t.rect_to_screen(selection.bounds());
        if screen.intersects(viewport) {
            tint(ctx, painter, cache, t, selection);
            let polygons: Vec<&[PointD]> = selection.outline().iter().map(Vec::as_slice).collect();
            ants(painter, t, &polygons, true, phase, Color32::WHITE);
            animate = true;
        }
    } else {
        cache.tint = None;
    }
    if let Some(preview) = &input.preview {
        paint_preview(painter, t, preview, phase);
        animate = true;
    }
    if !input.pen.is_empty() {
        pen_handles(painter, t, input.pen);
    }
    if let Some(r) = &input.retouch {
        let at = |p: PointD| t.to_screen(p.x as f32, p.y as f32);
        let stroke = Stroke::new(1.5, colors::SELECTION_ACCENT);
        if let Some(c) = r.cross {
            let c = at(c);
            painter.line_segment([c - vec2(8.0, 0.0), c + vec2(8.0, 0.0)], Stroke::new(2.5, Color32::BLACK));
            painter.line_segment([c - vec2(0.0, 8.0), c + vec2(0.0, 8.0)], Stroke::new(2.5, Color32::BLACK));
            painter.line_segment([c - vec2(8.0, 0.0), c + vec2(8.0, 0.0)], Stroke::new(1.0, Color32::WHITE));
            painter.line_segment([c - vec2(0.0, 8.0), c + vec2(0.0, 8.0)], Stroke::new(1.0, Color32::WHITE));
        }
        if r.outline.len() >= 2 {
            painter.add(Shape::closed_line(r.outline.iter().map(|p| at(*p)).collect(), stroke));
        }
    }
    if let Some(crop) = &input.crop {
        crop_overlay(painter, t, input.doc, crop);
    }
    if let Some(curve) = &input.curve {
        curve_handles(painter, t, curve);
    }
    if let Some(rect) = input.band {
        let r = t.rect_to_screen(rect);
        painter.rect_filled(r, 0, colors::SELECTION_TINT);
        let corners = rect_points(rect);
        ants(painter, t, &[&corners], true, phase, colors::SELECTION_ACCENT);
    }
    if let (Some(p), Some(size)) = (input.hover, input.brush) {
        let radius = ((size / 2.0) as f32 * t.zoom).max(1.5);
        painter.circle_stroke(p, radius, Stroke::new(3.0, Color32::from_rgba_unmultiplied(0, 0, 0, 0xB0)));
        painter.circle_stroke(p, radius, Stroke::new(1.0, Color32::WHITE));
    }
    let minimized = ctx.input(|i| i.viewport().minimized.unwrap_or(false));
    if animate && !minimized {
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }
}

fn rect_points(r: PixelRect) -> Vec<PointD> {
    let (l, t, rr, b) = (r.x as f64, r.y as f64, r.right() as f64, r.bottom() as f64);
    vec![PointD::new(l, t), PointD::new(rr, t), PointD::new(rr, b), PointD::new(l, b)]
}

/// The light blue fill over the selected area: a rectangle, or a texture made from the mask.
fn tint(
    ctx: &egui::Context,
    painter: &Painter,
    cache: &mut OverlayCache,
    t: ViewTransform,
    selection: &Arc<Selection>,
) {
    let bounds = selection.bounds();
    let screen = t.rect_to_screen(bounds);
    if selection.is_rectangle() {
        painter.rect_filled(screen, 0, colors::SELECTION_TINT);
        return;
    }
    let fresh = cache.tint.as_ref().is_some_and(|(s, _)| Arc::ptr_eq(s, selection));
    if !fresh {
        let (w, h) = (bounds.width as usize, bounds.height as usize);
        let scale = w.max(h).div_ceil(MAX_TINT).max(1);
        let (tw, th) = (w.div_ceil(scale), h.div_ceil(scale));
        let mask = selection.mask();
        let tint = colors::SELECTION_TINT;
        let mut pixels = Vec::with_capacity(tw * th);
        for ty in 0..th {
            for tx in 0..tw {
                let (x, y) = ((tx * scale + scale / 2).min(w - 1), (ty * scale + scale / 2).min(h - 1));
                let a = (tint.a() as u32 * mask[y * w + x] as u32 / 255) as u8;
                pixels.push(Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), a));
            }
        }
        let image = ColorImage::new([tw, th], pixels);
        let texture = ctx.load_texture("selection-tint", image, TextureOptions::NEAREST);
        cache.tint = Some((selection.clone(), texture));
    }
    if let Some((_, texture)) = &cache.tint {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        painter.image(texture.id(), screen, uv, Color32::WHITE);
    }
}

/// Screen points of a polygon, skipping points closer than half a point to the previous one.
fn screen_points(t: ViewTransform, points: &[PointD]) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::with_capacity(points.len());
    for p in points {
        let s = t.to_screen(p.x as f32, p.y as f32);
        if out.last().is_none_or(|last| last.distance_sq(s) >= 0.25) {
            out.push(s);
        }
    }
    out
}

/// Marching ants: a dark line with light dashes on top that walk along the outline.
fn ants(painter: &Painter, t: ViewTransform, polygons: &[&[PointD]], closed: bool, phase: f32, light: Color32) {
    let viewport = painter.clip_rect().expand(4.0);
    let mut dashes: Vec<Shape> = Vec::new();
    for polygon in polygons {
        let mut points = screen_points(t, polygon);
        if points.len() < 2 {
            continue;
        }
        let bbox = Rect::from_points(&points);
        if !bbox.intersects(viewport) {
            continue;
        }
        if closed {
            points.push(points[0]);
        }
        painter.add(Shape::line(points.clone(), Stroke::new(3.0, DARK)));
        // Light dashes, `DASH` on and `DASH` off, shifted by `phase`. Distances are counted in f64 from the start
        // of the outline, and every step moves at least a little, so long outlines never get stuck on rounding.
        let dash = DASH as f64;
        let mut along = phase as f64;
        for pair in points.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let length = a.distance(b) as f64;
            if length <= 0.0 || !Rect::from_two_pos(a, b).intersects(viewport) {
                along += length;
                continue;
            }
            let d = (b - a) / length as f32;
            let mut s = 0.0f64;
            while s < length {
                let pos = along + s;
                let index = (pos / dash).floor();
                let e = (s + ((index + 1.0) * dash - pos).max(0.01)).min(length);
                if (index as i64).rem_euclid(2) == 0 {
                    dashes.push(Shape::line_segment([a + d * s as f32, a + d * e as f32], Stroke::new(1.6, light)));
                }
                s = e;
            }
            along += length;
        }
    }
    painter.extend(dashes);
}

fn paint_preview(painter: &Painter, t: ViewTransform, preview: &Preview, phase: f32) {
    match preview {
        Preview::Rect(r) => {
            painter.rect_filled(t.rect_to_screen(*r), 0, colors::SELECTION_TINT);
            ants(painter, t, &[&rect_points(*r)], true, phase, colors::SELECTION_ACCENT);
        }
        Preview::Ellipse(r) => {
            let screen = t.rect_to_screen(*r);
            painter.add(Shape::ellipse_filled(screen.center(), screen.size() / 2.0, colors::SELECTION_TINT));
            let points = fe_core::selection::ellipse_points(*r);
            ants(painter, t, &[&points], true, phase, colors::SELECTION_ACCENT);
        }
        Preview::Path { points, closed } => ants(painter, t, &[points], *closed, phase, colors::SELECTION_ACCENT),
    }
}

/// The Pen's points (small squares; the first one bigger, it closes the path) and curve handles.
fn pen_handles(painter: &Painter, t: ViewTransform, anchors: &[fe_core::selection::bezier::Anchor]) {
    let stroke = Stroke::new(1.2, colors::SELECTION_ACCENT);
    let at = |p: PointD| t.to_screen(p.x as f32, p.y as f32);
    for (i, a) in anchors.iter().enumerate() {
        if let Some(out) = a.out {
            painter.line_segment([at(a.handle_in()), at(out)], stroke);
            painter.circle(at(out), 2.8, Color32::WHITE, stroke);
            painter.circle(at(a.handle_in()), 2.8, Color32::WHITE, stroke);
        }
        let size = if i == 0 { 5.6 } else { 3.5 };
        let square = Rect::from_center_size(at(a.point), vec2(size * 2.0, size * 2.0));
        painter.rect(square, 0, Color32::WHITE, stroke, egui::StrokeKind::Middle);
    }
}

/// The Crop tools: everything outside the box darkened with eight handles, or the four-sided area with its corners.
fn crop_overlay(painter: &Painter, t: ViewTransform, doc: &Document, crop: &crate::tools::crop::CropOverlay) {
    use crate::tools::crop::CropOverlay;
    let at = |p: PointD| t.to_screen(p.x as f32, p.y as f32);
    let accent = Stroke::new(1.5, colors::SELECTION_ACCENT);
    match crop {
        CropOverlay::Rect(r) => {
            let whole = Rect::from_two_pos(
                at(PointD::new(0.0, 0.0)),
                at(PointD::new(f64::from(doc.width()), f64::from(doc.height()))),
            );
            let inner = Rect::from_two_pos(
                at(PointD::new(f64::from(r.x), f64::from(r.y))),
                at(PointD::new(f64::from(r.x + r.width), f64::from(r.y + r.height))),
            );
            let shade = Color32::from_black_alpha(150);
            for strip in [
                Rect::from_min_max(whole.min, egui::pos2(whole.max.x, inner.min.y)),
                Rect::from_min_max(egui::pos2(whole.min.x, inner.max.y), whole.max),
                Rect::from_min_max(egui::pos2(whole.min.x, inner.min.y), egui::pos2(inner.min.x, inner.max.y)),
                Rect::from_min_max(egui::pos2(inner.max.x, inner.min.y), egui::pos2(whole.max.x, inner.max.y)),
            ] {
                painter.rect_filled(strip, 0, shade);
            }
            painter.rect_stroke(inner, 0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Middle);
            let c = inner.center();
            for p in [
                inner.left_top(),
                egui::pos2(c.x, inner.top()),
                inner.right_top(),
                egui::pos2(inner.left(), c.y),
                egui::pos2(inner.right(), c.y),
                inner.left_bottom(),
                egui::pos2(c.x, inner.bottom()),
                inner.right_bottom(),
            ] {
                painter.rect(
                    Rect::from_center_size(p, vec2(9.0, 9.0)),
                    0,
                    Color32::WHITE,
                    Stroke::new(1.0, Color32::from_gray(40)),
                    egui::StrokeKind::Middle,
                );
            }
        }
        CropOverlay::Quad(points) => {
            let screen: Vec<egui::Pos2> = points.iter().map(|p| at(*p)).collect();
            painter.add(Shape::convex_polygon(screen.clone(), colors::SELECTION_ACCENT.gamma_multiply(0.2), accent));
            for p in screen {
                painter.circle(p, 6.0, Color32::WHITE, accent);
            }
        }
    }
}

/// The Curve tool's line while drawing, and its points (squares) with the handles (circles) of the selected point.
fn curve_handles(painter: &Painter, t: ViewTransform, c: &crate::tools::curve::CurveOverlay) {
    let stroke = Stroke::new(1.2, colors::SELECTION_ACCENT);
    let at = |p: PointD| t.to_screen(p.x as f32, p.y as f32);
    if c.line.len() >= 2 {
        painter.add(Shape::line(c.line.iter().map(|p| at(*p)).collect(), Stroke::new(1.5, colors::SELECTION_ACCENT)));
    }
    for (i, n) in c.nodes.iter().enumerate() {
        if c.drawing || c.selected == Some(i) {
            for (has, handle) in [(n.has_out(), n.handle_out), (n.has_in(), n.handle_in)] {
                if has {
                    painter.line_segment([at(n.point), at(handle)], stroke);
                    painter.circle(at(handle), 2.8, Color32::WHITE, stroke);
                }
            }
        }
        let size = (if c.drawing && i == 0 { 5.6 } else { 3.5 }) * if c.selected == Some(i) { 1.3 } else { 1.0 };
        let square = Rect::from_center_size(at(n.point), vec2(size * 2.0, size * 2.0));
        let fill = if c.selected == Some(i) { colors::SELECTION_ACCENT } else { Color32::WHITE };
        painter.rect(square, 0, fill, stroke, egui::StrokeKind::Middle);
    }
}

/// The frame around each selected layer (turned with its object); with one layer selected, its resize handles
/// and the round rotate handle above it (C# `FrameOutline` / `FrameHandles`). While reshaping: the corner points.
fn layer_frames(
    painter: &Painter,
    cache: &mut OverlayCache,
    t: ViewTransform,
    doc: &Document,
    reshape: &[PointD],
    part: Option<usize>,
    editing: bool,
) {
    let accent = Stroke::new(1.5, colors::SELECTION_ACCENT);
    let at = |p: PointD| t.to_screen(p.x as f32, p.y as f32);
    let ids = doc.selected_ids();
    cache.bounds.retain(|(id, _, _)| ids.contains(id));
    let single = ids.len() == 1;
    for id in ids {
        let Some(layer) = doc.layer(id).filter(|l| l.visible && !l.is_adjustment()) else { continue };
        let part_object = part
            .filter(|_| doc.active_id() == Some(id))
            .and_then(|i| layer.object().and_then(|o| o.part(i)))
            .map(|m| m.object);
        let frame = match part_object.as_ref().or(layer.object()).and_then(|o| o.box_()) {
            Some(b) => b,
            None => {
                let signature = layer_signature(layer, doc.bounds());
                let bounds = match cache.bounds.iter().find(|(i, s, _)| *i == id && *s == signature) {
                    Some((_, _, b)) => *b,
                    None => {
                        let b = layer.visible_bounds();
                        cache.bounds.retain(|(i, _, _)| *i != id);
                        cache.bounds.push((id, signature, b));
                        b
                    }
                };
                // A layer that fills the whole canvas (a background) gets no frame: it would only trace the page edge.
                if bounds.is_empty() || bounds.intersect(&doc.bounds()) == doc.bounds() {
                    continue;
                }
                ObjectBox::new(bounds.x as f64, bounds.y as f64, bounds.width as f64, bounds.height as f64, 0.0)
            }
        };
        let corners: Vec<Pos2> = frame.corners().iter().map(|c| at(*c)).collect();
        painter.add(Shape::closed_line(corners, accent));
        if !single || layer.locked || editing {
            continue;
        }
        if !reshape.is_empty() {
            for p in reshape {
                painter.circle(at(*p), 5.0, Color32::WHITE, accent);
            }
            continue;
        }
        let zoom = t.zoom as f64;
        let top = handles::position(&frame, handles::Handle::N, zoom);
        let rotate = handles::position(&frame, handles::Handle::Rotate, zoom);
        painter.line_segment([at(top), at(rotate)], accent);
        painter.circle(at(rotate), 5.5, Color32::WHITE, accent);
        let size = handles::HANDLE_SIZE as f32;
        for h in handles::handles_for(part_object.as_ref().or(layer.object())) {
            let r = Rect::from_center_size(at(handles::position(&frame, *h, zoom)), vec2(size, size));
            painter.rect(r, 2, Color32::WHITE, accent, egui::StrokeKind::Middle);
        }
    }
}
