//! Round dabs along a mouse movement (shared by the Remove Brush and the Refine brush), so fast strokes leave no gaps.

use fe_core::imaging::PointD;

/// Calls `dab(x, y)` every quarter brush size on the line from `from` (or just at `to` when there is no earlier point).
pub fn along(from: Option<PointD>, to: PointD, size: f64, mut dab: impl FnMut(f64, f64)) {
    let from = from.unwrap_or(to);
    let distance = from.distance_to(&to);
    let step = (size / 4.0).max(1.0);
    let count = ((distance / step).ceil() as usize).max(1);
    for i in 1..=count {
        let t = i as f64 / count as f64;
        dab(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
    }
}
