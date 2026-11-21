//! Small preview pictures (layer and page thumbnails). Reads at most 4 x 4 source pixels per
//! thumbnail pixel, so even huge pictures are fast. Same method as the C# `Thumbnail`.

use super::PixelBuffer;
use super::color_math::round_even;

/// The thumbnail size that fits in `max_width` x `max_height`, keeps the shape and never enlarges.
pub fn fit_size(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let scale = (max_width as f64 / width as f64).min(max_height as f64 / height as f64).min(1.0);
    let w = round_even(width as f64 * scale).max(1.0) as u32;
    let h = round_even(height as f64 * scale).max(1.0) as u32;
    (w, h)
}

/// A thumbnail of `source` that fits in `max_width` x `max_height`.
pub fn create_thumbnail(source: &PixelBuffer, max_width: u32, max_height: u32) -> PixelBuffer {
    let (w, h) = fit_size(source.width(), source.height(), max_width, max_height);
    let mut result = PixelBuffer::new(w, h);
    fill_thumbnail(source, &mut result);
    result
}

/// Draws `source` scaled into `target`. Colors are weighted by alpha so half-transparent edges do not darken.
pub fn fill_thumbnail(source: &PixelBuffer, target: &mut PixelBuffer) {
    const SAMPLES: u64 = 4;
    let (sw, sh) = (source.width() as u64, source.height() as u64);
    let (tw, th) = (target.width() as u64, target.height() as u64);
    let src = source.data();
    for ty in 0..th {
        let y0 = ty * sh / th;
        let cell_h = ((ty + 1) * sh / th).saturating_sub(y0).max(1);
        let ny = cell_h.min(SAMPLES);
        for tx in 0..tw {
            let x0 = tx * sw / tw;
            let cell_w = ((tx + 1) * sw / tw).saturating_sub(x0).max(1);
            let nx = cell_w.min(SAMPLES);
            let (mut a, mut r, mut g, mut b, mut count) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for sy in 0..ny {
                let y = (sh - 1).min(y0 + ((sy as f64 + 0.5) * cell_h as f64 / ny as f64) as u64);
                for sx in 0..nx {
                    let x = (sw - 1).min(x0 + ((sx as f64 + 0.5) * cell_w as f64 / nx as f64) as u64);
                    let i = ((y * sw + x) * 4) as usize;
                    let alpha = src[i + 3] as u64;
                    a += alpha;
                    r += src[i] as u64 * alpha;
                    g += src[i + 1] as u64 * alpha;
                    b += src[i + 2] as u64 * alpha;
                    count += 1;
                }
            }
            let o = ((ty * tw + tx) * 4) as usize;
            let out = &mut target.data_mut()[o..o + 4];
            if a == 0 || count == 0 {
                out.fill(0);
            } else {
                out.copy_from_slice(&[(r / a) as u8, (g / a) as u8, (b / a) as u8, (a / count) as u8]);
            }
        }
    }
}
