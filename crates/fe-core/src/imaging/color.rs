//! One RGBA color with straight alpha.

use std::fmt::Write as _;

/// An 8-bit RGBA color (alpha 255 = fully opaque).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ColorRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgba {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const TRANSPARENT: Self = Self::new(0, 0, 0, 0);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// An opaque color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub const fn to_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const fn from_slice(p: &[u8]) -> Self {
        Self::new(p[0], p[1], p[2], p[3])
    }

    /// `#RRGGBB`, or `#RRGGBBAA` when not fully opaque.
    pub fn to_hex(self) -> String {
        let mut s = format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        if self.a != 255 {
            let _ = write!(s, "{:02X}", self.a);
        }
        s
    }

    /// Reads `#RGB`, `#RRGGBB` or `#RRGGBBAA` (the `#` is optional).
    pub fn parse_hex(text: &str) -> Option<Self> {
        let hex = text.trim().trim_start_matches('#');
        let hex: String = match hex.len() {
            3 => hex.chars().flat_map(|c| [c, c]).collect(),
            6 | 8 => hex.to_string(),
            _ => return None,
        };
        let v = u32::from_str_radix(&hex, 16).ok()?;
        Some(if hex.len() == 6 {
            Self::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
        } else {
            Self::new((v >> 24) as u8, (v >> 16) as u8, (v >> 8) as u8, v as u8)
        })
    }
}
