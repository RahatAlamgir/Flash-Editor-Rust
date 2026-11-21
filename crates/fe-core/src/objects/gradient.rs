//! Gradient editing and the ready-made gradients (C# `GradientStyle` methods, `GradientPreset`).

use serde_json::{Map, Value};

use super::Gradient;
use super::json::{self, number};
use crate::imaging::ColorRgba;
use crate::imaging::color_math::{hsv_to_rgb, rgb_to_hsv};

impl Gradient {
    pub const MAX_STOPS: usize = 8;
    pub const MIN_STOPS: usize = 2;

    /// Colors spread evenly from start to end.
    pub fn evenly(colors: &[ColorRgba], angle: f64, radial: bool) -> Self {
        let last = (colors.len().max(2) - 1) as f64;
        Self { stops: colors.iter().enumerate().map(|(i, c)| (*c, i as f64 / last)).collect(), angle, radial }
    }

    /// A pleasant second color for `start`: the same kind of color with the hue turned a little and lighter
    /// (grey, white and black fade to a lighter or darker version).
    pub fn starting(start: ColorRgba) -> Self {
        let (h, s, v) = rgb_to_hsv(f64::from(start.r) / 255.0, f64::from(start.g) / 255.0, f64::from(start.b) / 255.0);
        if s < 0.08 {
            let other =
                if v > 0.5 { ColorRgba::new(80, 80, 80, start.a) } else { ColorRgba::new(220, 220, 220, start.a) };
            return Self::evenly(&[start, other], 90.0, false);
        }
        let (r, g, b) = hsv_to_rgb(h + 45.0, s.min(1.0), (v * 1.05 + 0.08).min(1.0));
        let byte = |x: f64| (x * 255.0).round().clamp(0.0, 255.0) as u8;
        Self::evenly(&[start, ColorRgba::new(byte(r), byte(g), byte(b), start.a)], 90.0, false)
    }

    /// The gradient running the other way.
    pub fn reversed(&self) -> Self {
        Self { stops: self.stops.iter().map(|(c, p)| (*c, 1.0 - p)).collect(), ..self.clone() }
    }

    /// The color of the gradient at a place along it (0..1), mixing the two stops around it.
    pub fn color_at(&self, position: f64) -> ColorRgba {
        let sorted = self.sorted();
        if position <= sorted[0].1 {
            return sorted[0].0;
        }
        for pair in sorted.windows(2) {
            let ((a, pa), (b, pb)) = (pair[0], pair[1]);
            if position > pb {
                continue;
            }
            let span = pb - pa;
            let t = if span <= 1e-9 { 1.0 } else { (position - pa) / span };
            let mix = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round().clamp(0.0, 255.0) as u8;
            return ColorRgba::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a));
        }
        sorted[sorted.len() - 1].0
    }

    /// One more color, put in the biggest gap between the existing ones.
    pub fn with_stop_added(&self) -> Self {
        if self.stops.len() >= Self::MAX_STOPS {
            return self.clone();
        }
        let mut edges = vec![0.0];
        edges.extend(self.sorted().iter().map(|s| s.1));
        edges.push(1.0);
        let (mut start, mut end, mut best) = (0.0, 1.0, -1.0);
        for pair in edges.windows(2) {
            if pair[1] - pair[0] > best {
                (start, end, best) = (pair[0], pair[1], pair[1] - pair[0]);
            }
        }
        let position = ((start + end) / 2.0 * 1000.0).round() / 1000.0;
        let mut g = self.clone();
        g.stops.push((self.color_at(position), position));
        g
    }

    /// Without stop `index` (unchanged when only two are left).
    pub fn with_stop_removed(&self, index: usize) -> Self {
        let mut g = self.clone();
        if g.stops.len() > Self::MIN_STOPS && index < g.stops.len() {
            g.stops.remove(index);
        }
        g
    }

    pub fn to_json(&self) -> Value {
        let stops = self
            .stops
            .iter()
            .map(|(c, p)| {
                let mut m = Map::new();
                m.insert("Color".into(), json::color_value(*c));
                m.insert("Position".into(), number(*p));
                Value::Object(m)
            })
            .collect();
        let mut m = Map::new();
        m.insert("Stops".into(), Value::Array(stops));
        m.insert("Angle".into(), number(self.angle));
        m.insert("Radial".into(), self.radial.into());
        Value::Object(m)
    }
}

/// A ready-made gradient (C# `GradientPreset`).
pub struct GradientPreset {
    pub name: &'static str,
    pub colors: &'static [ColorRgba],
    pub angle: f64,
    pub radial: bool,
}

impl GradientPreset {
    pub fn create(&self) -> Gradient {
        Gradient::evenly(self.colors, self.angle, self.radial)
    }
}

const fn c(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::new(r, g, b, 255)
}

const fn ca(r: u8, g: u8, b: u8, a: u8) -> ColorRgba {
    ColorRgba::new(r, g, b, a)
}

pub const GRADIENT_PRESETS: [GradientPreset; 11] = [
    GradientPreset { name: "Sunset", colors: &[c(255, 94, 98), c(255, 195, 113)], angle: 45.0, radial: false },
    GradientPreset { name: "Ocean", colors: &[c(0, 198, 251), c(0, 91, 234)], angle: 90.0, radial: false },
    GradientPreset { name: "Forest", colors: &[c(86, 171, 47), c(19, 78, 94)], angle: 90.0, radial: false },
    GradientPreset { name: "Candy", colors: &[c(255, 106, 213), c(147, 112, 255)], angle: 0.0, radial: false },
    GradientPreset { name: "Fire", colors: &[c(255, 214, 0), c(220, 20, 20)], angle: -90.0, radial: false },
    GradientPreset { name: "Night", colors: &[c(67, 67, 160), c(12, 12, 40)], angle: 90.0, radial: false },
    GradientPreset { name: "Gold", colors: &[c(255, 236, 160), c(191, 130, 20)], angle: 45.0, radial: false },
    GradientPreset { name: "Glow", colors: &[c(255, 255, 255), c(59, 130, 246)], angle: 0.0, radial: true },
    GradientPreset {
        name: "Rainbow",
        colors: &[c(239, 68, 68), c(250, 204, 21), c(34, 197, 94), c(59, 130, 246), c(168, 85, 247)],
        angle: 0.0,
        radial: false,
    },
    GradientPreset {
        name: "Aurora",
        colors: &[c(16, 185, 129), c(59, 130, 246), c(168, 85, 247)],
        angle: 60.0,
        radial: false,
    },
    GradientPreset {
        name: "Sunrise",
        colors: &[c(30, 27, 75), c(236, 72, 153), c(251, 191, 36)],
        angle: -90.0,
        radial: false,
    },
];

/// See-through gradients laid over pictures.
pub const OVERLAY_PRESETS: [GradientPreset; 6] = [
    GradientPreset {
        name: "Dark at the bottom",
        colors: &[ca(0, 0, 0, 0), ca(0, 0, 0, 190)],
        angle: 90.0,
        radial: false,
    },
    GradientPreset {
        name: "Dark at the top",
        colors: &[ca(0, 0, 0, 0), ca(0, 0, 0, 190)],
        angle: -90.0,
        radial: false,
    },
    GradientPreset {
        name: "Light at the bottom",
        colors: &[ca(255, 255, 255, 0), ca(255, 255, 255, 210)],
        angle: 90.0,
        radial: false,
    },
    GradientPreset { name: "Vignette", colors: &[ca(0, 0, 0, 0), ca(0, 0, 0, 170)], angle: 0.0, radial: true },
    GradientPreset {
        name: "Warm tint",
        colors: &[ca(255, 120, 60, 110), ca(255, 60, 140, 110)],
        angle: 45.0,
        radial: false,
    },
    GradientPreset {
        name: "Cool tint",
        colors: &[ca(0, 170, 255, 110), ca(120, 60, 255, 110)],
        angle: 45.0,
        radial: false,
    },
];
