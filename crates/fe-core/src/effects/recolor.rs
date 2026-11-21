//! Recolor (C# `RecolorEffect`): changes the color of an area (a dress...) to a chosen color while keeping its
//! shading. Every pixel keeps its light / dark difference from the area's average and its relative color strength;
//! only the hue is replaced and the overall tone is moved to the chosen color.

use super::adjust::map_pixels;
use super::{Effect, Values, no_padding, slider};
use crate::imaging::color_math::{clamp_byte, hsl_to_rgb, rgb_to_hsl};
use crate::imaging::{ColorRgba, PixelBuffer};
use crate::selection::Selection;

pub const RECOLOR: Effect = Effect {
    name: "Recolor",
    params: &[
        slider("hue", "Hue", 0.0, 360.0, 0.0, 1.0, "°"),
        slider("sat", "Saturation", 0.0, 100.0, 50.0, 1.0, "%"),
        slider("light", "Lightness", 0.0, 100.0, 50.0, 1.0, "%"),
        slider("shading", "Keep shading", 0.0, 100.0, 100.0, 1.0, "%"),
        slider("strength", "Strength", 0.0, 100.0, 100.0, 1.0, "%"),
    ],
    padding: no_padding,
    run: |src: &PixelBuffer, v: &Values| {
        let hue = v.get("hue");
        let target_s = v.get("sat") / 100.0;
        let target_l = (v.get("light") / 100.0).clamp(0.0, 1.0);
        let shading = v.get("shading") / 100.0;
        let strength = v.get("strength") / 100.0;
        let ref_l = v.get_or("refL", 0.5).clamp(0.01, 0.99);
        let ref_s = v.get_or("refS", 0.0).clamp(0.0, 1.0);
        map_pixels(src, |p| {
            let (r, g, b) = (f64::from(p[0]) / 255.0, f64::from(p[1]) / 255.0, f64::from(p[2]) / 255.0);
            let (_, s, l) = rgb_to_hsl(r, g, b);
            // Lightness: the average moves to the target, scaling toward black or white so shadows and
            // highlights stay in proportion.
            let moved = if target_l >= ref_l {
                l + (1.0 - l) * (target_l - ref_l) / (1.0 - ref_l)
            } else {
                l * target_l / ref_l
            };
            let new_l = target_l + (moved - target_l) * shading;
            // Saturation: each pixel keeps its strength relative to the area's average.
            let mut new_s = if ref_s < 0.04 { target_s } else { (target_s * s / ref_s).clamp(0.0, 1.0) };
            new_s = target_s + (new_s - target_s) * shading;
            let (nr, ng, nb) = hsl_to_rgb(hue, new_s.clamp(0.0, 1.0), new_l.clamp(0.0, 1.0));
            p[0] = clamp_byte((r + (nr - r) * strength) * 255.0);
            p[1] = clamp_byte((g + (ng - g) * strength) * 255.0);
            p[2] = clamp_byte((b + (nb - b) * strength) * 255.0);
        })
    },
};

/// The settings for turning an area whose average lightness / saturation are given into `target`.
pub fn recolor_values(target: ColorRgba, ref_lightness: f64, ref_saturation: f64) -> Values {
    let mut v = RECOLOR.defaults();
    let (h, s, l) = rgb_to_hsl(f64::from(target.r) / 255.0, f64::from(target.g) / 255.0, f64::from(target.b) / 255.0);
    v.set("hue", h.round_ties_even());
    v.set("sat", (s * 100.0).round_ties_even());
    v.set("light", (l * 100.0).round_ties_even());
    v.set("refL", ref_lightness);
    v.set("refS", ref_saturation);
    v
}

/// Average color, lightness and saturation of the pixels inside the selection (all visible pixels when there is
/// none), weighted by selection amount and opacity.
pub fn measure(image: &PixelBuffer, selection: Option<&Selection>) -> (ColorRgba, f64, f64) {
    let area = selection.map_or(image.bounds(), |s| s.bounds().intersect(&image.bounds()));
    let (mut w_sum, mut r_sum, mut g_sum, mut b_sum, mut l_sum, mut s_sum) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            let p = image.pixel(x as u32, y as u32);
            let w = f64::from(p.a) / 255.0 * selection.map_or(1.0, |s| s.coverage(x, y));
            if w <= 0.0 {
                continue;
            }
            let (r, g, b) = (f64::from(p.r) / 255.0, f64::from(p.g) / 255.0, f64::from(p.b) / 255.0);
            let (_, s, l) = rgb_to_hsl(r, g, b);
            w_sum += w;
            r_sum += r * w;
            g_sum += g * w;
            b_sum += b * w;
            l_sum += l * w;
            s_sum += s * w;
        }
    }
    if w_sum <= 0.0 {
        return (ColorRgba::BLACK, 0.5, 0.0);
    }
    let avg = ColorRgba::rgb(
        clamp_byte(r_sum / w_sum * 255.0),
        clamp_byte(g_sum / w_sum * 255.0),
        clamp_byte(b_sum / w_sum * 255.0),
    );
    (avg, l_sum / w_sum, s_sum / w_sum)
}
