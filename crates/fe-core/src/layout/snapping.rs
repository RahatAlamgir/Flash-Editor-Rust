//! Snapping while something is dragged (C# `SnapSession`): edges and centers line up with other objects, the
//! canvas, the grid and the guides; equal gaps between neighbours ("equal spacing") snap too. Made once when a
//! drag starts and asked on every mouse move.

use crate::document::{Document, LayerId};
use crate::imaging::{PixelRect, PointD};

/// What things snap to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapSettings {
    pub objects: bool,
    pub grid: bool,
    pub canvas: bool,
    pub guides: bool,
    pub grid_size: f64,
    pub spacing: bool,
}

/// A gap shown while it equals another gap: from `from` to `to` along x (horizontal) or y, at `cross`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GapMark {
    pub horizontal: bool,
    pub from: f64,
    pub to: f64,
    pub cross: f64,
    pub value: f64,
}

/// How far to move to line up, and the lines / gaps that lined up (drawn as smart guides).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    pub dx: f64,
    pub dy: f64,
    pub lines_x: Vec<f64>,
    pub lines_y: Vec<f64>,
    pub gaps: Vec<GapMark>,
}

#[derive(Clone, Copy)]
struct Area {
    l: f64,
    t: f64,
    r: f64,
    b: f64,
}

/// One drag's snapping targets.
pub struct SnapSession {
    xs: Vec<f64>,
    ys: Vec<f64>,
    boxes: Vec<Area>,
    settings: SnapSettings,
}

/// The area a layer covers: its object's outline, or its painted pixels.
pub fn layer_area(layer: &crate::document::Layer) -> PixelRect {
    match layer.object() {
        Some(obj) if obj.box_().is_some() => crate::render::draw::visual_bounds(obj, 0.0),
        _ => layer.visible_bounds(),
    }
}

type Range = (f64, f64, f64, f64);

impl SnapSession {
    /// Looks at every layer except `exclude` (the layers being dragged).
    pub fn new(
        doc: &Document,
        exclude: &[LayerId],
        guides_x: &[f64],
        guides_y: &[f64],
        settings: SnapSettings,
    ) -> Self {
        let mut s = Self { xs: Vec::new(), ys: Vec::new(), boxes: Vec::new(), settings };
        if settings.objects || settings.spacing {
            for layer in doc.layers() {
                if exclude.contains(&layer.id()) || !layer.visible || layer.opacity() <= 0.02 || layer.is_adjustment() {
                    continue;
                }
                let b = layer_area(layer);
                if b.is_empty()
                    || (layer.object().is_none() && b.width >= doc.width() as i32 && b.height >= doc.height() as i32)
                {
                    continue; // a background
                }
                let a = Area { l: b.x as f64, t: b.y as f64, r: b.right() as f64, b: b.bottom() as f64 };
                if settings.objects {
                    s.xs.extend([a.l, (a.l + a.r) / 2.0, a.r]);
                    s.ys.extend([a.t, (a.t + a.b) / 2.0, a.b]);
                }
                s.boxes.push(a);
            }
        }
        if settings.canvas {
            let (w, h) = (doc.width() as f64, doc.height() as f64);
            s.xs.extend([0.0, w / 2.0, w]);
            s.ys.extend([0.0, h / 2.0, h]);
        }
        if settings.guides {
            s.xs.extend_from_slice(guides_x);
            s.ys.extend_from_slice(guides_y);
        }
        s
    }

    /// A rectangle: its edges and center snap to the nearest line within `threshold`, or to equal gaps.
    pub fn snap_box(&self, left: f64, top: f64, right: f64, bottom: f64, threshold: f64) -> SnapResult {
        let (lx, mx) = self.best(&[left, (left + right) / 2.0, right], &self.xs, threshold);
        let (ly, my) = self.best(&[top, (top + bottom) / 2.0, bottom], &self.ys, threshold);
        let mut gaps = Vec::new();
        let (gx, gy) = if self.settings.spacing {
            (
                self.equal_spacing(true, left, right, top, bottom, threshold, &mut gaps),
                self.equal_spacing(false, top, bottom, left, right, threshold, &mut gaps),
            )
        } else {
            (None, None)
        };
        let (dx, use_lx, use_gx) = pick(lx, gx);
        let (dy, use_ly, use_gy) = pick(ly, gy);
        gaps.retain(|g| if g.horizontal { use_gx } else { use_gy });
        SnapResult {
            dx,
            dy,
            lines_x: mx.filter(|_| use_lx).into_iter().collect(),
            lines_y: my.filter(|_| use_ly).into_iter().collect(),
            gaps,
        }
    }

    /// A single point (a handle or a corner being dragged).
    pub fn snap_point(&self, p: PointD, threshold: f64) -> SnapResult {
        let (dx, lx) = self.best(&[p.x], &self.xs, threshold);
        let (dy, ly) = self.best(&[p.y], &self.ys, threshold);
        SnapResult {
            dx: dx.unwrap_or(0.0),
            dy: dy.unwrap_or(0.0),
            lines_x: lx.into_iter().collect(),
            lines_y: ly.into_iter().collect(),
            gaps: Vec::new(),
        }
    }

    /// The smallest move (within the threshold) that puts a value on a line or a grid line, and the line.
    fn best(&self, values: &[f64], lines: &[f64], threshold: f64) -> (Option<f64>, Option<f64>) {
        let (mut best, mut line) = (f64::MAX, None);
        for v in values {
            for l in lines {
                let d = l - v;
                if d.abs() <= threshold && d.abs() < best.abs() {
                    (best, line) = (d, Some(*l));
                }
            }
        }
        if self.settings.grid && self.settings.grid_size >= 1.0 {
            let g = self.settings.grid_size;
            for v in values {
                let d = (v / g).round_ties_even() * g - v;
                if d.abs() <= threshold && d.abs() < best.abs() {
                    (best, line) = (d, None);
                }
            }
        }
        if best == f64::MAX { (None, None) } else { (Some(best), line) }
    }

    /// Equal spacing along one direction among the objects that share a row (C# `EqualSpacing`).
    #[allow(clippy::too_many_arguments)]
    fn equal_spacing(
        &self,
        horizontal: bool,
        start: f64,
        end: f64,
        cross_start: f64,
        cross_end: f64,
        threshold: f64,
        marks: &mut Vec<GapMark>,
    ) -> Option<f64> {
        let mut row: Vec<Range> = self
            .boxes
            .iter()
            .filter(|b| {
                if horizontal { b.t < cross_end && b.b > cross_start } else { b.l < cross_end && b.r > cross_start }
            })
            .map(|b| if horizontal { (b.l, b.r, b.t, b.b) } else { (b.t, b.b, b.l, b.r) })
            .collect();
        if row.is_empty() {
            return None;
        }
        row.sort_by(|a, b| a.0.total_cmp(&b.0));
        let (size, cross_mid) = (end - start, (cross_start + cross_end) / 2.0);
        let mark = |from: f64, to: f64, cross: f64| GapMark { horizontal, from, to, cross, value: (to - from).round() };
        let row_cross = |a: &Range, b: &Range| (a.2.max(b.2) + a.3.min(b.3)) / 2.0;
        let mut candidates: Vec<(f64, Vec<GapMark>)> = Vec::new();
        // 1) In the middle between two neighbours.
        for w in row.windows(2) {
            let room = w[1].0 - w[0].1;
            if room < size {
                continue;
            }
            let target = w[0].1 + (room - size) / 2.0;
            let d = target - start;
            if d.abs() <= threshold {
                candidates.push((d, vec![mark(w[0].1, target, cross_mid), mark(target + size, w[1].0, cross_mid)]));
            }
        }
        // 2) Next to an object, at a distance two other objects already have between them.
        for w in row.windows(2) {
            let g = w[1].0 - w[0].1;
            if g <= 0.0 {
                continue;
            }
            let reference = mark(w[0].1, w[1].0, row_cross(&w[0], &w[1]));
            for x in &row {
                let right_start = x.1 + g;
                let d1 = right_start - start;
                if d1.abs() <= threshold {
                    candidates.push((d1, vec![reference, mark(x.1, right_start, cross_mid)]));
                }
                let left_end = x.0 - g;
                let d2 = left_end - end;
                if d2.abs() <= threshold {
                    candidates.push((d2, vec![reference, mark(left_end, x.0, cross_mid)]));
                }
            }
        }
        let best = candidates.iter().map(|c| c.0).min_by(|a, b| a.abs().total_cmp(&b.abs()))?;
        for (d, ms) in candidates {
            if (d - best).abs() <= 0.5 {
                for m in ms {
                    if !marks.contains(&m) {
                        marks.push(m);
                    }
                }
            }
        }
        Some(best)
    }
}

/// Per direction the nearer of "a line" and "equal gaps" wins; when both agree both are shown.
fn pick(line: Option<f64>, gap: Option<f64>) -> (f64, bool, bool) {
    match (line, gap) {
        (Some(l), Some(g)) if (g - l).abs() <= 0.5 => (l, true, true),
        (Some(l), Some(g)) if g.abs() < l.abs() => (g, false, true),
        (Some(l), _) => (l, true, false),
        (None, Some(g)) => (g, false, true),
        (None, None) => (0.0, false, false),
    }
}
