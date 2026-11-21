//! The "quick select" brush (C# `QuickSelect`): every dab looks at the colors under the brush and grows the
//! selection from there into connected pixels of similar color, up to a few brush widths away. Dragging along an
//! object selects it piece by piece, following its edges.

use std::collections::VecDeque;

use crate::imaging::{PixelBuffer, PixelRect};

/// How far (in brush radii) a dab may grow past the brush.
pub const GROW_RADII: f64 = 10.0;

fn sq(v: f64) -> f64 {
    v * v
}

/// Adds one dab centered at (cx, cy) to `mask` (one byte per pixel of `image`, 255 = selected). `tolerance`:
/// how different a color may be from the colors under the brush (0..255). Returns the area that changed.
pub fn add_dab(image: &PixelBuffer, mask: &mut [u8], cx: f64, cy: f64, radius: f64, tolerance: u8) -> PixelRect {
    let w = image.width() as i32;
    let radius = radius.max(1.0);
    let in_brush = |x: i32, y: i32| sq(x as f64 + 0.5 - cx) + sq(y as f64 + 0.5 - cy) <= radius * radius;
    let brush = PixelRect::from_bounds(cx - radius, cy - radius, cx + radius + 1.0, cy + radius + 1.0)
        .intersect(&image.bounds());
    if brush.is_empty() {
        return PixelRect::EMPTY;
    }
    let data = image.data();
    let px = |x: i32, y: i32| {
        let i = ((y * w + x) * 4) as usize;
        &data[i..i + 4]
    };

    // The average color under the brush (the color the selection is "painted" with).
    let (mut r, mut g, mut b, mut count) = (0.0, 0.0, 0.0, 0.0);
    for y in brush.y..brush.bottom() {
        for x in brush.x..brush.right() {
            let p = px(x, y);
            if in_brush(x, y) && p[3] >= 16 {
                r += p[0] as f64;
                g += p[1] as f64;
                b += p[2] as f64;
                count += 1.0;
            }
        }
    }
    if count == 0.0 {
        return PixelRect::EMPTY;
    }
    (r, g, b) = (r / count, g / count, b / count);
    let limit = tolerance as f64 * tolerance as f64 * 3.0;
    let matches = |x: i32, y: i32| {
        let p = px(x, y);
        p[3] >= 16 && sq(p[0] as f64 - r) + sq(p[1] as f64 - g) + sq(p[2] as f64 - b) <= limit
    };

    // Grow from the matching pixels under the brush, inside a window around the dab.
    let reach = radius * GROW_RADII;
    let window =
        PixelRect::from_bounds(cx - reach, cy - reach, cx + reach + 1.0, cy + reach + 1.0).intersect(&image.bounds());
    let mut visited = vec![false; window.area() as usize];
    let mut queue = VecDeque::new();
    let mut changed = PixelRect::EMPTY;
    let mut take = |x: i32, y: i32, queue: &mut VecDeque<(i32, i32)>| {
        let v = ((y - window.y) * window.width + (x - window.x)) as usize;
        if visited[v] {
            return;
        }
        visited[v] = true;
        if !matches(x, y) {
            return;
        }
        mask[(y * w + x) as usize] = 255;
        queue.push_back((x, y));
        changed = changed.union(&PixelRect::new(x, y, 1, 1));
    };
    for y in brush.y..brush.bottom() {
        for x in brush.x..brush.right() {
            if in_brush(x, y) {
                take(x, y, &mut queue);
            }
        }
    }
    while let Some((x, y)) = queue.pop_front() {
        if x > window.x {
            take(x - 1, y, &mut queue);
        }
        if x < window.right() - 1 {
            take(x + 1, y, &mut queue);
        }
        if y > window.y {
            take(x, y - 1, &mut queue);
        }
        if y < window.bottom() - 1 {
            take(x, y + 1, &mut queue);
        }
    }
    changed
}
