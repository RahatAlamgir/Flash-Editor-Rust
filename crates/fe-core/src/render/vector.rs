//! A small drawing helper for the generated objects (charts, QR codes, barcodes): rectangles, circles, arcs, lines
//! and labels in an object's box space, mapped to the pixmap by a transform.

use tiny_skia::{FillRule, LineCap, LineJoin, Path, PathBuilder, Pixmap, Rect, Stroke, Transform};

use super::draw::solid;
use super::label::Label;
use crate::imaging::ColorRgba;

/// Draws on a pixmap in box space.
pub struct Canvas<'a> {
    pub pm: &'a mut Pixmap,
    pub t: Transform,
}

impl Canvas<'_> {
    pub fn fill(&mut self, path: &Path, c: ColorRgba) {
        self.pm.fill_path(path, &solid(c), FillRule::Winding, self.t, None);
    }

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, c: ColorRgba) {
        if let Some(r) = Rect::from_xywh(x as f32, y as f32, w.max(0.01) as f32, h.max(0.01) as f32) {
            self.fill(&PathBuilder::from_rect(r), c);
        }
    }

    pub fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64, c: ColorRgba) {
        if let Some(p) = round_rect_path(x, y, w, h, r) {
            self.fill(&p, c);
        }
    }

    pub fn stroke(&mut self, path: &Path, c: ColorRgba, width: f64, round: bool) {
        let stroke = Stroke {
            width: width.max(0.1) as f32,
            line_cap: if round { LineCap::Round } else { LineCap::Butt },
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        self.pm.stroke_path(path, &solid(c), &stroke, self.t, None);
    }

    pub fn line(&mut self, a: (f64, f64), b: (f64, f64), c: ColorRgba, width: f64, round: bool) {
        let mut pb = PathBuilder::new();
        pb.move_to(a.0 as f32, a.1 as f32);
        pb.line_to(b.0 as f32, b.1 as f32);
        if let Some(p) = pb.finish() {
            self.stroke(&p, c, width, round);
        }
    }

    /// A circle, filled and / or with a border.
    pub fn circle(&mut self, cx: f64, cy: f64, r: f64, fill: Option<ColorRgba>, border: Option<(ColorRgba, f64)>) {
        let Some(p) = PathBuilder::from_circle(cx as f32, cy as f32, r.max(0.01) as f32) else { return };
        if let Some(c) = fill {
            self.fill(&p, c);
        }
        if let Some((c, w)) = border {
            self.stroke(&p, c, w, false);
        }
    }

    /// A label with its top-left corner at (x, y).
    pub fn label(&mut self, l: &Label, x: f64, y: f64, c: ColorRgba) {
        if let Some(p) = &l.path {
            let t = self.t.pre_translate(x as f32, y as f32);
            self.pm.fill_path(p, &solid(c), FillRule::Winding, t, None);
        }
    }
}

/// A rectangle with rounded corners (radius clamped to half the shorter side).
pub fn round_rect_path(x: f64, y: f64, w: f64, h: f64, r: f64) -> Option<Path> {
    let r = r.clamp(0.0, w.min(h) / 2.0);
    let mut pb = PathBuilder::new();
    if r < 0.01 {
        pb.push_rect(Rect::from_xywh(x as f32, y as f32, w.max(0.01) as f32, h.max(0.01) as f32)?);
        return pb.finish();
    }
    pb.move_to((x + r) as f32, y as f32);
    pb.line_to((x + w - r) as f32, y as f32);
    arc(&mut pb, x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    pb.line_to((x + w) as f32, (y + h - r) as f32);
    arc(&mut pb, x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    pb.line_to((x + r) as f32, (y + h) as f32);
    arc(&mut pb, x + r, y + h - r, r, std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
    pb.line_to(x as f32, (y + r) as f32);
    arc(&mut pb, x + r, y + r, r, std::f64::consts::PI, std::f64::consts::FRAC_PI_2);
    pb.close();
    pb.finish()
}

/// Adds an arc of a circle (angles in radians, clockwise on the screen) to the path, starting from the current point
/// (which should be at the start angle) in pieces of at most a quarter turn.
pub fn arc(pb: &mut PathBuilder, cx: f64, cy: f64, r: f64, start: f64, sweep: f64) {
    let pieces = ((sweep.abs() / std::f64::consts::FRAC_PI_2).ceil() as usize).max(1);
    let step = sweep / pieces as f64;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    for i in 0..pieces {
        let (a0, a1) = (start + step * i as f64, start + step * (i + 1) as f64);
        let (s0, c0) = a0.sin_cos();
        let (s1, c1) = a1.sin_cos();
        pb.cubic_to(
            (cx + r * (c0 - k * s0)) as f32,
            (cy + r * (s0 + k * c0)) as f32,
            (cx + r * (c1 + k * s1)) as f32,
            (cy + r * (s1 - k * c1)) as f32,
            (cx + r * c1) as f32,
            (cy + r * s1) as f32,
        );
    }
}

/// A point on a circle.
pub fn on_circle(cx: f64, cy: f64, r: f64, angle: f64) -> (f64, f64) {
    (cx + angle.cos() * r, cy + angle.sin() * r)
}

/// An open arc as a path (for thick strokes).
pub fn arc_path(cx: f64, cy: f64, r: f64, start: f64, sweep: f64) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let p = on_circle(cx, cy, r, start);
    pb.move_to(p.0 as f32, p.1 as f32);
    arc(&mut pb, cx, cy, r, start, sweep);
    pb.finish()
}

/// A closed shape through the points.
pub fn polygon(points: &[(f64, f64)]) -> Option<Path> {
    let mut pb = PathBuilder::new();
    for (i, p) in points.iter().enumerate() {
        if i == 0 {
            pb.move_to(p.0 as f32, p.1 as f32);
        } else {
            pb.line_to(p.0 as f32, p.1 as f32);
        }
    }
    pb.close();
    pb.finish()
}
