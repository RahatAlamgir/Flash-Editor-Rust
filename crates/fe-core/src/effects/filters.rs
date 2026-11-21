//! Filters (C# `Effects/Filters`): blur, motion blur, sharpen, noise, pixelate, vignette, emboss, find edges.

use rayon::prelude::*;

use super::blur::gaussian;
use super::{Effect, Values, no_padding, slider, toggle};
use crate::imaging::PixelBuffer;
use crate::imaging::color_math::clamp_byte;

pub const GAUSSIAN_BLUR: Effect = Effect {
    name: "Gaussian Blur",
    params: &[slider("radius", "Radius", 0.5, 100.0, 4.0, 0.5, "px")],
    padding: |v| (v.get("radius") * 3.0).ceil() as i32,
    run: |s, v| gaussian(s, v.get("radius")),
};

pub const MOTION_BLUR: Effect = Effect {
    name: "Motion Blur",
    params: &[
        slider("angle", "Angle", -180.0, 180.0, 0.0, 1.0, "°"),
        slider("distance", "Distance", 1.0, 200.0, 20.0, 1.0, "px"),
    ],
    padding: |v| (v.get("distance") / 2.0).ceil() as i32 + 1,
    run: motion_blur,
};

fn motion_blur(src: &PixelBuffer, v: &Values) -> PixelBuffer {
    let angle = v.get("angle").to_radians();
    let distance = v.get("distance");
    let samples = (distance.ceil() as i64).max(2);
    let step_x = angle.cos() * distance / (samples - 1) as f64;
    let step_y = -angle.sin() * distance / (samples - 1) as f64;
    let (w, h) = (src.width() as i64, src.height() as i64);
    let s = src.data();
    let mut out = PixelBuffer::new(src.width(), src.height());
    out.data_mut().par_chunks_mut(w as usize * 4).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let (mut r, mut g, mut b, mut a) = (0.0, 0.0, 0.0, 0.0);
            for k in 0..samples {
                let t = k as f64 - (samples - 1) as f64 / 2.0;
                let sx = ((x as f64 + t * step_x).round_ties_even() as i64).clamp(0, w - 1);
                let sy = ((y as f64 + t * step_y).round_ties_even() as i64).clamp(0, h - 1);
                let i = ((sy * w + sx) * 4) as usize;
                let pa = f64::from(s[i + 3]);
                r += f64::from(s[i]) * pa;
                g += f64::from(s[i + 1]) * pa;
                b += f64::from(s[i + 2]) * pa;
                a += pa;
            }
            if a <= 0.0 {
                continue;
            }
            let o = x as usize * 4;
            row[o] = clamp_byte(r / a);
            row[o + 1] = clamp_byte(g / a);
            row[o + 2] = clamp_byte(b / a);
            row[o + 3] = clamp_byte(a / samples as f64);
        }
    });
    out
}

pub const SHARPEN: Effect = Effect {
    name: "Sharpen",
    params: &[
        slider("amount", "Amount", 0.0, 500.0, 100.0, 1.0, "%"),
        slider("radius", "Radius", 0.5, 10.0, 1.0, 0.1, "px"),
    ],
    padding: |v| (v.get("radius") * 3.0).ceil() as i32,
    run: |src, v| {
        let amount = v.get("amount") / 100.0;
        let blurred = gaussian(src, v.get("radius"));
        let mut out = src.clone();
        let stride = src.stride();
        out.data_mut()
            .par_chunks_mut(stride)
            .zip(src.data().par_chunks(stride))
            .zip(blurred.data().par_chunks(stride))
            .for_each(|((d, s), b)| {
                for ((d, s), b) in d.chunks_exact_mut(4).zip(s.chunks_exact(4)).zip(b.chunks_exact(4)) {
                    if s[3] == 0 {
                        continue;
                    }
                    for c in 0..3 {
                        d[c] = clamp_byte(f64::from(s[c]) + (f64::from(s[c]) - f64::from(b[c])) * amount);
                    }
                }
            });
        out
    },
};

/// The brightness (0..255) of every pixel's 3x3 neighborhood is given to `compute` (edges are repeated); the result
/// is the gray the pixel becomes. Alpha is kept.
fn map_luma_3x3(src: &PixelBuffer, compute: impl Fn(&[f64; 9]) -> u8 + Sync) -> PixelBuffer {
    let (w, h) = (src.width() as usize, src.height() as usize);
    let luma: Vec<f32> = src
        .data()
        .chunks_exact(4)
        .map(|p| (0.114 * f64::from(p[2]) + 0.587 * f64::from(p[1]) + 0.299 * f64::from(p[0])) as f32)
        .collect();
    let mut out = src.clone();
    out.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
        let mut n = [0f64; 9];
        for x in 0..w {
            let mut k = 0;
            for dy in -1i64..=1 {
                let yy = (y as i64 + dy).clamp(0, h as i64 - 1) as usize * w;
                for dx in -1i64..=1 {
                    n[k] = f64::from(luma[yy + (x as i64 + dx).clamp(0, w as i64 - 1) as usize]);
                    k += 1;
                }
            }
            let g = compute(&n);
            row[x * 4..x * 4 + 3].fill(g);
        }
    });
    out
}

pub const EMBOSS: Effect = Effect {
    name: "Emboss",
    params: &[slider("strength", "Strength", 0.2, 5.0, 1.5, 0.1, ""), toggle("color", "Keep original colors", false)],
    padding: |_| 1,
    run: |src, v| {
        let strength = v.get("strength");
        let embossed =
            map_luma_3x3(src, |n| clamp_byte(128.0 + strength * (n[8] + n[5] + n[7] - n[0] - n[1] - n[3]) / 3.0));
        if !v.on("color") {
            return embossed;
        }
        // The relief goes over the original colors.
        let mut out = src.clone();
        for (d, e) in out.data_mut().chunks_exact_mut(4).zip(embossed.data().chunks_exact(4)) {
            let shift = f64::from(e[0]) - 128.0;
            for c in &mut d[..3] {
                *c = clamp_byte(f64::from(*c) + shift);
            }
        }
        out
    },
};

pub const FIND_EDGES: Effect = Effect {
    name: "Find Edges",
    params: &[
        slider("strength", "Strength", 0.2, 4.0, 1.0, 0.1, ""),
        toggle("sketch", "Dark lines on white (sketch)", true),
    ],
    padding: |_| 1,
    run: |src, v| {
        let (strength, sketch) = (v.get("strength"), v.on("sketch"));
        map_luma_3x3(src, |n| {
            let gx = (n[2] + 2.0 * n[5] + n[8]) - (n[0] + 2.0 * n[3] + n[6]);
            let gy = (n[6] + 2.0 * n[7] + n[8]) - (n[0] + 2.0 * n[1] + n[2]);
            let magnitude = ((gx * gx + gy * gy).sqrt() * strength / 2.0).min(255.0);
            clamp_byte(if sketch { 255.0 - magnitude } else { magnitude })
        })
    },
};

pub const VIGNETTE: Effect = Effect {
    name: "Vignette",
    params: &[
        slider("amount", "Amount (- lighten / + darken)", -100.0, 100.0, 50.0, 1.0, ""),
        slider("size", "Clear center size", 0.0, 100.0, 50.0, 1.0, "%"),
        slider("softness", "Softness", 1.0, 100.0, 60.0, 1.0, "%"),
    ],
    padding: no_padding,
    run: |src, v| {
        let amount = v.get("amount") / 100.0;
        let inner = v.get("size") / 100.0 * 0.9;
        let softness = (v.get("softness") / 100.0).max(0.01);
        let (w, h) = (src.width() as usize, src.height() as usize);
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        let mut out = src.clone();
        out.data_mut().par_chunks_mut(w * 4).enumerate().for_each(|(y, row)| {
            let ny = (y as f64 + 0.5 - cy) / cy;
            for (x, p) in row.chunks_exact_mut(4).enumerate() {
                if p[3] == 0 {
                    continue;
                }
                let nx = (x as f64 + 0.5 - cx) / cx;
                let distance = ((nx * nx + ny * ny) / 2.0).sqrt(); // 0 center .. 1 corner
                let t = ((distance - inner) / softness).clamp(0.0, 1.0);
                let k = t * t * (3.0 - 2.0 * t) * amount; // smooth step
                for c in &mut p[..3] {
                    let v = f64::from(*c);
                    *c = clamp_byte(if k >= 0.0 { v * (1.0 - k) } else { v + (255.0 - v) * -k });
                }
            }
        });
        out
    },
};

pub const PIXELATE: Effect = Effect {
    name: "Pixelate",
    params: &[slider("size", "Cell size", 2.0, 200.0, 10.0, 1.0, "px")],
    padding: |v| v.get("size") as i32,
    run: |src, v| {
        let cell = (v.get("size") as usize).max(1);
        let (w, h) = (src.width() as usize, src.height() as usize);
        let s = src.data();
        let mut out = PixelBuffer::new(src.width(), src.height());
        out.data_mut().par_chunks_mut(w * 4 * cell).enumerate().for_each(|(cy, band)| {
            let y0 = cy * cell;
            let y1 = (y0 + cell).min(h);
            for x0 in (0..w).step_by(cell) {
                let x1 = (x0 + cell).min(w);
                let (mut r, mut g, mut b, mut a, mut count) = (0.0, 0.0, 0.0, 0.0, 0usize);
                for y in y0..y1 {
                    for x in x0..x1 {
                        let i = (y * w + x) * 4;
                        let pa = f64::from(s[i + 3]);
                        r += f64::from(s[i]) * pa;
                        g += f64::from(s[i + 1]) * pa;
                        b += f64::from(s[i + 2]) * pa;
                        a += pa;
                        count += 1;
                    }
                }
                let px = if a > 0.0 {
                    [clamp_byte(r / a), clamp_byte(g / a), clamp_byte(b / a), clamp_byte(a / count as f64)]
                } else {
                    [0; 4]
                };
                for y in y0..y1 {
                    for x in x0..x1 {
                        let o = ((y - y0) * w + x) * 4;
                        band[o..o + 4].copy_from_slice(&px);
                    }
                }
            }
        });
        out
    },
};

/// A small fixed-seed random generator (the grain must not flicker while a preview is redrawn).
struct Rng(u64);

impl Rng {
    fn next_f64(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Normally distributed (Box-Muller).
    fn gaussian(&mut self) -> f64 {
        let u1 = 1.0 - self.next_f64();
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }
}

pub const ADD_NOISE: Effect = Effect {
    name: "Add Noise",
    params: &[slider("amount", "Amount", 0.0, 100.0, 20.0, 1.0, "%"), toggle("mono", "Monochromatic", true)],
    padding: no_padding,
    run: |src, v| {
        let sigma = v.get("amount") * 0.8;
        let mono = v.on("mono");
        let mut out = src.clone();
        let stride = out.stride();
        out.data_mut().par_chunks_mut(stride).enumerate().for_each(|(y, row)| {
            let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (y as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9));
            for p in row.chunks_exact_mut(4) {
                if p[3] == 0 {
                    continue;
                }
                if mono {
                    let n = rng.gaussian() * sigma;
                    for c in &mut p[..3] {
                        *c = clamp_byte(f64::from(*c) + n);
                    }
                } else {
                    for c in &mut p[..3] {
                        *c = clamp_byte(f64::from(*c) + rng.gaussian() * sigma);
                    }
                }
            }
        });
        out
    },
};

pub const REDUCE_NOISE: Effect = Effect {
    name: "Reduce Noise",
    params: &[
        slider("radius", "Radius", 1.0, 3.0, 1.0, 1.0, "px"),
        slider("strength", "Strength", 0.0, 100.0, 100.0, 1.0, "%"),
    ],
    padding: |v| v.get("radius") as i32,
    run: |src, v| reduce_noise(src, (v.get("radius") as i64).clamp(1, 3), v.get("strength") / 100.0),
};

/// Median filter: each pixel becomes the middle value of its neighbors.
pub(super) fn reduce_noise(src: &PixelBuffer, radius: i64, strength: f64) -> PixelBuffer {
    let (w, h) = (src.width() as i64, src.height() as i64);
    let size = ((2 * radius + 1) * (2 * radius + 1)) as usize;
    let s = src.data();
    let mut out = src.clone();
    out.data_mut().par_chunks_mut(w as usize * 4).enumerate().for_each(|(y, row)| {
        let mut window = vec![0u8; size];
        for x in 0..w {
            let i = ((y as i64 * w + x) * 4) as usize;
            if s[i + 3] == 0 {
                continue;
            }
            for c in 0..3 {
                let mut n = 0;
                for dy in -radius..=radius {
                    let r = (y as i64 + dy).clamp(0, h - 1) * w;
                    for dx in -radius..=radius {
                        window[n] = s[((r + (x + dx).clamp(0, w - 1)) * 4) as usize + c];
                        n += 1;
                    }
                }
                window.sort_unstable();
                let median = f64::from(window[size / 2]);
                let own = f64::from(s[i + c]);
                row[x as usize * 4 + c] = clamp_byte(own + (median - own) * strength);
            }
        }
    });
    out
}
