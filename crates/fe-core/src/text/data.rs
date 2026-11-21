//! The typed view of a text object's fields (the C# `TextObject` record).

use serde_json::{Map, Value};

use crate::imaging::ColorRgba;
use crate::objects::json::{self, number};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextData {
    pub text: String,
    pub family: String,
    pub size: f64,
    pub color: ColorRgba,
    pub bold: bool,
    pub italic: bool,
    pub align: TextAlign,
    pub outline_width: f64,
    pub outline_color: Option<ColorRgba>,
    /// 1 = the font's own line spacing.
    pub line_spacing: f64,
    /// Degrees the text bends along a circle (0 = straight).
    pub arc: f64,
    /// Extra space after every letter / space, in font sizes.
    pub letter_spacing: f64,
    pub word_spacing: f64,
}

impl TextData {
    pub fn new(text: &str, family: &str, size: f64, color: ColorRgba) -> Self {
        Self {
            text: text.to_owned(),
            family: family.to_owned(),
            size,
            color,
            bold: false,
            italic: false,
            align: TextAlign::Left,
            outline_width: 0.0,
            outline_color: None,
            line_spacing: 1.0,
            arc: 0.0,
            letter_spacing: 0.0,
            word_spacing: 0.0,
        }
    }

    pub fn from_json(map: &Map<String, Value>) -> Option<Self> {
        Some(Self {
            text: json::str_of(map, "Text")?.to_owned(),
            family: json::str_of(map, "FontFamily").unwrap_or("Segoe UI").to_owned(),
            size: json::f64_of(map, "FontSize", 24.0),
            color: json::color_of(map, "Color").unwrap_or(ColorRgba::BLACK),
            bold: json::bool_of(map, "Bold"),
            italic: json::bool_of(map, "Italic"),
            align: match json::f64_of(map, "Align", 0.0) as i32 {
                1 => TextAlign::Center,
                2 => TextAlign::Right,
                _ => TextAlign::Left,
            },
            outline_width: json::f64_of(map, "OutlineWidth", 0.0),
            outline_color: json::color_of(map, "OutlineColor"),
            line_spacing: json::f64_of(map, "LineSpacing", 1.0),
            arc: json::f64_of(map, "Arc", 0.0),
            letter_spacing: json::f64_of(map, "LetterSpacing", 0.0),
            word_spacing: json::f64_of(map, "WordSpacing", 0.0),
        })
    }

    /// Writes the fields into a description (the box, looks and other fields stay).
    pub fn write(&self, map: &mut Map<String, Value>) {
        json::set(map, "Text", Some(self.text.clone().into()));
        json::set(map, "FontFamily", Some(self.family.clone().into()));
        json::set(map, "FontSize", Some(number(self.size)));
        json::set(map, "Color", Some(json::color_value(self.color)));
        json::set(map, "Bold", Some(self.bold.into()));
        json::set(map, "Italic", Some(self.italic.into()));
        let align = match self.align {
            TextAlign::Left => 0,
            TextAlign::Center => 1,
            TextAlign::Right => 2,
        };
        json::set(map, "Align", Some(align.into()));
        json::set(map, "OutlineWidth", Some(number(self.outline_width)));
        json::set(map, "OutlineColor", self.outline_color.map(json::color_value));
        json::set(map, "LineSpacing", Some(number(self.line_spacing)));
        json::set(map, "Arc", Some(number(self.arc)));
        json::set(map, "LetterSpacing", Some(number(self.letter_spacing)));
        json::set(map, "WordSpacing", Some(number(self.word_spacing)));
    }
}
