//! The outline of every shape, fitted into a box of `w` x `h` with its top-left at (0, 0) (C#
//! `ShapeGeometryFactory` and `ShapeVertices`). Shapes the C# version builds by combining two outlines (cloud,
//! bubbles, crescent) come back as separate parts that the drawing code combines.

use tiny_skia::{Path, PathBuilder};

use crate::imaging::PointD;
use crate::objects::{BendStyle, CurveNode, ObjectBox, ShapeData, ShapeKind};

/// How the parts of an outline go together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combine {
    /// One outline (filled with the non-zero rule).
    Single,
    /// One path whose inner loops are holes (ring, frame).
    EvenOdd,
    /// All parts together (the border follows the outside only).
    Union,
    /// The first part with the second cut out of it.
    Exclude,
}

/// A shape's outline.
pub struct Outline {
    pub parts: Vec<Path>,
    pub combine: Combine,
    /// A line or an open curve: drawn with its border only.
    pub open: bool,
}

/// A path builder that takes f64 points.
struct Builder(PathBuilder);

impl Builder {
    fn new() -> Self {
        Self(PathBuilder::new())
    }
    fn m(&mut self, x: f64, y: f64) {
        self.0.move_to(x as f32, y as f32);
    }
    fn l(&mut self, x: f64, y: f64) {
        self.0.line_to(x as f32, y as f32);
    }
    fn c(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) {
        self.0.cubic_to(x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32);
    }
    fn close(&mut self) {
        self.0.close();
    }

    /// An elliptical arc from `start` turning by `sweep` degrees (positive = clockwise on screen), without the
    /// first move: the caller has already drawn to its start point.
    fn arc(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, start: f64, sweep: f64) {
        let pieces = ((sweep.abs() / 90.0).ceil() as usize).max(1);
        let step = sweep.to_radians() / pieces as f64;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let mut a = start.to_radians();
        for _ in 0..pieces {
            let b = a + step;
            let (sa, ca) = a.sin_cos();
            let (sb, cb) = b.sin_cos();
            self.c(
                cx + rx * (ca - k * sa),
                cy + ry * (sa + k * ca),
                cx + rx * (cb + k * sb),
                cy + ry * (sb - k * cb),
                cx + rx * cb,
                cy + ry * sb,
            );
            a = b;
        }
    }

    /// A whole ellipse, clockwise.
    fn ellipse(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let (cx, cy, rx, ry) = (x + w / 2.0, y + h / 2.0, w / 2.0, h / 2.0);
        self.m(cx + rx, cy);
        self.arc(cx, cy, rx, ry, 0.0, 360.0);
        self.close();
    }

    /// A rectangle with rounded corners, clockwise.
    fn round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, r: f64) {
        let r = r.clamp(0.0, w.min(h) / 2.0);
        if r <= 0.01 {
            self.polygon(&[(x, y), (x + w, y), (x + w, y + h), (x, y + h)]);
            return;
        }
        self.m(x + r, y);
        self.l(x + w - r, y);
        self.arc(x + w - r, y + r, r, r, -90.0, 90.0);
        self.l(x + w, y + h - r);
        self.arc(x + w - r, y + h - r, r, r, 0.0, 90.0);
        self.l(x + r, y + h);
        self.arc(x + r, y + h - r, r, r, 90.0, 90.0);
        self.l(x, y + r);
        self.arc(x + r, y + r, r, r, 180.0, 90.0);
        self.close();
    }

    /// A closed polygon, always clockwise (so polygons and ellipses can be joined with the non-zero rule).
    fn polygon(&mut self, points: &[(f64, f64)]) {
        if points.len() < 2 {
            return;
        }
        let area: f64 = (0..points.len())
            .map(|i| {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum();
        let ordered: Vec<(f64, f64)> =
            if area < 0.0 { points.iter().rev().copied().collect() } else { points.to_vec() };
        self.m(ordered[0].0, ordered[0].1);
        for p in &ordered[1..] {
            self.l(p.0, p.1);
        }
        self.close();
    }

    fn finish(self) -> Option<Path> {
        self.0.finish()
    }
}

fn single(b: Builder, combine: Combine) -> Option<Outline> {
    Some(Outline { parts: vec![b.finish()?], combine, open: false })
}

/// The corner points of the straight-sided shapes, in the box (0..1), or `None` for curved shapes
/// (C# `ShapeVertices.For`).
pub fn vertices(shape: &ShapeData, bx: &ObjectBox) -> Option<Vec<PointD>> {
    use ShapeKind::*;
    let p = |pts: &[(f64, f64)]| pts.iter().map(|(x, y)| PointD::new(*x, *y)).collect::<Vec<_>>();
    Some(match shape.kind {
        Rectangle => p(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]),
        Triangle => p(&[(0.5, 0.0), (1.0, 1.0), (0.0, 1.0)]),
        RightTriangle => p(&[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0)]),
        Diamond => p(&[(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)]),
        Trapezoid => p(&[(0.2, 0.0), (0.8, 0.0), (1.0, 1.0), (0.0, 1.0)]),
        Parallelogram => p(&[(0.25, 0.0), (1.0, 0.0), (0.75, 1.0), (0.0, 1.0)]),
        Chevron => p(&[(0.0, 0.0), (0.6, 0.0), (1.0, 0.5), (0.6, 1.0), (0.0, 1.0), (0.4, 0.5)]),
        Lightning => p(&[(0.6, 0.0), (0.2, 0.55), (0.47, 0.55), (0.35, 1.0), (0.8, 0.4), (0.52, 0.4), (0.75, 0.0)]),
        Star => star(shape.points.clamp(3, 24) as usize, 0.45),
        Polygon => star(shape.points.clamp(3, 24) as usize, 1.0),
        Pentagon => star(5, 1.0),
        Hexagon => star(6, 1.0),
        Octagon => star(8, 1.0),
        Sparkle => star(4, 0.28),
        Burst => star(16, 0.82),
        Arrow => {
            let (w, h) = (bx.width.max(1.0), bx.height.max(1.0));
            let head = (w * 0.45).min(h * 0.9) / w;
            let neck = 1.0 - head;
            p(&[(0.0, 0.3), (neck, 0.3), (neck, 0.0), (1.0, 0.5), (neck, 1.0), (neck, 0.7), (0.0, 0.7)])
        }
        DoubleArrow => p(&[
            (0.0, 0.5),
            (0.25, 0.0),
            (0.25, 0.3),
            (0.75, 0.3),
            (0.75, 0.0),
            (1.0, 0.5),
            (0.75, 1.0),
            (0.75, 0.7),
            (0.25, 0.7),
            (0.25, 1.0),
        ]),
        Cross => {
            let arm = shape.inner.clamp(0.1, 0.9) * 0.8;
            let (a, b) = ((1.0 - arm) / 2.0, 1.0 - (1.0 - arm) / 2.0);
            p(&[
                (a, 0.0),
                (b, 0.0),
                (b, a),
                (1.0, a),
                (1.0, b),
                (b, b),
                (b, 1.0),
                (a, 1.0),
                (a, b),
                (0.0, b),
                (0.0, a),
                (a, a),
            ])
        }
        Check => p(&[(0.05, 0.55), (0.18, 0.42), (0.40, 0.64), (0.82, 0.15), (0.95, 0.28), (0.40, 0.90)]),
        XMark => p(&[
            (0.18, 0.0),
            (0.5, 0.32),
            (0.82, 0.0),
            (1.0, 0.18),
            (0.68, 0.5),
            (1.0, 0.82),
            (0.82, 1.0),
            (0.5, 0.68),
            (0.18, 1.0),
            (0.0, 0.82),
            (0.32, 0.5),
            (0.0, 0.18),
        ]),
        Home => p(&[(0.5, 0.0), (1.0, 0.45), (0.85, 0.45), (0.85, 1.0), (0.15, 1.0), (0.15, 0.45), (0.0, 0.45)]),
        Tag => p(&[(0.0, 0.15), (0.7, 0.15), (1.0, 0.5), (0.7, 0.85), (0.0, 0.85)]),
        _ => return None,
    })
}

/// A star (inner ratio below 1) or a regular polygon (ratio 1) with the first point at the top.
fn star(points: usize, inner: f64) -> Vec<PointD> {
    let polygon = inner >= 1.0;
    let count = if polygon { points } else { points * 2 };
    (0..count)
        .map(|i| {
            let r = if polygon || i % 2 == 0 { 1.0 } else { inner };
            let a = -std::f64::consts::FRAC_PI_2 + std::f64::consts::TAU * i as f64 / count as f64;
            PointD::new(0.5 + a.cos() * 0.5 * r, 0.5 + a.sin() * 0.5 * r)
        })
        .collect()
}

/// The four control points of a (possibly bent) line in its box (C# `LineCurves.Controls`).
pub fn line_controls(w: f64, h: f64, bend: f64, style: BendStyle) -> [PointD; 4] {
    let (start, end) = (PointD::new(0.0, h / 2.0), PointD::new(w, h / 2.0));
    let bend = bend.clamp(-1.0, 1.0);
    if bend.abs() < 1e-6 {
        return [start, start, end, end];
    }
    let y = h / 2.0;
    if style == BendStyle::Wave {
        let offset = bend * w * 0.45;
        return [start, PointD::new(w / 3.0, y + offset), PointD::new(w * 2.0 / 3.0, y - offset), end];
    }
    let control = y + bend * w * 0.5;
    let k = (control - y) * 2.0 / 3.0;
    [start, PointD::new(w / 3.0, y + k), PointD::new(w * 2.0 / 3.0, y + k), end]
}

/// How far a bent line reaches away from the straight line (C# `LineCurves.Reach`).
pub fn line_reach(w: f64, bend: f64, style: BendStyle) -> f64 {
    bend.clamp(-1.0, 1.0).abs() * w * if style == BendStyle::Wave { 0.3 } else { 0.26 }
}

fn curve(b: &mut Builder, nodes: &[CurveNode], closed: bool, w: f64, h: f64) {
    let p = |q: PointD| (q.x * w, q.y * h);
    let (x, y) = p(nodes[0].point);
    b.m(x, y);
    let segments = if closed { nodes.len() } else { nodes.len() - 1 };
    for s in 0..segments {
        let (a, n) = (nodes[s], nodes[(s + 1) % nodes.len()]);
        let (ex, ey) = p(n.point);
        if !a.has_out() && !n.has_in() {
            b.l(ex, ey);
        } else {
            let ((c1x, c1y), (c2x, c2y)) = (p(a.handle_out), p(n.handle_in));
            b.c(c1x, c1y, c2x, c2y, ex, ey);
        }
    }
    if closed {
        b.close();
    }
}

/// The outline of a shape in a `w` x `h` box (C# `ShapeGeometryFactory.Build`).
pub fn outline(shape: &ShapeData, bx: &ObjectBox) -> Option<Outline> {
    use ShapeKind::*;
    let (w, h) = (bx.width.max(1.0), bx.height.max(1.0));
    let radius = |r: f64, h: f64| shape.corner_radius.clamp(0.0, (r.min(h) / 2.0).max(0.0));
    let inner = shape.inner.clamp(0.05, 0.95);
    let mut b = Builder::new();

    // Straight-sided shapes are drawn from their corner points; a plain rectangle keeps its rounded corners.
    let corners = shape.vertices.clone().or_else(|| if shape.kind == Rectangle { None } else { vertices(shape, bx) });
    if let Some(corners) = corners {
        let pts: Vec<(f64, f64)> = corners.iter().map(|c| (c.x * w, c.y * h)).collect();
        b.polygon(&pts);
        return single(b, Combine::Single);
    }
    match shape.kind {
        Rectangle => b.round_rect(0.0, 0.0, w, h, radius(w, h)),
        Ellipse => b.ellipse(0.0, 0.0, w, h),
        Line => {
            let [a, c1, c2, e] = line_controls(w, h, shape.bend, shape.bend_style);
            b.m(a.x, a.y);
            b.c(c1.x, c1.y, c2.x, c2.y, e.x, e.y);
            return Some(Outline { parts: vec![b.finish()?], combine: Combine::Single, open: true });
        }
        Path => {
            let nodes = shape.nodes.as_deref().filter(|n| n.len() >= 2);
            match nodes {
                Some(nodes) => curve(&mut b, nodes, shape.closed, w, h),
                None => {
                    b.m(0.0, h / 2.0);
                    b.l(w, h / 2.0);
                }
            }
            let open = !shape.closed || nodes.is_none();
            return Some(Outline { parts: vec![b.finish()?], combine: Combine::Single, open });
        }
        Ring => {
            b.ellipse(0.0, 0.0, w, h);
            b.ellipse(w * (1.0 - inner) / 2.0, h * (1.0 - inner) / 2.0, w * inner, h * inner);
            return single(b, Combine::EvenOdd);
        }
        HalfRing => arc(&mut b, w / 2.0, h, w / 2.0, h, 180.0, 180.0, inner),
        HalfCircle => arc(&mut b, w / 2.0, h, w / 2.0, h, 180.0, 180.0, 0.0),
        QuarterCircle => arc(&mut b, 0.0, h, w, h, 270.0, 90.0, 0.0),
        Pie => {
            let (inner, sweep) = (shape.inner.clamp(0.0, 0.95), shape.sweep.clamp(10.0, 360.0));
            if sweep >= 359.5 {
                b.ellipse(0.0, 0.0, w, h);
                if inner > 0.02 {
                    b.ellipse(w * (1.0 - inner) / 2.0, h * (1.0 - inner) / 2.0, w * inner, h * inner);
                    return single(b, Combine::EvenOdd);
                }
            } else {
                arc(&mut b, w / 2.0, h / 2.0, w / 2.0, h / 2.0, -90.0, sweep, if inner > 0.02 { inner } else { 0.0 });
            }
        }
        Frame => {
            let r = radius(w, h);
            b.round_rect(0.0, 0.0, w, h, r);
            let (hw, hh) = (w * inner, h * inner);
            let inset = r.min(hw.min(hh) / 2.0) * 0.5;
            b.round_rect((w - hw) / 2.0, (h - hh) / 2.0, hw, hh, inset);
            return single(b, Combine::EvenOdd);
        }
        Crescent => {
            b.ellipse(0.0, 0.0, w, h);
            let mut cut = Builder::new();
            cut.ellipse(w * inner * 0.6, h * 0.04, w * 0.92, h * 0.92);
            return Some(Outline { parts: vec![b.finish()?, cut.finish()?], combine: Combine::Exclude, open: false });
        }
        Cloud => {
            let mut parts = Vec::new();
            let mut body = Builder::new();
            body.round_rect(w * 0.04, h * 0.5, w * 0.92, h * 0.46, h * 0.23);
            parts.push(body.finish()?);
            for (x, y, bw, bh) in [(0.02, 0.34, 0.42, 0.54), (0.22, 0.04, 0.54, 0.7), (0.52, 0.22, 0.46, 0.6)] {
                let mut bump = Builder::new();
                bump.ellipse(w * x, h * y, w * bw, h * bh);
                parts.push(bump.finish()?);
            }
            return Some(Outline { parts, combine: Combine::Union, open: false });
        }
        SpeechBubble => {
            let body_h = h * 0.78;
            b.round_rect(0.0, 0.0, w, body_h, radius(w, body_h));
            let mut tail = Builder::new();
            tail.polygon(&[(w * 0.18, body_h - 1.0), (w * 0.40, body_h - 1.0), (w * 0.12, h)]);
            return Some(Outline { parts: vec![b.finish()?, tail.finish()?], combine: Combine::Union, open: false });
        }
        OvalBubble => {
            b.ellipse(0.0, 0.0, w, h * 0.82);
            let mut tail = Builder::new();
            tail.polygon(&[(w * 0.2, h * 0.7), (w * 0.4, h * 0.78), (w * 0.1, h)]);
            return Some(Outline { parts: vec![b.finish()?, tail.finish()?], combine: Combine::Union, open: false });
        }
        Heart => {
            b.m(w * 0.5, h);
            b.c(w * 0.15, h * 0.72, 0.0, h * 0.45, 0.0, h * 0.28);
            b.c(0.0, h * 0.06, w * 0.18, 0.0, w * 0.29, 0.0);
            b.c(w * 0.40, 0.0, w * 0.48, h * 0.08, w * 0.5, h * 0.17);
            b.c(w * 0.52, h * 0.08, w * 0.60, 0.0, w * 0.71, 0.0);
            b.c(w * 0.82, 0.0, w, h * 0.06, w, h * 0.28);
            b.c(w, h * 0.45, w * 0.85, h * 0.72, w * 0.5, h);
            b.close();
        }
        Teardrop => {
            b.m(w / 2.0, 0.0);
            b.c(w * 0.62, h * 0.22, w, h * 0.44, w, h * 0.68);
            b.c(w, h * 0.87, w * 0.78, h, w / 2.0, h);
            b.c(w * 0.22, h, 0.0, h * 0.87, 0.0, h * 0.68);
            b.c(0.0, h * 0.44, w * 0.38, h * 0.22, w / 2.0, 0.0);
            b.close();
        }
        Shield => {
            b.m(0.0, h * 0.12);
            b.c(w * 0.2, h * 0.12, w * 0.38, h * 0.08, w * 0.5, 0.0);
            b.c(w * 0.62, h * 0.08, w * 0.8, h * 0.12, w, h * 0.12);
            b.l(w, h * 0.5);
            b.c(w, h * 0.78, w * 0.78, h * 0.92, w * 0.5, h);
            b.c(w * 0.22, h * 0.92, 0.0, h * 0.78, 0.0, h * 0.5);
            b.close();
        }
        _ => b.polygon(&[(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]),
    }
    single(b, Combine::Single)
}

/// A slice of an ellipse from `start` clockwise by `sweep` degrees; `inner` 0 = a pie slice, above 0 = a ring
/// segment with that much cut out.
#[allow(clippy::too_many_arguments)]
fn arc(b: &mut Builder, cx: f64, cy: f64, rx: f64, ry: f64, start: f64, sweep: f64, inner: f64) {
    let on = |rx: f64, ry: f64, a: f64| (cx + a.to_radians().cos() * rx, cy + a.to_radians().sin() * ry);
    let (x0, y0) = on(rx, ry, start);
    b.m(x0, y0);
    b.arc(cx, cy, rx, ry, start, sweep);
    if inner > 0.0 {
        let (x1, y1) = on(rx * inner, ry * inner, start + sweep);
        b.l(x1, y1);
        b.arc(cx, cy, rx * inner, ry * inner, start + sweep, -sweep);
    } else {
        b.l(cx, cy);
    }
    b.close();
}
