//! File > Save as Template: asks for the template's name.

use egui::{Id, RichText};
use fe_ui_kit::theme::colors;

/// `Some(true)` = Save, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, name: &mut String) -> Option<bool> {
    let mut result = None;
    let modal = egui::Modal::new(Id::unique("save-template")).show(ctx, |ui| {
        ui.set_width(340.0);
        ui.label(RichText::new("Save as Template").heading());
        ui.add_space(8.0);
        ui.label(RichText::new("Name of the template").color(colors::SUBTLE_TEXT));
        let edit = ui.add(egui::TextEdit::singleline(name).char_limit(60).desired_width(f32::INFINITY));
        if edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            result = Some(true);
        }
        ui.label(
            RichText::new("It appears in New project > Start from, marked with a star.")
                .size(11.0)
                .color(colors::SUBTLE_TEXT),
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("Save").fill(colors::ACCENT).min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(90.0, 28.0))).clicked() {
                result = Some(false);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    if result == Some(true) && name.trim().is_empty() {
        return None;
    }
    result
}
