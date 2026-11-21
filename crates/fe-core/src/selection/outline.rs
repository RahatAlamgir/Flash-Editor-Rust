//! Traces the outline of a mask into closed polygons along pixel edges (C# `MaskOutline`), so any selection
//! (magic wand, quick select, combined selections) can be shown with marching ants. Holes come out as separate
//! loops; drawn with the even-odd rule they look right.

use std::collections::HashMap;

use crate::imaging::{PixelRect, PointD};

const NONE: u64 = u64::MAX;

fn key(x: i32, y: i32) -> u64 {
    ((y as u32 as u64) << 32) | x as u32 as u64
}

fn point(k: u64) -> PointD {
    PointD::new((k & 0xFFFF_FFFF) as u32 as i32 as f64, (k >> 32) as u32 as i32 as f64)
}

/// The outline loops of the pixels with a value of at least `threshold`. `mask` covers `rect` row by row;
/// everything outside `rect` counts as not selected. Points are in canvas pixels.
pub fn trace(mask: &[u8], rect: PixelRect, threshold: u8) -> Vec<Vec<PointD>> {
    let (w, h) = (rect.width, rect.height);
    if w <= 0 || h <= 0 {
        return Vec::new();
    }
    let inside = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && mask[(y * w + x) as usize] >= threshold;

    // Directed boundary edges between pixel corners, keeping the inside on the right-hand side. A corner has
    // at most two edges leaving it (where two selected pixels touch only at a corner).
    let mut edges: HashMap<u64, (u64, u64)> = HashMap::new();
    let mut order: Vec<u64> = Vec::new();
    let mut add = |x1: i32, y1: i32, x2: i32, y2: i32| {
        let from = key(x1 + rect.x, y1 + rect.y);
        let to = key(x2 + rect.x, y2 + rect.y);
        let slot = edges.entry(from).or_insert_with(|| {
            order.push(from);
            (NONE, NONE)
        });
        if slot.0 == NONE {
            slot.0 = to;
        } else {
            slot.1 = to;
        }
    };
    for y in 0..h {
        for x in 0..w {
            if !inside(x, y) {
                continue;
            }
            if !inside(x, y - 1) {
                add(x, y, x + 1, y); // top, left to right
            }
            if !inside(x + 1, y) {
                add(x + 1, y, x + 1, y + 1); // right, downwards
            }
            if !inside(x, y + 1) {
                add(x + 1, y + 1, x, y + 1); // bottom, right to left
            }
            if !inside(x - 1, y) {
                add(x, y + 1, x, y); // left, upwards
            }
        }
    }

    let mut loops = Vec::new();
    for start in order {
        if !edges.contains_key(&start) {
            continue;
        }
        let mut points = Vec::new();
        let mut current = start;
        while let Some(slot) = edges.get_mut(&current) {
            let next = if slot.1 != NONE {
                std::mem::replace(&mut slot.1, NONE)
            } else {
                let n = slot.0;
                edges.remove(&current);
                n
            };
            points.push(point(current));
            current = next;
            if current == start {
                break;
            }
        }
        let simplified = remove_straight_points(points);
        if simplified.len() >= 3 {
            loops.push(simplified);
        }
    }
    loops
}

/// Drops points that lie on a straight line between their neighbors.
fn remove_straight_points(points: Vec<PointD>) -> Vec<PointD> {
    if points.len() < 4 {
        return points;
    }
    let n = points.len();
    (0..n)
        .filter(|&i| {
            let (prev, p, next) = (points[(i + n - 1) % n], points[i], points[(i + 1) % n]);
            !((prev.x == p.x && p.x == next.x) || (prev.y == p.y && p.y == next.y))
        })
        .map(|i| points[i])
        .collect()
}
