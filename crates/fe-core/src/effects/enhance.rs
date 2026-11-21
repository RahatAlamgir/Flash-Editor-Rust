//! "Make a blurry picture cleaner" (C# `EnhanceEffect`): a little noise reduction, sharpening of fine detail
//! (ignoring tiny differences, which are noise) and clarity (mid-size contrast). It cannot invent detail that is
//! completely gone.

use rayon::prelude::*;

use super::blur::gaussian;
use super::filters::reduce_noise;
use super::{Effect, slider};
use crate::imaging::color_math::clamp_byte;

const CLARITY_RADIUS: f64 = 20.0;

pub const ENHANCE: Effect = Effect {
    name: "Enhance (Clean Up Blur)",
    params: &[
        slider("sharpen", "Sharpen", 0.0, 300.0, 120.0, 1.0, "%"),
        slider("radius", "Detail size", 0.5, 5.0, 1.5, 0.1, "px"),
        slider("clarity", "Clarity", 0.0, 100.0, 30.0, 1.0, "%"),
        slider("denoise", "Noise reduction", 0.0, 100.0, 30.0, 1.0, "%"),
        slider("threshold", "Ignore tiny details", 0.0, 30.0, 4.0, 1.0, ""),
    ],
    padding: |v| (v.get("radius").max(CLARITY_RADIUS) * 3.0).ceil() as i32 + 1,
    run: |src, v| {
        let sharpen = v.get("sharpen") / 100.0;
        let clarity = v.get("clarity") / 100.0 * 0.6;
        let threshold = v.get("threshold");
        // 1) Noise first, so sharpening does not make grain worse.
        let clean = if v.get("denoise") > 0.0 { reduce_noise(src, 1, v.get("denoise") / 100.0) } else { src.clone() };
        // 2) Fine detail (unsharp mask with a threshold) and 3) clarity (large-radius local contrast).
        let fine = if sharpen > 0.0 { gaussian(&clean, v.get("radius")) } else { clean.clone() };
        let broad = if clarity > 0.0 { gaussian(&clean, CLARITY_RADIUS) } else { clean.clone() };
        let mut out = clean.clone();
        let stride = out.stride();
        out.data_mut()
            .par_chunks_mut(stride)
            .zip(clean.data().par_chunks(stride))
            .zip(fine.data().par_chunks(stride))
            .zip(broad.data().par_chunks(stride))
            .for_each(|(((d, c), f), g)| {
                for (((d, c), f), g) in
                    d.chunks_exact_mut(4).zip(c.chunks_exact(4)).zip(f.chunks_exact(4)).zip(g.chunks_exact(4))
                {
                    if c[3] == 0 {
                        continue;
                    }
                    for k in 0..3 {
                        let mut value = f64::from(c[k]);
                        let detail = value - f64::from(f[k]);
                        let magnitude = detail.abs();
                        // Differences smaller than the threshold are treated as noise.
                        if magnitude > threshold {
                            value += detail.signum() * (magnitude - threshold) * sharpen;
                        }
                        value += (f64::from(c[k]) - f64::from(g[k])) * clarity;
                        d[k] = clamp_byte(value);
                    }
                }
            });
        out
    },
};
