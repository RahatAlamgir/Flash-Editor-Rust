//! Settings > Brand Kit (C# `BrandKitDialog`): the colors and fonts used again and again, kept for every project.
//! The colors appear first in every color picker; the fonts appear as quick buttons when a text is selected.
//! Changes are kept at once (there is nothing to save or cancel).

use egui::{Color32, RichText, Sense, Stroke, vec2};
use fe_core::imaging::ColorRgba;
use fe_ui_kit::theme::colors;

use super::{ColorPicker, ColorTarget, Dialog};
use crate::app::FlashApp;

/// How many colors and fonts a brand kit keeps.
pub const MAX_COLORS: usize = 12;
pub const MAX_FONTS: usize = 3;

/// The open dialog (everything it edits lives in the settings).
pub struct BrandKit;

fn hex(c: ColorRgba) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
}

/// Shows the dialog. False once it is closed.
pub fn show(ctx: &egui::Context, app: &mut FlashApp) -> bool {
    let mut open = true;
    let modal = super::frame::modal(ctx, "brand-kit", |ui| {
        ui.set_width(460.0);
        ui.label(RichText::new("Brand Kit").heading());
        ui.add_space(6.0);
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Your colors and fonts, kept for every project. The colors appear first in every color picker; the \
                     fonts appear as quick buttons when a text is selected.",
                )
                .color(colors::SUBTLE_TEXT),
            )
            .wrap(),
        );
        ui.add_space(12.0);
        colors_section(ui, app);
        ui.add_space(12.0);
        ui.label(RichText::new("Fonts (up to 3)").size(12.0).color(colors::SUBTLE_TEXT));
        ui.add_space(4.0);
        fonts_section(ui, app);
        ui.add_space(14.0);
        if ui.add(egui::Button::new("Done").fill(colors::ACCENT).min_size(vec2(100.0, 28.0))).clicked() {
            open = false;
        }
    });
    if modal.should_close() {
        open = false;
    }
    open
}

fn colors_section(ui: &mut egui::Ui, app: &mut FlashApp) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Colors").size(12.0).color(colors::SUBTLE_TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let can = app.settings.brand_colors.len() < MAX_COLORS;
            if ui
                .add_enabled(can, egui::Button::new(RichText::new("Add colors from this picture").size(11.0)))
                .on_hover_text("Takes the main colors of the picture you are working on")
                .clicked()
            {
                let picture = fe_core::paint::compositor::flatten(app.doc());
                for color in fe_core::imaging::palette::extract(&picture, 6) {
                    if app.settings.brand_colors.len() >= MAX_COLORS {
                        break;
                    }
                    let h = hex(color);
                    if !app.settings.brand_colors.iter().any(|c| c.eq_ignore_ascii_case(&h)) {
                        app.settings.brand_colors.push(h);
                    }
                }
            }
        });
    });
    ui.add_space(4.0);
    let (mut picked, mut removed) = (None, None);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (i, h) in app.settings.brand_colors.iter().enumerate() {
            let Some(c) = ColorRgba::parse_hex(h) else { continue };
            let (rect, response) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click());
            ui.painter().rect(
                rect,
                4,
                Color32::from_rgb(c.r, c.g, c.b),
                Stroke::new(1.0, colors::DIVIDER),
                egui::StrokeKind::Inside,
            );
            let response = response.on_hover_text("Click to change this color, right-click to remove it");
            if response.clicked() {
                picked = Some((i, c));
            }
            if response.secondary_clicked() {
                removed = Some(i);
            }
        }
    });
    if let Some(i) = removed {
        app.settings.brand_colors.remove(i);
    }
    if let Some((i, c)) = picked {
        app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::BrandColor(Some(i)), c)));
    }
    if ui.add_enabled(app.settings.brand_colors.len() < MAX_COLORS, egui::Button::new("+ Add a color")).clicked() {
        let start = ColorRgba::rgb(59, 130, 246);
        app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::BrandColor(None), start)));
    }
    let n = app.settings.brand_colors.len();
    let hint = if n == 0 {
        "No colors yet. Add one, or take them from the picture you are working on.".to_owned()
    } else {
        format!("{n} of {MAX_COLORS} colors.")
    };
    ui.label(RichText::new(hint).size(11.0).color(colors::SUBTLE_TEXT));
}

fn fonts_section(ui: &mut egui::Ui, app: &mut FlashApp) {
    let mut fonts: Vec<String> = app.settings.brand_fonts.clone();
    fonts.resize(MAX_FONTS, String::new());
    let mut changed = false;
    for (i, family) in fonts.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            let mut name = if family.is_empty() { "(none)".to_owned() } else { family.clone() };
            if crate::tools::options_bar::font_picker(ui, &format!("brand-font-{i}"), &mut name, 260.0) {
                *family = name;
                changed = true;
            }
            if !family.is_empty() && ui.small_button("✕").on_hover_text("Remove this font").clicked() {
                family.clear();
                changed = true;
            }
        });
    }
    if changed {
        let mut kept: Vec<String> = Vec::new();
        for f in fonts.into_iter().filter(|f| !f.is_empty()) {
            if !kept.iter().any(|k| k.eq_ignore_ascii_case(&f)) {
                kept.push(f);
            }
        }
        app.settings.brand_fonts = kept;
    }
}
