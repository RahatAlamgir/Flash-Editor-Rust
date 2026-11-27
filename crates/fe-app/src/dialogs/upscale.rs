//! The AI Upscale / Clean Up window: size and strength.

use egui::{RichText, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::ai::upscale::UpscaleForm;

/// Shows the window. `Some(true)` = OK, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, form: &mut UpscaleForm) -> Option<bool> {
    let mut result = None;
    let modal = super::frame::modal(ctx, "ai-upscale", |ui| {
        ui.set_width(380.0);
        ui.label(RichText::new("AI Upscale / Clean Up").heading());
        ui.add_space(6.0);
        let note = "AI makes the picture sharper and cleaner (removes blur, blockiness and noise).\nSize 1 = same size, just \
                    cleaner. 2, 3 or 4 = that many times larger, with sharp new detail.\nStrength below 100% mixes in the \
                    original look.";
        ui.add(egui::Label::new(RichText::new(note).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(8.0);
        ui.label(RichText::new("Size (1 = same size)").color(colors::SUBTLE_TEXT));
        number_slider(ui, &mut form.scale, 1.0..=4.0, "x", 250.0, 60.0);
        form.scale = form.scale.round().clamp(1.0, 4.0);
        ui.add_space(4.0);
        ui.label(RichText::new("Strength").color(colors::SUBTLE_TEXT));
        number_slider(ui, &mut form.strength, 0.0..=100.0, "%", 250.0, 60.0);
        form.strength = form.strength.round();
        ui.add_space(8.0);
        let info = format!("This picture needs {} tiles: {} on this PC.", form.tiles, form.time_text());
        ui.add(egui::Label::new(RichText::new(info).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new("OK").fill(colors::ACCENT).min_size(vec2(80.0, 28.0))).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(vec2(80.0, 28.0))).clicked() {
                result = Some(false);
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    result
}
