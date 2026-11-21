//! Adjustment layers and per-object tone settings: brightness, contrast, saturation, hue, temperature,
//! tint, Levels and Curves. Same math as the C# `AdjustmentStyle`.

use rayon::prelude::*;

use super::curve::{self, CurvePoint};
use crate::imaging::PixelBuffer;
use crate::imaging::PixelRect;
use crate::imaging::color_math::{clamp_byte, hsv_to_rgb, rgb_to_hsv};

/// Which editor an adjustment layer shows (the math always applies every setting).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AdjustmentKind {
    #[default]
    Basic,
    Levels,
    Curves,
}

/// A color / tone change. All sliders at their default = no change.
#[derive(Clone, Debug, PartialEq)]
pub struct AdjustmentStyle {
    /// -100..100
    pub brightness: f64,
    /// -100..100
    pub contrast: f64,
    /// -100..100
    pub saturation: f64,
    /// Degrees.
    pub hue: f64,
    pub kind: AdjustmentKind,
    pub levels_black: f64,
    pub levels_white: f64,
    pub levels_gamma: f64,
    pub output_black: f64,
    pub output_white: f64,
    pub curve: Option<Vec<CurvePoint>>,
    pub curve_red: Option<Vec<CurvePoint>>,
    pub curve_green: Option<Vec<CurvePoint>>,
    pub curve_blue: Option<Vec<CurvePoint>>,
    /// -100 (cooler) .. 100 (warmer)
    pub temperature: f64,
    /// -100 (greener) .. 100 (more magenta)
    pub tint: f64,
}

impl Default for AdjustmentStyle {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 0.0,
            saturation: 0.0,
            hue: 0.0,
            kind: AdjustmentKind::Basic,
            levels_black: 0.0,
            levels_white: 255.0,
            levels_gamma: 1.0,
            output_black: 0.0,
            output_white: 255.0,
            curve: None,
            curve_red: None,
            curve_green: None,
            curve_blue: None,
            temperature: 0.0,
            tint: 0.0,
        }
    }
}

/// Lookup tables for Levels + Curves, built once per use.
#[derive(Clone, Debug)]
pub struct ToneTables {
    red: [u8; 256],
    green: [u8; 256],
    blue: [u8; 256],
}

impl AdjustmentStyle {
    /// A new Levels adjustment (changes nothing until edited).
    pub fn new_levels() -> Self {
        Self { kind: AdjustmentKind::Levels, ..Self::default() }
    }

    /// A new Curves adjustment (changes nothing until edited).
    pub fn new_curves() -> Self {
        Self { kind: AdjustmentKind::Curves, ..Self::default() }
    }

    fn levels_are_neutral(&self) -> bool {
        self.levels_black <= 0.0
            && self.levels_white >= 255.0
            && (self.levels_gamma - 1.0).abs() < 1e-9
            && self.output_black <= 0.0
            && self.output_white >= 255.0
    }

    fn curves_are_neutral(&self) -> bool {
        [&self.curve, &self.curve_red, &self.curve_green, &self.curve_blue]
            .iter()
            .all(|c| curve::is_straight(c.as_deref()))
    }

    /// True when applying it changes nothing.
    pub fn is_neutral(&self) -> bool {
        self.brightness == 0.0
            && self.contrast == 0.0
            && self.saturation == 0.0
            && self.hue == 0.0
            && self.temperature == 0.0
            && self.tint == 0.0
            && self.levels_are_neutral()
            && self.curves_are_neutral()
    }

    /// The Levels + Curves tables, or `None` when both are neutral.
    pub fn tone_tables(&self) -> Option<ToneTables> {
        if self.levels_are_neutral() && self.curves_are_neutral() {
            return None;
        }
        let levels = curve::levels_table(
            self.levels_black,
            self.levels_white,
            self.levels_gamma,
            self.output_black,
            self.output_white,
        );
        let master = curve::build_table(self.curve.as_deref());
        // Levels first, then the all-colors curve, then the channel's own curve.
        let channel = |own: &Option<Vec<CurvePoint>>| {
            let own = curve::build_table(own.as_deref());
            std::array::from_fn(|i| own[master[levels[i] as usize] as usize])
        };
        Some(ToneTables {
            red: channel(&self.curve_red),
            green: channel(&self.curve_green),
            blue: channel(&self.curve_blue),
        })
    }

    /// Changes one RGB(A) pixel in place. `strength` 0..1 mixes with the original (layer opacity).
    pub fn apply_pixel(&self, pixel: &mut [u8], strength: f64) {
        if self.is_neutral() || strength <= 0.0 {
            return;
        }
        self.apply_with(pixel, strength, self.tone_tables().as_ref());
    }

    /// Changes every visible pixel of `region` in `target`.
    pub fn apply_buffer(&self, target: &mut PixelBuffer, region: PixelRect, strength: f64) {
        let region = region.intersect(&target.bounds());
        if self.is_neutral() || strength <= 0.0 || region.is_empty() {
            return;
        }
        let tones = self.tone_tables();
        let stride = target.stride();
        let (x0, x1) = (region.x as usize * 4, region.right() as usize * 4);
        target
            .data_mut()
            .par_chunks_exact_mut(stride)
            .skip(region.y as usize)
            .take(region.height as usize)
            .for_each(|row| self.apply_row(&mut row[x0..x1], strength, tones.as_ref()));
    }

    /// Changes every visible pixel in a run of RGBA bytes (tables built by the caller).
    pub fn apply_row(&self, row: &mut [u8], strength: f64, tones: Option<&ToneTables>) {
        for p in row.as_chunks_mut::<4>().0.iter_mut() {
            if p[3] != 0 {
                self.apply_with(p, strength, tones);
            }
        }
    }

    fn apply_with(&self, pixel: &mut [u8], strength: f64, tones: Option<&ToneTables>) {
        let (or, og, ob) = (pixel[0] as f64 / 255.0, pixel[1] as f64 / 255.0, pixel[2] as f64 / 255.0);
        let (mut r, mut g, mut b) = (or, og, ob);
        if self.temperature != 0.0 || self.tint != 0.0 {
            let t = self.temperature / 100.0 * 40.0 / 255.0;
            let tint = self.tint / 100.0 * 30.0 / 255.0;
            r = (r + t + tint * 0.5).clamp(0.0, 1.0);
            g = (g - tint).clamp(0.0, 1.0);
            b = (b - t + tint * 0.5).clamp(0.0, 1.0);
        }
        if self.hue != 0.0 || self.saturation != 0.0 {
            let (mut h, mut s, v) = rgb_to_hsv(r, g, b);
            h += self.hue;
            if self.saturation != 0.0 {
                s = if self.saturation > 0.0 {
                    s + (1.0 - s) * self.saturation / 100.0 * (s * 2.0 + 0.2).min(1.0)
                } else {
                    s * (1.0 + self.saturation / 100.0)
                }
                .clamp(0.0, 1.0);
            }
            (r, g, b) = hsv_to_rgb(h, s, v);
        }
        if self.contrast != 0.0 {
            // Up to 2.5 times steeper, down to flat grey.
            let k = if self.contrast > 0.0 { 1.0 + self.contrast / 100.0 * 1.5 } else { 1.0 + self.contrast / 100.0 };
            r = (r - 0.5) * k + 0.5;
            g = (g - 0.5) * k + 0.5;
            b = (b - 0.5) * k + 0.5;
        }
        if self.brightness != 0.0 {
            let add = self.brightness / 100.0 * 0.5;
            r += add;
            g += add;
            b += add;
        }
        if let Some(t) = tones {
            r = t.red[clamp_byte(r * 255.0) as usize] as f64 / 255.0;
            g = t.green[clamp_byte(g * 255.0) as usize] as f64 / 255.0;
            b = t.blue[clamp_byte(b * 255.0) as usize] as f64 / 255.0;
        }
        pixel[0] = clamp_byte((or + (r - or) * strength) * 255.0);
        pixel[1] = clamp_byte((og + (g - og) * strength) * 255.0);
        pixel[2] = clamp_byte((ob + (b - ob) * strength) * 255.0);
    }
}
