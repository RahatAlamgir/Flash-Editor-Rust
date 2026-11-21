//! QR codes and barcodes (C# `QrCodeObject`, `BarcodeObject`): the typed views of `"kind": "qr"` and
//! `"kind": "barcode"` descriptions.

use serde_json::{Map, Value};

use super::json::{self, number};
use crate::codes::{BarcodeKind, QrLevel};
use crate::imaging::ColorRgba;

/// How the dark squares of a QR code are drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QrStyle {
    #[default]
    Square,
    Rounded,
    Dots,
}

impl QrStyle {
    pub const ALL: [QrStyle; 3] = [QrStyle::Square, QrStyle::Rounded, QrStyle::Dots];

    pub fn from_number(n: u64) -> Self {
        Self::ALL.get(n as usize).copied().unwrap_or_default()
    }

    pub fn name(self) -> &'static str {
        match self {
            QrStyle::Square => "Square",
            QrStyle::Rounded => "Rounded",
            QrStyle::Dots => "Dots",
        }
    }
}

/// A QR code that stays editable.
#[derive(Clone, Debug, PartialEq)]
pub struct QrData {
    /// What the code says: a web address, a phone number, plain text...
    pub content: String,
    pub foreground: ColorRgba,
    pub background: ColorRgba,
    pub level: QrLevel,
    pub style: QrStyle,
    /// The empty border around the code, in modules. Scanners need at least 2 to 4.
    pub quiet: i32,
    pub transparent_background: bool,
}

impl QrData {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_owned(),
            foreground: ColorRgba::BLACK,
            background: ColorRgba::WHITE,
            level: QrLevel::Medium,
            style: QrStyle::Square,
            quiet: 2,
            transparent_background: false,
        }
    }

    pub fn from_json(map: &Map<String, Value>) -> Option<Self> {
        Some(Self {
            content: json::str_of(map, "Content")?.to_owned(),
            foreground: json::color_of(map, "Foreground").unwrap_or(ColorRgba::BLACK),
            background: json::color_of(map, "Background").unwrap_or(ColorRgba::WHITE),
            level: QrLevel::from_number(json::f64_of(map, "Level", 1.0) as u64),
            style: QrStyle::from_number(json::f64_of(map, "Style", 0.0) as u64),
            quiet: json::f64_of(map, "Quiet", 2.0) as i32,
            transparent_background: json::bool_of(map, "TransparentBackground"),
        })
    }

    pub fn write(&self, map: &mut Map<String, Value>) {
        json::set(map, "Content", Some(self.content.clone().into()));
        json::set(map, "Foreground", Some(json::color_value(self.foreground)));
        json::set(map, "Background", Some(json::color_value(self.background)));
        json::set(map, "Level", Some((self.level as u32).into()));
        json::set(map, "Style", Some((self.style as u32).into()));
        json::set(map, "Quiet", Some(number(f64::from(self.quiet))));
        json::set(map, "TransparentBackground", Some(self.transparent_background.into()));
    }
}

/// A barcode that stays editable.
#[derive(Clone, Debug, PartialEq)]
pub struct BarcodeObjData {
    pub content: String,
    pub kind: BarcodeKind,
    pub foreground: ColorRgba,
    pub background: ColorRgba,
    /// Print the numbers or letters under the bars.
    pub show_text: bool,
    pub transparent_background: bool,
}

impl BarcodeObjData {
    pub fn new(content: &str, kind: BarcodeKind) -> Self {
        Self {
            content: content.to_owned(),
            kind,
            foreground: ColorRgba::BLACK,
            background: ColorRgba::WHITE,
            show_text: true,
            transparent_background: false,
        }
    }

    pub fn from_json(map: &Map<String, Value>) -> Option<Self> {
        Some(Self {
            content: json::str_of(map, "Content")?.to_owned(),
            kind: BarcodeKind::from_number(json::f64_of(map, "Kind", 0.0) as u64),
            foreground: json::color_of(map, "Foreground").unwrap_or(ColorRgba::BLACK),
            background: json::color_of(map, "Background").unwrap_or(ColorRgba::WHITE),
            show_text: map.get("ShowText").and_then(Value::as_bool).unwrap_or(true),
            transparent_background: json::bool_of(map, "TransparentBackground"),
        })
    }

    pub fn write(&self, map: &mut Map<String, Value>) {
        json::set(map, "Content", Some(self.content.clone().into()));
        json::set(map, "Kind", Some((self.kind as u32).into()));
        json::set(map, "Foreground", Some(json::color_value(self.foreground)));
        json::set(map, "Background", Some(json::color_value(self.background)));
        json::set(map, "ShowText", Some(self.show_text.into()));
        json::set(map, "TransparentBackground", Some(self.transparent_background.into()));
    }

    /// A sample text that fits the kind.
    pub fn sample(kind: BarcodeKind) -> &'static str {
        match kind {
            BarcodeKind::Ean13 => "590123412345",
            BarcodeKind::UpcA => "03600029145",
            BarcodeKind::Ean8 => "9638507",
            BarcodeKind::Code39 => "FLASH-123",
            BarcodeKind::Code128 => "FLASH-12345",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_survive_json() {
        let mut map = Map::new();
        let mut qr = QrData::new("https://example.com");
        qr.style = QrStyle::Dots;
        qr.level = QrLevel::High;
        qr.write(&mut map);
        assert_eq!(QrData::from_json(&map), Some(qr));
        let mut map = Map::new();
        let bar = BarcodeObjData::new("123", BarcodeKind::Code39);
        bar.write(&mut map);
        assert_eq!(BarcodeObjData::from_json(&map), Some(bar));
    }
}
