//! Maths for curves made of Bezier segments (C# `CurvePath`): finding a spot, adding a point, smooth / sharp
//! points, fitting a box around a drawn curve. Points of a shape are fractions of its box; the editing helpers
//! work in canvas pixels, so `to_canvas` / `from_canvas` convert (the box may be moved, turned or mirrored).

use super::{CurveNode, ObjectBox};
use crate::imaging::PointD;
use crate::selection::bezier::flatten_cubic;

fn lerp(a: PointD, b: PointD, t: f64) -> PointD {
    PointD::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

fn cubic(p0: PointD, p1: PointD, p2: PointD, p3: PointD, t: f64) -> PointD {
    let (a, b, c) = (lerp(p0, p1, t), lerp(p1, p2, t), lerp(p2, p3, t));
    lerp(lerp(a, b, t), lerp(b, c, t), t)
}

/// How many segments the curve has (a closed curve has one more: the last point joins the first).
pub fn segment_count(nodes: usize, closed: bool) -> usize {
    match nodes {
        0 | 1 => 0,
        n if closed => n,
        n => n - 1,
    }
}

/// True when both handles exist and lie on one line through the point (a smooth join).
pub fn is_smooth(n: &CurveNode) -> bool {
    if !n.has_in() || !n.has_out() {
        return false;
    }
    let (ax, ay) = (n.handle_in.x - n.point.x, n.handle_in.y - n.point.y);
    let (bx, by) = (n.handle_out.x - n.point.x, n.handle_out.y - n.point.y);
    let scale = (ax * ax + ay * ay).sqrt() * (bx * bx + by * by).sqrt();
    scale > 1e-12 && (ax * by - ay * bx).abs() / scale < 0.02 && ax * bx + ay * by < 0.0
}

/// A smooth point: the handles are opposite each other, as far apart as `out` is from the point.
pub fn smooth_node(point: PointD, out: PointD) -> CurveNode {
    CurveNode { point, handle_in: PointD::new(2.0 * point.x - out.x, 2.0 * point.y - out.y), handle_out: out }
}

pub fn moved(n: &CurveNode, dx: f64, dy: f64) -> CurveNode {
    n.mapped(|p| PointD::new(p.x + dx, p.y + dy))
}

/// The curve as points close enough together to look smooth.
pub fn flatten(nodes: &[CurveNode], closed: bool) -> Vec<PointD> {
    let mut points = Vec::new();
    let Some(first) = nodes.first() else { return points };
    points.push(first.point);
    for s in 0..segment_count(nodes.len(), closed) {
        let (a, b) = (&nodes[s], &nodes[(s + 1) % nodes.len()]);
        flatten_cubic(a.point, a.handle_out, b.handle_in, b.point, 0.2, &mut points);
    }
    points
}

/// Puts a curve given in canvas pixels into the smallest box around it; the points become fractions of that box.
/// A straight horizontal or vertical curve gets a box one pixel thick.
pub fn fit(canvas_nodes: &[CurveNode], closed: bool) -> (ObjectBox, Vec<CurveNode>) {
    let points = flatten(canvas_nodes, closed);
    let (mut l, mut t, mut r, mut b) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in &points {
        l = l.min(p.x);
        r = r.max(p.x);
        t = t.min(p.y);
        b = b.max(p.y);
    }
    if points.is_empty() {
        return (ObjectBox::new(0.0, 0.0, 1.0, 1.0, 0.0), Vec::new());
    }
    let (w, h) = ((r - l).max(1.0), (b - t).max(1.0));
    let unit = canvas_nodes.iter().map(|n| n.mapped(|p| PointD::new((p.x - l) / w, (p.y - t) / h))).collect();
    (ObjectBox::new(l, t, w, h, 0.0), unit)
}

/// The place on the curve nearest to `point`: which segment, how far along it (0..1) and how far away.
pub fn nearest(nodes: &[CurveNode], closed: bool, point: PointD) -> Option<(usize, f64, f64)> {
    let mut best: Option<(usize, f64, f64)> = None;
    for s in 0..segment_count(nodes.len(), closed) {
        let (a, b) = (&nodes[s], &nodes[(s + 1) % nodes.len()]);
        for i in 0..=40 {
            let t = i as f64 / 40.0;
            let p = cubic(a.point, a.handle_out, b.handle_in, b.point, t);
            let d = p.distance_to(&point);
            if best.is_none_or(|(_, _, bd)| d < bd) {
                best = Some((s, t, d));
            }
        }
    }
    best
}

/// A new point on a segment at `t` that keeps the curve exactly the same (de Casteljau).
pub fn split(nodes: &[CurveNode], segment: usize, t: f64) -> Vec<CurveNode> {
    let mut result = nodes.to_vec();
    let next = (segment + 1) % nodes.len();
    let (p0, p1, p2, p3) = (nodes[segment].point, nodes[segment].handle_out, nodes[next].handle_in, nodes[next].point);
    let (a, b, c) = (lerp(p0, p1, t), lerp(p1, p2, t), lerp(p2, p3, t));
    let (d, e) = (lerp(a, b, t), lerp(b, c, t));
    let q = lerp(d, e, t);
    result[segment].handle_out = a;
    result[next].handle_in = c;
    result.insert(segment + 1, CurveNode { point: q, handle_in: d, handle_out: e });
    result
}

/// Gives a corner point smooth handles (along the line from the previous to the next point), or takes them away.
pub fn toggle_smooth(nodes: &[CurveNode], closed: bool, index: usize) -> Vec<CurveNode> {
    let mut result = nodes.to_vec();
    let node = nodes[index];
    if node.has_in() || node.has_out() {
        result[index] = CurveNode::corner(node.point);
        return result;
    }
    let n = nodes.len();
    let (mut prev, mut next) = (nodes[(index + n - 1) % n].point, nodes[(index + 1) % n].point);
    if !closed && index == 0 {
        prev = node.point;
    }
    if !closed && index == n - 1 {
        next = node.point;
    }
    let (dx, dy) = ((next.x - prev.x) / 6.0, (next.y - prev.y) / 6.0);
    if dx == 0.0 && dy == 0.0 {
        return result;
    }
    result[index] = CurveNode {
        point: node.point,
        handle_in: PointD::new(node.point.x - dx, node.point.y - dy),
        handle_out: PointD::new(node.point.x + dx, node.point.y + dy),
    };
    result
}

/// Canvas position of a point given as a fraction of the box (turning and mirroring included).
pub fn to_canvas(b: &ObjectBox, flips: (bool, bool), p: PointD) -> PointD {
    let (mut lx, mut ly) = (p.x * b.width - b.width / 2.0, p.y * b.height - b.height / 2.0);
    if flips.0 {
        lx = -lx;
    }
    if flips.1 {
        ly = -ly;
    }
    b.to_canvas(lx, ly)
}

/// The curve points in canvas pixels.
pub fn nodes_to_canvas(b: &ObjectBox, flips: (bool, bool), nodes: &[CurveNode]) -> Vec<CurveNode> {
    nodes.iter().map(|n| n.mapped(|p| to_canvas(b, flips, p))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> PointD {
        PointD::new(x, y)
    }

    #[test]
    fn splitting_keeps_the_curve_and_adds_a_point() {
        let nodes = vec![
            CurveNode { point: p(0.0, 0.0), handle_in: p(0.0, 0.0), handle_out: p(0.0, 10.0) },
            CurveNode { point: p(10.0, 10.0), handle_in: p(0.0, 10.0), handle_out: p(10.0, 10.0) },
        ];
        let more = split(&nodes, 0, 0.5);
        assert_eq!(3, more.len());
        let before = cubic(nodes[0].point, nodes[0].handle_out, nodes[1].handle_in, nodes[1].point, 0.5);
        assert!(more[1].point.distance_to(&before) < 1e-9);
    }

    #[test]
    fn fit_puts_the_points_in_a_box_and_toggle_smooth_flips() {
        let canvas =
            vec![CurveNode::corner(p(10.0, 20.0)), CurveNode::corner(p(50.0, 20.0)), CurveNode::corner(p(30.0, 60.0))];
        let (b, unit) = fit(&canvas, true);
        assert_eq!((10.0, 20.0, 40.0, 40.0), (b.x, b.y, b.width, b.height));
        assert_eq!(p(1.0, 0.0), unit[1].point);
        let smooth = toggle_smooth(&canvas, true, 1);
        assert!(smooth[1].has_in() && smooth[1].has_out());
        assert!(!toggle_smooth(&smooth, true, 1)[1].has_out());
    }

    #[test]
    fn mapping_follows_the_turned_box() {
        let b = ObjectBox::new(0.0, 0.0, 20.0, 10.0, 90.0);
        let c = to_canvas(&b, (false, false), p(1.0, 0.5));
        assert!((c.x - 10.0).abs() < 1e-9 && (c.y - 15.0).abs() < 1e-9, "{c:?}");
    }
}
