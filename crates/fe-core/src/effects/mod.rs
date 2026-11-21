//! Effect framework plus adjustments and filters (C# `ImageEffect`, `EffectParameter`, `EffectValues`,
//! `EffectApplier`). An effect takes a source picture and returns a new, same-sized one; it never changes the
//! source. Pictures are straight-alpha RGBA; alpha is never touched by color adjustments.

#![allow(clippy::chunks_exact_to_as_chunks)] // the loops below read well as plain 4-byte chunks

mod adjust;
mod apply;
mod blur;
mod enhance;
mod filters;
mod recolor;

use std::collections::HashMap;

use crate::imaging::PixelBuffer;

pub use apply::{apply_to_selection, padding_for};
pub use blur::{gaussian, gaussian_in_place};
pub use recolor::{measure, recolor_values};

/// One setting of an effect. The dialog is built from these, so a new effect never needs new dialog code.
#[derive(Clone, Copy, Debug)]
pub struct Param {
    pub key: &'static str,
    pub label: &'static str,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step: f64,
    pub unit: &'static str,
    /// An on / off box (the value is 0 or 1).
    pub toggle: bool,
}

pub(crate) const fn slider(
    key: &'static str,
    label: &'static str,
    min: f64,
    max: f64,
    default: f64,
    step: f64,
    unit: &'static str,
) -> Param {
    Param { key, label, min, max, default, step, unit, toggle: false }
}

pub(crate) const fn toggle(key: &'static str, label: &'static str, on: bool) -> Param {
    Param { key, label, min: 0.0, max: 1.0, default: if on { 1.0 } else { 0.0 }, step: 1.0, unit: "", toggle: true }
}

/// The chosen value for each setting, by key.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Values(HashMap<&'static str, f64>);

impl Values {
    pub fn defaults(params: &[Param]) -> Self {
        Self(params.iter().map(|p| (p.key, p.default)).collect())
    }

    pub fn get(&self, key: &str) -> f64 {
        self.0.get(key).copied().unwrap_or(0.0)
    }

    pub fn get_or(&self, key: &str, fallback: f64) -> f64 {
        self.0.get(key).copied().unwrap_or(fallback)
    }

    pub fn on(&self, key: &str) -> bool {
        self.get(key) >= 0.5
    }

    pub fn set(&mut self, key: &'static str, value: f64) {
        self.0.insert(key, value);
    }
}

/// An adjustment or a filter.
pub struct Effect {
    /// Menu / dialog / history name.
    pub name: &'static str,
    /// Settings shown in the dialog. Empty = applied at once.
    pub params: &'static [Param],
    /// How many pixels outside a selection the effect reads (a blur radius...).
    pub padding: fn(&Values) -> i32,
    pub run: fn(&PixelBuffer, &Values) -> PixelBuffer,
}

impl Effect {
    pub fn defaults(&self) -> Values {
        Values::defaults(self.params)
    }

    pub fn apply(&self, source: &PixelBuffer, values: &Values) -> PixelBuffer {
        (self.run)(source, values)
    }
}

pub(crate) fn no_padding(_: &Values) -> i32 {
    0
}

pub use adjust::{
    AUTO_LEVELS, BRIGHTNESS_CONTRAST, EXPOSURE_GAMMA, GRAYSCALE, HUE_SATURATION, INVERT, POSTERIZE, SEPIA,
    TEMPERATURE_TINT, THRESHOLD, VIBRANCE,
};
pub use enhance::ENHANCE;
pub use filters::{
    ADD_NOISE, EMBOSS, FIND_EDGES, GAUSSIAN_BLUR, MOTION_BLUR, PIXELATE, REDUCE_NOISE, SHARPEN, VIGNETTE,
};
pub use recolor::RECOLOR;

/// The adjustments, in the order of the Adjust tab.
pub fn adjustments() -> [&'static Effect; 12] {
    [
        &AUTO_LEVELS,
        &BRIGHTNESS_CONTRAST,
        &EXPOSURE_GAMMA,
        &HUE_SATURATION,
        &VIBRANCE,
        &TEMPERATURE_TINT,
        &GRAYSCALE,
        &SEPIA,
        &INVERT,
        &THRESHOLD,
        &POSTERIZE,
        &RECOLOR,
    ]
}

/// The filters, in the order of the Filter tab.
pub fn filters() -> [&'static Effect; 10] {
    [
        &GAUSSIAN_BLUR,
        &MOTION_BLUR,
        &SHARPEN,
        &ENHANCE,
        &ADD_NOISE,
        &REDUCE_NOISE,
        &PIXELATE,
        &VIGNETTE,
        &EMBOSS,
        &FIND_EDGES,
    ]
}
