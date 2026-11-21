//! Color and tone adjustments (C# `Effects/Adjustments`). Pixels that are fully see-through stay as they are.

use rayon::prelude::*;

use super::{Effect, Values, no_padding, slider, toggle};
use crate::imaging::PixelBuffer;
use crate::imaging::color_math::{clamp_byte, hsl_to_rgb, rgb_to_hsl};

/// Runs `f` on the color channels of every visible pixel, rows in parallel.
pub(super) fn map_pixels(src: &PixelBuffer, f: impl Fn(&mut [u8]) + Sync) -> PixelBuffer {
    let mut out = src.clone();
    let stride = out.stride();
    out.data_mut().par_chunks_mut(stride).for_each(|row| {
        for p in row.chunks_exact_mut(4) {
            if p[3] != 0 {
                f(&mut p[..3]);
            }
        }
    });
    out
}

fn lut(f: impl Fn(f64) -> f64) -> [u8; 256] {
    let mut t = [0u8; 256];
    for (i, v) in t.iter_mut().enumerate() {
        *v = clamp_byte(f(i as f64));
    }
    t
}

fn apply_lut(src: &PixelBuffer, l: &[u8; 256]) -> PixelBuffer {
    map_pixels(src, |p| {
        for c in p {
            *c = l[*c as usize];
        }
    })
}

fn luma(p: &[u8]) -> f64 {
    0.299 * f64::from(p[0]) + 0.587 * f64::from(p[1]) + 0.114 * f64::from(p[2])
}

pub const INVERT: Effect = Effect {
    name: "Invert",
    params: &[],
    padding: no_padding,
    run: |s, _| {
        map_pixels(s, |p| {
            for c in p {
                *c = 255 - *c;
            }
        })
    },
};

pub const GRAYSCALE: Effect = Effect {
    name: "Grayscale",
    params: &[],
    padding: no_padding,
    run: |s, _| {
        map_pixels(s, |p| {
            let g = clamp_byte(luma(p));
            p.fill(g);
        })
    },
};

pub const SEPIA: Effect = Effect {
    name: "Sepia",
    params: &[],
    padding: no_padding,
    run: |s, _| {
        map_pixels(s, |p| {
            let (r, g, b) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            p[0] = clamp_byte(0.393 * r + 0.769 * g + 0.189 * b);
            p[1] = clamp_byte(0.349 * r + 0.686 * g + 0.168 * b);
            p[2] = clamp_byte(0.272 * r + 0.534 * g + 0.131 * b);
        })
    },
};

pub const THRESHOLD: Effect = Effect {
    name: "Threshold",
    params: &[slider("level", "Level", 1.0, 255.0, 128.0, 1.0, "")],
    padding: no_padding,
    run: |s, v| {
        let level = v.get("level");
        map_pixels(s, |p| p.fill(if luma(p) >= level { 255 } else { 0 }))
    },
};

pub const POSTERIZE: Effect = Effect {
    name: "Posterize",
    params: &[slider("levels", "Levels", 2.0, 32.0, 5.0, 1.0, "")],
    padding: no_padding,
    run: |s, v| {
        let levels = v.get("levels").max(2.0).floor();
        let step = 255.0 / (levels - 1.0);
        apply_lut(s, &lut(|x| (x / step).round_ties_even() * step))
    },
};

pub const EXPOSURE_GAMMA: Effect = Effect {
    name: "Exposure / Gamma",
    params: &[
        slider("exposure", "Exposure", -3.0, 3.0, 0.0, 0.05, "EV"),
        slider("gamma", "Gamma (midtones)", 0.2, 3.0, 1.0, 0.01, ""),
    ],
    padding: no_padding,
    run: |s, v| {
        let gain = 2f64.powf(v.get("exposure"));
        let inv = 1.0 / v.get("gamma").max(0.01);
        apply_lut(s, &lut(|x| 255.0 * (x / 255.0 * gain).clamp(0.0, 1.0).powf(inv)))
    },
};

pub const BRIGHTNESS_CONTRAST: Effect = Effect {
    name: "Brightness / Contrast",
    params: &[
        slider("brightness", "Brightness", -100.0, 100.0, 0.0, 1.0, ""),
        slider("contrast", "Contrast", -100.0, 100.0, 0.0, 1.0, ""),
    ],
    padding: no_padding,
    run: |s, v| {
        let brightness = v.get("brightness") * 1.5; // a shift of up to 150 levels
        let c = v.get("contrast") * 2.55;
        let factor = 259.0 * (c + 255.0) / (255.0 * (259.0 - c)); // the classic contrast curve
        apply_lut(s, &lut(|x| (x + brightness - 128.0) * factor + 128.0))
    },
};

pub const TEMPERATURE_TINT: Effect = Effect {
    name: "Temperature / Tint",
    params: &[
        slider("temperature", "Temperature (cool / warm)", -100.0, 100.0, 0.0, 1.0, ""),
        slider("tint", "Tint (green / magenta)", -100.0, 100.0, 0.0, 1.0, ""),
    ],
    padding: no_padding,
    run: |s, v| {
        let t = v.get("temperature") / 100.0 * 40.0; // up to 40 levels
        let tint = v.get("tint") / 100.0 * 30.0;
        let (red, green, blue) = (lut(|x| x + t + tint * 0.5), lut(|x| x - tint), lut(|x| x - t + tint * 0.5));
        map_pixels(s, |p| {
            p[0] = red[p[0] as usize];
            p[1] = green[p[1] as usize];
            p[2] = blue[p[2] as usize];
        })
    },
};

pub const VIBRANCE: Effect = Effect {
    name: "Vibrance",
    params: &[slider("vibrance", "Vibrance", -100.0, 100.0, 40.0, 1.0, "")],
    padding: no_padding,
    run: |s, v| {
        let amount = v.get("vibrance") / 100.0;
        map_pixels(s, |p| {
            let max = p[0].max(p[1]).max(p[2]);
            let min = p[0].min(p[1]).min(p[2]);
            let saturation = if max == 0 { 0.0 } else { f64::from(max - min) / f64::from(max) };
            // Strong boost for dull pixels, little for saturated ones.
            let scale = 1.0 + amount * (1.0 - saturation) * if amount > 0.0 { 1.5 } else { 1.0 };
            let gray = luma(p);
            for c in p.iter_mut() {
                *c = clamp_byte(gray + (f64::from(*c) - gray) * scale);
            }
        })
    },
};

pub const HUE_SATURATION: Effect = Effect {
    name: "Hue / Saturation",
    params: &[
        slider("hue", "Hue", -180.0, 180.0, 0.0, 1.0, "°"),
        slider("saturation", "Saturation", -100.0, 100.0, 0.0, 1.0, ""),
        slider("lightness", "Lightness", -100.0, 100.0, 0.0, 1.0, ""),
    ],
    padding: no_padding,
    run: |s, v| {
        let (hue, sat, light) = (v.get("hue"), v.get("saturation") / 100.0, v.get("lightness") / 100.0);
        map_pixels(s, |p| {
            let (mut h, mut sa, mut l) =
                rgb_to_hsl(f64::from(p[0]) / 255.0, f64::from(p[1]) / 255.0, f64::from(p[2]) / 255.0);
            h += hue;
            // Positive saturation pushes towards fully saturated, negative towards gray.
            sa = if sat >= 0.0 { sa + (1.0 - sa) * sat * sa } else { sa * (1.0 + sat) };
            l = if light >= 0.0 { l + (1.0 - l) * light } else { l * (1.0 + light) };
            let (r, g, b) = hsl_to_rgb(h, sa.clamp(0.0, 1.0), l.clamp(0.0, 1.0));
            p[0] = clamp_byte(r * 255.0);
            p[1] = clamp_byte(g * 255.0);
            p[2] = clamp_byte(b * 255.0);
        })
    },
};

pub const AUTO_LEVELS: Effect = Effect {
    name: "Auto Levels",
    params: &[
        slider("strength", "Strength", 0.0, 100.0, 100.0, 1.0, "%"),
        toggle("perChannel", "Also fix color cast (each channel separately)", true),
    ],
    padding: no_padding,
    run: auto_levels,
};

fn auto_levels(src: &PixelBuffer, v: &Values) -> PixelBuffer {
    let strength = v.get("strength") / 100.0;
    let mut hist = [[0u64; 256]; 3];
    let mut count = 0u64;
    for p in src.data().chunks_exact(4) {
        if p[3] != 0 {
            for c in 0..3 {
                hist[c][p[c] as usize] += 1;
            }
            count += 1;
        }
    }
    if count == 0 {
        return src.clone();
    }
    let clip = (count as f64 * 0.005) as u64; // the extreme 0.5 % is ignored
    let range = |c: usize| {
        let (mut low, mut sum) = (0usize, 0u64);
        while low < 255 {
            sum += hist[c][low];
            if sum > clip {
                break;
            }
            low += 1;
        }
        let (mut high, mut sum) = (255usize, 0u64);
        while high > 0 {
            sum += hist[c][high];
            if sum > clip {
                break;
            }
            high -= 1;
        }
        if high <= low { (0usize, 255usize) } else { (low, high) }
    };
    let mut ranges = [range(0), range(1), range(2)];
    if !v.on("perChannel") {
        let low = ranges.iter().map(|r| r.0).min().unwrap_or(0);
        let high = ranges.iter().map(|r| r.1).max().unwrap_or(255);
        ranges = [(low, high); 3];
    }
    let luts: Vec<[u8; 256]> = ranges
        .iter()
        .map(|(low, high)| {
            let span = (*high as f64 - *low as f64).max(1.0);
            lut(|x| {
                let stretched = (x - *low as f64) * 255.0 / span;
                x + (stretched - x) * strength
            })
        })
        .collect();
    map_pixels(src, |p| {
        for c in 0..3 {
            p[c] = luts[c][p[c] as usize];
        }
    })
}
