//! Fonts: Segoe UI (like the C# app) and the Windows icon font (Segoe Fluent Icons, or Segoe MDL2 Assets on
//! older Windows) for the ribbon glyphs. They are read from the Windows font folder, not shipped with the app.
//! When a font is missing the egui default font is used instead.

use std::path::PathBuf;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// Font family name for semibold text (panel titles, tool names).
pub const SEMIBOLD: &str = "semibold";
/// Font family name for icon glyphs.
pub const ICONS: &str = "icons";

/// Loads the fonts and makes Segoe UI the normal font.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let regular = add(&mut fonts, "segoe-ui", &["segoeui.ttf"]);
    let semibold = add(&mut fonts, "segoe-ui-semibold", &["seguisb.ttf"]);
    let icons = add(&mut fonts, "fluent-icons", &["SegoeIcons.ttf", "segmdl2.ttf"]);
    // Segoe UI lacks some symbols the C# labels use (▾, ⌂); Windows draws them from Segoe UI Symbol.
    let symbols = add(&mut fonts, "segoe-ui-symbol", &["seguisym.ttf"]);

    let defaults = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut proportional = Vec::new();
    proportional.extend(regular.clone());
    proportional.extend(symbols);
    proportional.extend(defaults.iter().cloned());
    fonts.families.insert(FontFamily::Proportional, proportional.clone());

    let mut semi = Vec::new();
    semi.extend(semibold);
    semi.extend(proportional.iter().cloned());
    fonts.families.insert(FontFamily::Name(SEMIBOLD.into()), semi);

    let mut glyphs = Vec::new();
    glyphs.extend(icons);
    glyphs.extend(defaults);
    fonts.families.insert(FontFamily::Name(ICONS.into()), glyphs);

    ctx.set_fonts(fonts);
}

/// Reads the first font file that exists and registers it; returns its name.
fn add(fonts: &mut FontDefinitions, name: &str, files: &[&str]) -> Option<String> {
    let dir = windows_fonts_dir();
    for file in files {
        if let Ok(bytes) = std::fs::read(dir.join(file)) {
            fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_owned(bytes)));
            return Some(name.to_owned());
        }
    }
    log::warn!("Font {name} not found in {}; using the default font", dir.display());
    None
}

fn windows_fonts_dir() -> PathBuf {
    let windows = std::env::var_os("WINDIR").map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    windows.join("Fonts")
}
