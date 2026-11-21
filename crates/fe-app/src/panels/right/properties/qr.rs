//! Properties > QR code (C# `ObjectPropertiesViewModel`, QR part): the text, how much damage the code survives, how
//! the squares look, the colors and the empty border.

use egui::{RichText, Ui};
use fe_core::codes::{QrLevel, max_bytes};
use fe_core::objects::QrStyle;

use super::label;
use super::shape::{slider, swatch};
use crate::app::FlashApp;
use crate::commands::codes::{ObjectColor, change_qr};
use crate::commands::parts;
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(q) = parts::target_object(app).and_then(|o| o.qr()) else { return };
    ui.label(RichText::new("What the code says").color(fe_ui_kit::theme::colors::SUBTLE_TEXT));
    let mut text = q.content.clone();
    if ui.add(egui::TextEdit::multiline(&mut text).desired_rows(2).desired_width(f32::INFINITY)).changed() {
        change_qr(app, "QR Text", |d| d.content = text.clone());
    }
    let limit = max_bytes(q.level);
    if q.content.len() > limit {
        ui.colored_label(
            egui::Color32::from_rgb(0xF8, 0x71, 0x71),
            format!(
                "Too long for a QR code ({} of {limit} letters). Shorten it or lower the protection.",
                q.content.len()
            ),
        );
    }
    ui.horizontal(|ui| {
        label(ui, "Protect");
        egui::ComboBox::from_id_salt("qr-level").selected_text(q.level.name()).width(140.0).show_ui(ui, |ui| {
            for level in QrLevel::ALL {
                if ui.selectable_label(q.level == level, level.name()).clicked() {
                    change_qr(app, "QR Protection", |d| d.level = level);
                }
            }
        });
    })
    .response
    .on_hover_text("How much damage the code survives. More protection makes a bigger, denser code.");
    ui.horizontal(|ui| {
        label(ui, "Squares");
        for style in QrStyle::ALL {
            if ui.selectable_label(q.style == style, style.name()).clicked() {
                change_qr(app, "QR Style", |d| d.style = style);
            }
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Code");
        if swatch(ui, Some(q.foreground), "Color of the squares") {
            app.dialog =
                Some(Dialog::Color(ColorPicker::new(ColorTarget::Object(ObjectColor::QrForeground), q.foreground)));
        }
        ui.add_space(8.0);
        label(ui, "Back");
        if swatch(ui, (!q.transparent_background).then_some(q.background), "Color behind the code") {
            app.dialog =
                Some(Dialog::Color(ColorPicker::new(ColorTarget::Object(ObjectColor::QrBackground), q.background)));
        }
    });
    let mut transparent = q.transparent_background;
    if ui.checkbox(&mut transparent, "Transparent background").changed() {
        change_qr(app, "QR Background", |d| d.transparent_background = transparent);
    }
    let mut quiet = f64::from(q.quiet);
    if slider(ui, "Border", &mut quiet, 0.0..=12.0, "") {
        change_qr(app, "QR Border", |d| d.quiet = quiet.round() as i32);
    }
    super::note(
        ui,
        "Dots and rounded squares look nice but are a little harder to scan: test your code. Scanners need a border of 2 or more.",
    );
}
