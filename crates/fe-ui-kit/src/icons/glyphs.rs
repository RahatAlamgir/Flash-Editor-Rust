//! Glyphs of the Windows icon font (Segoe Fluent Icons / Segoe MDL2 Assets), as used by the C# ribbon.
//! The code points are the same as in the C# `RibbonBar.xaml`.

use egui::{Color32, FontFamily, FontId, RichText};

use crate::theme::fonts;

/// The "hamburger" menu glyph on the File button.
pub const MENU: char = '\u{E700}';
/// Magnifier in the search box.
pub const SEARCH: char = '\u{E721}';
/// Arrow that hides the ribbon.
pub const CHEVRON_UP: char = '\u{E70E}';
/// Arrow that shows the ribbon again.
pub const CHEVRON_DOWN: char = '\u{E70D}';
/// Plus.
pub const ADD: char = '\u{E710}';
/// "More" (three dots).
pub const MORE: char = '\u{E712}';
/// Close cross.
pub const CLOSE: char = '\u{E711}';

/// A glyph as rich text in the icon font.
pub fn glyph(ch: char, size: f32, color: Color32) -> RichText {
    RichText::new(ch.to_string()).font(font(size)).color(color)
}

/// The icon font at a size.
pub fn font(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(fonts::ICONS.into()))
}

/// Parses "E7A7" (the hex code in the C# XAML) into a glyph.
pub fn from_hex(hex: &str) -> Option<char> {
    u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
}
