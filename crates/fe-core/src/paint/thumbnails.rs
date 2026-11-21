//! Small previews of a layer or a whole page, made by sampling a few pixels per preview pixel. They never
//! build a canvas-sized copy, so they stay fast and light even for huge pictures.

use crate::document::{Document, Layer};
use crate::imaging::{PixelBuffer, PixelRect, fit_size};

use super::compositor::composite_pixel;

/// A preview of one layer's pixels inside the canvas, fitting in `max_w` x `max_h`.
pub fn layer_thumbnail(layer: &Layer, canvas: PixelRect, max_w: u32, max_h: u32) -> PixelBuffer {
    sampled(canvas, max_w, max_h, 4, |x, y| layer.pixels().get(x, y))
}

/// A preview of the whole page (all visible layers combined), fitting in `max_w` x `max_h`.
pub fn page_thumbnail(doc: &Document, max_w: u32, max_h: u32) -> PixelBuffer {
    sampled(doc.bounds(), max_w, max_h, 2, |x, y| composite_pixel(doc, x, y))
}

/// Builds a preview by averaging `samples` x `samples` points per preview pixel (alpha-weighted, so see-through
/// edges do not turn dark).
fn sampled(area: PixelRect, max_w: u32, max_h: u32, samples: u32, read: impl Fn(i32, i32) -> [u8; 4]) -> PixelBuffer {
    let (w, h) = fit_size(area.width.max(1) as u32, area.height.max(1) as u32, max_w, max_h);
    let mut out = PixelBuffer::new(w, h);
    let cell_w = area.width as f64 / w as f64;
    let cell_h = area.height as f64 / h as f64;
    for ty in 0..h {
        for tx in 0..w {
            let (mut r, mut g, mut b, mut a, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for sy in 0..samples {
                let y = area.y + ((ty as f64 + (sy as f64 + 0.5) / samples as f64) * cell_h) as i32;
                for sx in 0..samples {
                    let x = area.x + ((tx as f64 + (sx as f64 + 0.5) / samples as f64) * cell_w) as i32;
                    let p = read(x, y);
                    let alpha = p[3] as u64;
                    r += p[0] as u64 * alpha;
                    g += p[1] as u64 * alpha;
                    b += p[2] as u64 * alpha;
                    a += alpha;
                    n += 1;
                }
            }
            if let (Some(r), Some(g), Some(b), Some(alpha)) =
                (r.checked_div(a), g.checked_div(a), b.checked_div(a), a.checked_div(n))
            {
                out.set_pixel(tx, ty, crate::imaging::ColorRgba::new(r as u8, g as u8, b as u8, alpha as u8));
            }
        }
    }
    out
}
