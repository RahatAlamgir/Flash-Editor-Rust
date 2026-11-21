//! Fills polygons into a byte mask with the even-odd rule (scanlines through pixel centers), like the C#
//! `PolygonRasterizer`.

use crate::imaging::{PixelRect, PointD};

/// The area a set of polygons can touch (whole pixels).
pub fn polygons_bounds(polygons: &[Vec<PointD>]) -> PixelRect {
    let mut bounds = PixelRect::EMPTY;
    for poly in polygons.iter().filter(|p| p.len() >= 3) {
        let (mut l, mut t, mut r, mut b) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in poly {
            l = l.min(p.x);
            t = t.min(p.y);
            r = r.max(p.x);
            b = b.max(p.y);
        }
        bounds = bounds.union(&PixelRect::from_bounds(l, t, r, b));
    }
    bounds
}

/// Flips mask bytes between 0 and 255 for every pixel whose center lies inside the polygons (even-odd rule over
/// all polygons together, so a polygon inside another makes a hole). `mask` covers `rect` row by row; pixels
/// outside `rect` are skipped.
pub fn fill(mask: &mut [u8], rect: PixelRect, polygons: &[Vec<PointD>]) {
    debug_assert_eq!(mask.len(), rect.area().max(0) as usize);
    let mut edges: Vec<(f64, f64, f64, f64)> = Vec::new();
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    for poly in polygons.iter().filter(|p| p.len() >= 3) {
        for (i, a) in poly.iter().enumerate() {
            let b = poly[(i + 1) % poly.len()];
            if a.y == b.y {
                continue; // horizontal edges never cross a scanline
            }
            edges.push((a.x, a.y, b.x, b.y));
            min_y = min_y.min(a.y.min(b.y));
            max_y = max_y.max(a.y.max(b.y));
        }
    }
    if edges.is_empty() || rect.is_empty() {
        return;
    }
    let y_start = rect.y.max(min_y.floor() as i32);
    let y_end = (rect.bottom() - 1).min(max_y.ceil() as i32);
    let mut crossings: Vec<f64> = Vec::new();
    for y in y_start..=y_end {
        let sy = y as f64 + 0.5; // sample at pixel centers
        crossings.clear();
        for &(x1, y1, x2, y2) in &edges {
            if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
                crossings.push(x1 + (sy - y1) / (y2 - y1) * (x2 - x1));
            }
        }
        if crossings.len() < 2 {
            continue;
        }
        crossings.sort_by(f64::total_cmp);
        let row = ((y - rect.y) * rect.width) as usize;
        for pair in crossings.as_chunks::<2>().0 {
            // Pixel x is inside when its center x + 0.5 lies in [left, right).
            let from = rect.x.max((pair[0] - 0.5).ceil() as i32);
            let to = (rect.right() - 1).min((pair[1] - 0.5).ceil() as i32 - 1);
            for x in from..=to {
                mask[row + (x - rect.x) as usize] ^= 255;
            }
        }
    }
}
