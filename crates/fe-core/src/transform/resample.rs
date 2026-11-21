//! Picture scaling and sub-pixel sampling (C# `Resampler`).

use rayon::prelude::*;

use crate::imaging::PixelBuffer;
use crate::imaging::color_math::clamp_byte;

/// How a picture is resized.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResampleMode {
    /// Smooth, good for photos (bilinear when enlarging, area-averaging when shrinking).
    #[default]
    Smooth,
    /// Hard pixel edges, good for pixel art.
    NearestNeighbor,
    /// Sharper than Smooth (bicubic). A good all-round choice.
    Bicubic,
    /// Sharpest (Lanczos-3). Best for enlarging photos; keeps fine detail when shrinking.
    Lanczos,
}

impl ResampleMode {
    pub const ALL: [ResampleMode; 4] =
        [ResampleMode::Smooth, ResampleMode::Bicubic, ResampleMode::Lanczos, ResampleMode::NearestNeighbor];

    pub fn name(self) -> &'static str {
        match self {
            ResampleMode::Smooth => "Smooth",
            ResampleMode::NearestNeighbor => "Pixel art (nearest)",
            ResampleMode::Bicubic => "Bicubic (sharper)",
            ResampleMode::Lanczos => "Lanczos (sharpest)",
        }
    }
}

pub fn resize(source: &PixelBuffer, width: u32, height: u32, mode: ResampleMode) -> PixelBuffer {
    if width == source.width() && height == source.height() {
        return source.clone();
    }
    if width == 0 || height == 0 || source.width() == 0 || source.height() == 0 {
        return PixelBuffer::new(width.max(1), height.max(1));
    }
    if mode == ResampleMode::NearestNeighbor {
        resize_nearest(source, width, height)
    } else {
        resize_filtered(source, width, height, mode)
    }
}

fn kernel(mode: ResampleMode) -> (fn(f64) -> f64, f64) {
    match mode {
        ResampleMode::Bicubic => (cubic, 2.0),
        ResampleMode::Lanczos => (lanczos3, 3.0),
        _ => (|x| (1.0 - x.abs()).max(0.0), 1.0),
    }
}

/// Catmull-Rom style cubic (a = -0.5).
fn cubic(x: f64) -> f64 {
    const A: f64 = -0.5;
    let x = x.abs();
    if x < 1.0 {
        (A + 2.0) * x * x * x - (A + 3.0) * x * x + 1.0
    } else if x < 2.0 {
        A * x * x * x - 5.0 * A * x * x + 8.0 * A * x - 4.0 * A
    } else {
        0.0
    }
}

fn lanczos3(x: f64) -> f64 {
    let x = x.abs();
    if x < 1e-8 {
        return 1.0;
    }
    if x >= 3.0 {
        return 0.0;
    }
    let px = std::f64::consts::PI * x;
    3.0 * px.sin() * (px / 3.0).sin() / (px * px)
}

fn resize_nearest(src: &PixelBuffer, width: u32, height: u32) -> PixelBuffer {
    let mut out = PixelBuffer::new(width, height);
    let (sx, sy) = (f64::from(src.width()) / f64::from(width), f64::from(src.height()) / f64::from(height));
    let w = width as usize;
    out.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let yy = (((y as f64 + 0.5) * sy) as u32).min(src.height() - 1);
        for x in 0..w {
            let xx = (((x as f64 + 0.5) * sx) as u32).min(src.width() - 1);
            let i = src.index_of(xx, yy);
            row[x * 4..x * 4 + 4].copy_from_slice(&src.data()[i..i + 4]);
        }
    });
    out
}

/// For each output index: the first source index and the normalized filter weights.
fn build_weights(source: usize, target: usize, filter: (fn(f64) -> f64, f64)) -> Vec<(usize, Vec<f32>)> {
    let scale = source as f64 / target as f64;
    let stretch = scale.max(1.0); // the filter widens when shrinking
    let support = filter.1 * stretch;
    (0..target)
        .map(|i| {
            let center = (i as f64 + 0.5) * scale - 0.5;
            let mut start = ((center - support).floor() as i64 + 1).max(0) as usize;
            let mut end = (((center + support).ceil() as i64 - 1).min(source as i64 - 1)).max(-1);
            if end < start as i64 {
                let k = (center.round_ties_even() as i64).clamp(0, source as i64 - 1) as usize;
                start = k;
                end = k as i64;
            }
            let end = end as usize;
            let mut weights: Vec<f32> =
                (start..=end).map(|j| (filter.0((j as f64 - center) / stretch)) as f32).collect();
            let total: f64 = (start..=end).map(|j| filter.0((j as f64 - center) / stretch)).sum();
            if total.abs() < 1e-9 {
                let n = weights.len() as f32;
                weights.iter_mut().for_each(|w| *w = 1.0 / n);
            } else {
                for (k, j) in (start..=end).enumerate() {
                    weights[k] = (filter.0((j as f64 - center) / stretch) / total) as f32;
                }
            }
            (start, weights)
        })
        .collect()
}

/// Separable resize with a filter that widens when shrinking (no aliasing). Transparent pixels do not bleed their
/// hidden color (premultiplied).
fn resize_filtered(src: &PixelBuffer, width: u32, height: u32, mode: ResampleMode) -> PixelBuffer {
    let filter = kernel(mode);
    let (sw, sh, w, h) = (src.width() as usize, src.height() as usize, width as usize, height as usize);
    let mut pre = vec![0f32; sw * sh * 4];
    pre.par_chunks_mut(sw * 4).zip(src.data().par_chunks(sw * 4)).for_each(|(d, s)| {
        for (d, s) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)) {
            let a = f32::from(s[3]) / 255.0;
            d[0] = f32::from(s[0]) * a;
            d[1] = f32::from(s[1]) * a;
            d[2] = f32::from(s[2]) * a;
            d[3] = f32::from(s[3]);
        }
    });
    let xw = build_weights(sw, w, filter);
    let yw = build_weights(sh, h, filter);
    // Horizontal pass: sw x sh -> w x sh
    let mut tmp = vec![0f32; w * sh * 4];
    tmp.par_chunks_mut(w * 4).zip(pre.par_chunks(sw * 4)).for_each(|(out, row)| {
        for (x, (start, weights)) in xw.iter().enumerate() {
            let mut acc = [0f32; 4];
            for (k, wgt) in weights.iter().enumerate() {
                let i = (start + k) * 4;
                for c in 0..4 {
                    acc[c] += row[i + c] * wgt;
                }
            }
            out[x * 4..x * 4 + 4].copy_from_slice(&acc);
        }
    });
    // Vertical pass, then un-premultiply.
    let mut out = PixelBuffer::new(width, height);
    out.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let (start, weights) = &yw[y];
        for x in 0..w {
            let mut acc = [0f32; 4];
            for (k, wgt) in weights.iter().enumerate() {
                let i = ((start + k) * w + x) * 4;
                for c in 0..4 {
                    acc[c] += tmp[i + c] * wgt;
                }
            }
            let mut a = acc[3];
            if a < 0.5 {
                continue; // stays transparent
            }
            a = a.min(255.0); // sharp filters can overshoot
            let inv = 255.0 / a;
            let o = &mut row[x * 4..x * 4 + 4];
            o[0] = clamp_byte(f64::from(acc[0] * inv));
            o[1] = clamp_byte(f64::from(acc[1] * inv));
            o[2] = clamp_byte(f64::from(acc[2] * inv));
            o[3] = clamp_byte(f64::from(a));
        }
    });
    out
}

/// The bilinearly interpolated color at a fractional source position; outside the picture fades to transparent.
pub fn sample_bilinear(src: &PixelBuffer, sx: f64, sy: f64) -> [u8; 4] {
    let (x0, y0) = (sx.floor() as i64, sy.floor() as i64);
    let (fx, fy) = (sx - x0 as f64, sy - y0 as f64);
    let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
    let mut add = |x: i64, y: i64, weight: f64| {
        if weight <= 0.0 || x < 0 || y < 0 || x >= i64::from(src.width()) || y >= i64::from(src.height()) {
            return;
        }
        let i = src.index_of(x as u32, y as u32);
        let d = src.data();
        let pa = f64::from(d[i + 3]) * weight;
        r += f64::from(d[i]) * pa;
        g += f64::from(d[i + 1]) * pa;
        b += f64::from(d[i + 2]) * pa;
        a += pa;
    };
    add(x0, y0, (1.0 - fx) * (1.0 - fy));
    add(x0 + 1, y0, fx * (1.0 - fy));
    add(x0, y0 + 1, (1.0 - fx) * fy);
    add(x0 + 1, y0 + 1, fx * fy);
    if a <= 0.001 {
        return [0; 4];
    }
    [clamp_byte(r / a), clamp_byte(g / a), clamp_byte(b / a), clamp_byte(a)]
}
