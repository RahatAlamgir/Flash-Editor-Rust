//! Properties > Barcode: the text, the kind of barcode, printed text, colors.

use egui::Ui;
use fe_core::codes::{BarcodeKind, barcode};
use fe_core::objects::BarcodeObjData;

use super::label;
use super::shape::swatch;
use crate::app::FlashApp;
use crate::commands::codes::{ObjectColor, change_barcode};
use crate::commands::parts;
use crate::dialogs::{ColorPicker, ColorTarget, Dialog};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(c) = parts::target_object(app).and_then(|o| o.barcode()) else { return };
    ui.horizontal(|ui| {
        label(ui, "Type");
        egui::ComboBox::from_id_salt("barcode-kind").selected_text(c.kind.name()).width(140.0).show_ui(ui, |ui| {
            for kind in BarcodeKind::ALL {
                if ui.selectable_label(c.kind == kind, kind.name()).clicked() {
                    change_barcode(app, "Barcode Type", |d| {
                        d.kind = kind;
                        // A text the new kind cannot show is replaced by a sample that fits.
                        if barcode::validate(&d.content, kind).is_some() {
                            d.content = BarcodeObjData::sample(kind).to_owned();
                        }
                    });
                }
            }
        });
    });
    let mut text = c.content.clone();
    if ui.add(egui::TextEdit::singleline(&mut text).desired_width(f32::INFINITY)).changed() {
        change_barcode(app, "Barcode Text", |d| d.content = text.clone());
    }
    match barcode::validate(&c.content, c.kind) {
        Some(problem) => {
            ui.colored_label(egui::Color32::from_rgb(0xF8, 0x71, 0x71), problem);
        }
        None => super::note(ui, c.kind.describe()),
    }
    let mut show_text = c.show_text;
    if ui.checkbox(&mut show_text, "Show the numbers under the bars").changed() {
        change_barcode(app, "Barcode Text Shown", |d| d.show_text = show_text);
    }
    ui.horizontal(|ui| {
        label(ui, "Bars");
        if swatch(ui, Some(c.foreground), "Color of the bars") {
            app.dialog = Some(Dialog::Color(ColorPicker::new(
                ColorTarget::Object(ObjectColor::BarcodeForeground),
                c.foreground,
            )));
        }
        ui.add_space(8.0);
        label(ui, "Back");
        if swatch(ui, (!c.transparent_background).then_some(c.background), "Color behind the bars") {
            app.dialog = Some(Dialog::Color(ColorPicker::new(
                ColorTarget::Object(ObjectColor::BarcodeBackground),
                c.background,
            )));
        }
    });
    let mut transparent = c.transparent_background;
    if ui.checkbox(&mut transparent, "Transparent background").changed() {
        change_barcode(app, "Barcode Background", |d| d.transparent_background = transparent);
    }
    super::note(
        ui,
        "Test a printed barcode with a scanner or your phone: scanners need dark bars on a light background.",
    );
}
