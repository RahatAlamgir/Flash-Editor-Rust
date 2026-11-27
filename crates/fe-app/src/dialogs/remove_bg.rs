//! The Remove Background (AI) window: three sliders, a live preview on the picture, OK / Cancel / Reset.

use egui::{RichText, vec2};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::ai::remove_bg::{self, RemoveBgSession};
use crate::app::FlashApp;

const NOTE: &str = "AI finds the main subject (person, animal, product...) and removes everything else. The first run takes a \
few seconds. Edge sharpness: higher = crisper cut. Shrink / grow: negative removes more around the edge, positive keeps more.";

/// Shows the window. `Some(true)` = OK, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, s: &mut RemoveBgSession) -> Option<bool> {
    let mut result = None;
    let mut changed = s.fresh;
    s.fresh = false;
    let modal = super::frame::floating(ctx, "remove-bg", vec2(-330.0, -80.0), |ui| {
        ui.set_width(300.0);
        ui.label(RichText::new("Remove Background (AI)").heading());
        ui.add_space(6.0);
        ui.add(egui::Label::new(RichText::new(NOTE).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
        ui.add_space(6.0);
        ui.label(RichText::new("Edge sharpness").color(colors::SUBTLE_TEXT));
        changed |= number_slider(ui, &mut s.sharpness, 0.0..=100.0, "%", 230.0, 60.0).changed();
        ui.add_space(4.0);
        ui.label(RichText::new("Shrink / grow edge").color(colors::SUBTLE_TEXT));
        changed |= number_slider(ui, &mut s.shift, -50.0..=50.0, "", 230.0, 60.0).changed();
        ui.add_space(4.0);
        ui.label(RichText::new("Feather").color(colors::SUBTLE_TEXT));
        changed |= number_slider(ui, &mut s.feather, 0.0..=10.0, "px", 230.0, 60.0).changed();
        s.shift = s.shift.round();
        s.feather = (s.feather * 2.0).round() / 2.0;
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let ok = egui::Button::new("OK").fill(colors::ACCENT).min_size(vec2(80.0, 28.0));
            if ui.add_enabled(s.ready(), ok).clicked() {
                result = Some(true);
            }
            if ui.add(egui::Button::new("Cancel").min_size(vec2(80.0, 28.0))).clicked() {
                result = Some(false);
            }
            if ui.add(egui::Button::new("Reset").min_size(vec2(64.0, 28.0))).clicked() {
                (s.sharpness, s.shift, s.feather) = (30.0, 0.0, 0.0);
                changed = true;
            }
            if s.busy() {
                ui.spinner();
            }
        });
    });
    if result.is_none() && modal.should_close() {
        result = Some(false);
    }
    if changed && result.is_none() {
        remove_bg::request_preview(app, s);
    }
    result
}
