//! Fast Gaussian blur approximated by three box blurs (C# `Blur`). Shared by Blur, Sharpen and Enhance.

use rayon::prelude::*;

use crate::imaging::PixelBuffer;
use crate::imaging::color_math::clamp_byte;

/// Blurs a color picture. Premultiplied alpha is used, so see-through areas do not make dark halos.
pub fn gaussian(source: &PixelBuffer, sigma: f64) -> PixelBuffer {
    if sigma < 0.3 {
        return source.clone();
    }
    let (w, h) = (source.width() as usize, source.height() as usize);
    let mut data = vec![0f32; w * h * 4];
    data.par_chunks_mut(w * 4).zip(source.data().par_chunks(w * 4)).for_each(|(d, s)| {
        for (d, s) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)) {
            let a = f32::from(s[3]) / 255.0;
            d[0] = f32::from(s[0]) * a;
            d[1] = f32::from(s[1]) * a;
            d[2] = f32::from(s[2]) * a;
            d[3] = f32::from(s[3]);
        }
    });
    gaussian_in_place(&mut data, w, h, 4, sigma);
    let mut out = PixelBuffer::new(source.width(), source.height());
    out.data_mut().par_chunks_mut(w * 4).zip(data.par_chunks(w * 4)).for_each(|(d, s)| {
        for (d, s) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)) {
            let a = s[3];
            if a < 0.5 {
                continue;
            }
            let inv = 255.0 / a;
            d[0] = clamp_byte(f64::from(s[0] * inv));
            d[1] = clamp_byte(f64::from(s[1] * inv));
            d[2] = clamp_byte(f64::from(s[2] * inv));
            d[3] = clamp_byte(f64::from(a));
        }
    });
    out
}

/// Blurs interleaved float data with any number of channels (1 for a mask).
pub fn gaussian_in_place(data: &mut [f32], width: usize, height: usize, channels: usize, sigma: f64) {
    if sigma < 0.3 || width == 0 || height == 0 {
        return;
    }
    let mut temp = vec![0f32; data.len()];
    for radius in box_radii(sigma, 3) {
        box_rows(data, &mut temp, width, channels, radius);
        transpose(&temp, data, width, height, channels);
        box_rows(data, &mut temp, height, channels, radius);
        transpose(&temp, data, height, width, channels);
    }
}

/// Box sizes whose repeated use approximates a Gaussian of the given sigma.
fn box_radii(sigma: f64, passes: usize) -> Vec<usize> {
    let p = passes as f64;
    let ideal = (12.0 * sigma * sigma / p + 1.0).sqrt();
    let mut lower = ideal.floor() as i64;
    if lower % 2 == 0 {
        lower -= 1;
    }
    let upper = lower + 2;
    let l = lower as f64;
    let m_ideal = (12.0 * sigma * sigma - p * l * l - 4.0 * p * l - 3.0 * p) / (-4.0 * l - 4.0);
    let m = m_ideal.round_ties_even() as i64;
    (0..passes as i64).map(|i| (((if i < m { lower } else { upper }) - 1) / 2).max(0) as usize).collect()
}

/// Blurs every row of `src` into `dst` with a running sum (edges are repeated).
fn box_rows(src: &[f32], dst: &mut [f32], width: usize, ch: usize, r: usize) {
    let scale = 1.0 / (2 * r + 1) as f32;
    let row_len = width * ch;
    dst.par_chunks_mut(row_len).zip(src.par_chunks(row_len)).for_each(|(out, row)| {
        let mut sum = vec![0f32; ch];
        for k in -(r as i64)..=(r as i64) {
            let p = k.clamp(0, width as i64 - 1) as usize * ch;
            for c in 0..ch {
                sum[c] += row[p + c];
            }
        }
        for x in 0..width {
            let add = (x + r + 1).min(width - 1) * ch;
            let remove = x.saturating_sub(r) * ch;
            for c in 0..ch {
                out[x * ch + c] = sum[c] * scale;
                sum[c] += row[add + c] - row[remove + c];
            }
        }
    });
}

/// `dst` (height x width pixels) = `src` (width x height pixels) turned so rows become columns.
fn transpose(src: &[f32], dst: &mut [f32], width: usize, height: usize, ch: usize) {
    // `src` has `height` rows of `width` pixels; `dst` gets `width` rows of `height` pixels.
    dst.par_chunks_mut(height * ch).enumerate().for_each(|(x, out)| {
        for y in 0..height {
            let s = (y * width + x) * ch;
            out[y * ch..y * ch + ch].copy_from_slice(&src[s..s + ch]);
        }
    });
}
