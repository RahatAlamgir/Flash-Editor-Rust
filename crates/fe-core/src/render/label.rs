//! A single line of text for charts and barcodes: measured and drawn as an outline, like the C# `FormattedText`.

use tiny_skia::Path;

use crate::imaging::ColorRgba;
use crate::text::{TextData, lay_out};

/// One line of text, ready to draw at any place.
pub struct Label {
    pub path: Option<Path>,
    pub width: f64,
    pub height: f64,
}

/// The empty margin `lay_out` leaves around the letters, on each side.
const MARGIN: f64 = 1.0;

impl Label {
    /// The text on one line (a line break becomes a space).
    pub fn new(text: &str, size: f64, bold: bool, family: &str) -> Self {
        let text = text.replace(['\r', '\n'], " ");
        let mut data = TextData::new(&text, family, size.max(4.0), ColorRgba::BLACK);
        data.bold = bold;
        match lay_out(&data) {
            Some(laid) => Self {
                path: laid
                    .path
                    .and_then(|p| p.transform(tiny_skia::Transform::from_translate(-MARGIN as f32, -MARGIN as f32))),
                width: (laid.width - 2.0 * MARGIN).max(0.0),
                height: (laid.height - 2.0 * MARGIN).max(0.0),
            },
            None => Self { path: None, width: 0.0, height: size },
        }
    }

    /// Like `new`, but cut with "…" when it is wider than `max_width`.
    pub fn fit(text: &str, size: f64, bold: bool, family: &str, max_width: f64) -> Self {
        let full = Self::new(text, size, bold, family);
        if full.width <= max_width.max(4.0) {
            return full;
        }
        let chars: Vec<char> = text.chars().collect();
        // The longest start of the text that fits with the dots after it.
        let (mut lo, mut hi) = (0, chars.len());
        while lo < hi {
            let mid = (lo + hi).div_ceil(2);
            let trial: String = chars[..mid].iter().collect::<String>() + "…";
            if Self::new(&trial, size, bold, family).width <= max_width.max(4.0) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        let cut: String = chars[..lo].iter().collect::<String>() + "…";
        Self::new(&cut, size, bold, family)
    }
}
