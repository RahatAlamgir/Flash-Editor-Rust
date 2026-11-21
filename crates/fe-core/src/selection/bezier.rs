//! Smooth paths for the Pen tool: points with optional curve handles, joined by cubic Bezier curves, and turned
//! into a polygon (C# `PenTool` + WPF `GetFlattenedPathGeometry`).

use crate::imaging::PointD;

/// One point of a pen path. `out` is the curve handle (none = sharp corner).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub point: PointD,
    pub out: Option<PointD>,
}

impl Anchor {
    pub fn corner(point: PointD) -> Self {
        Self { point, out: None }
    }

    /// The handle on the incoming side: the mirror of `out` (a smooth point), or the point itself.
    pub fn handle_in(&self) -> PointD {
        match self.out {
            Some(o) => PointD::new(2.0 * self.point.x - o.x, 2.0 * self.point.y - o.y),
            None => self.point,
        }
    }

    pub fn handle_out(&self) -> PointD {
        self.out.unwrap_or(self.point)
    }
}

/// The point at `t` (0..1) of a cubic curve.
pub fn cubic_point(p0: PointD, c1: PointD, c2: PointD, p3: PointD, t: f64) -> PointD {
    let u = 1.0 - t;
    let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    PointD::new(a * p0.x + b * c1.x + c * c2.x + d * p3.x, a * p0.y + b * c1.y + c * c2.y + d * p3.y)
}

/// Adds the points of a cubic curve (without its start point) to `out`, close enough that no point of the real
/// curve is further than about `tolerance` pixels from the lines.
pub fn flatten_cubic(p0: PointD, c1: PointD, c2: PointD, p3: PointD, tolerance: f64, out: &mut Vec<PointD>) {
    // Straight segment (both handles on the ends): one line.
    if c1 == p0 && c2 == p3 {
        out.push(p3);
        return;
    }
    // Steps from the control polygon's second differences (a standard bound for cubic flattening).
    let ddx = (p0.x - 2.0 * c1.x + c2.x).abs().max((c1.x - 2.0 * c2.x + p3.x).abs());
    let ddy = (p0.y - 2.0 * c1.y + c2.y).abs().max((c1.y - 2.0 * c2.y + p3.y).abs());
    let dd = ddx.hypot(ddy);
    let steps = ((0.75 * dd / tolerance.max(0.01)).sqrt().ceil() as usize).clamp(1, 1000);
    for i in 1..=steps {
        out.push(cubic_point(p0, c1, c2, p3, i as f64 / steps as f64));
    }
}

/// The polygon of a pen path. `closed` joins the last point back to the first with a curve.
pub fn path_points(anchors: &[Anchor], closed: bool, tolerance: f64) -> Vec<PointD> {
    let Some(first) = anchors.first() else { return Vec::new() };
    let mut out = vec![first.point];
    for pair in anchors.windows(2) {
        flatten_cubic(pair[0].point, pair[0].handle_out(), pair[1].handle_in(), pair[1].point, tolerance, &mut out);
    }
    if closed && anchors.len() > 2 {
        let (last, first) = (anchors[anchors.len() - 1], anchors[0]);
        flatten_cubic(last.point, last.handle_out(), first.handle_in(), first.point, tolerance, &mut out);
        out.pop(); // the start point is already first
    }
    out
}
