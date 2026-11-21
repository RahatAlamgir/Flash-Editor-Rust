//! The window of an adjustment or filter with settings (C# `ParametersDialog`): one slider per setting, a live
//! preview on the picture, OK / Cancel / Reset. Also used for Rotate and Straighten (no preview).

use egui::{Align2, Color32, Id, RichText, vec2};
use fe_core::effects::{Param, Values};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::app::FlashApp;
use crate::commands::effects::{self, EffectSession};

/// The slider rows of settings. True when a value changed.
pub fn param_rows(ui: &mut egui::Ui, params: &[Param], values: &mut Values) -> bool {
    let mut changed = false;
    for p in params {
        if p.toggle {
            let mut on = values.on(p.key);
            if ui.checkbox(&mut on, p.label).changed() {
                values.set(p.key, f64::from(u8::from(on)));
                changed = true;
            }
            continue;
        }
        ui.label(RichText::new(p.label).color(colors::SUBTLE_TEXT));
        let mut v = values.get(p.key);
        let unit = if p.unit == "EV" { " EV" } else { p.unit };
        if number_slider(ui, &mut v, p.min..=p.max, unit, 230.0, 60.0).changed() {
            let stepped = if p.step > 0.0 { (v / p.step).round() * p.step } else { v };
            values.set(p.key, stepped.clamp(p.min, p.max));
            changed = true;
        }
        ui.add_space(4.0);
    }
    changed
}

/// Shows the window. `Some(true)` = OK, `Some(false)` = Cancel, `None` = still open.
pub fn show(ctx: &egui::Context, app: &mut FlashApp, s: &mut EffectSession) -> Option<bool> {
    let mut result = None;
    let mut changed = s.fresh;
    s.fresh = false;
    let area = egui::Area::new(Id::NULL.with("effect-area")).anchor(Align2::RIGHT_BOTTOM, vec2(-330.0, -80.0));
    let modal =
        egui::Modal::new(Id::NULL.with("effect")).backdrop_color(Color32::TRANSPARENT).area(area).show(ctx, |ui| {
            ui.set_width(300.0);
            ui.label(RichText::new(s.effect.name).heading());
            ui.add_space(6.0);
            if !s.note.is_empty() {
                ui.add(egui::Label::new(RichText::new(s.note).size(11.5).color(colors::SUBTLE_TEXT)).wrap());
                ui.add_space(6.0);
            }
            changed |= param_rows(ui, s.effect.params, &mut s.values);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("OK").fill(colors::ACCENT).min_size(vec2(80.0, 28.0))).clicked() {
                    result = Some(true);
                }
                if ui.add(egui::Button::new("Cancel").min_size(vec2(80.0, 28.0))).clicked() {
                    result = Some(false);
                }
                if ui.add(egui::Button::new("Reset").min_size(vec2(64.0, 28.0))).clicked() {
                    s.values = s.defaults.clone();
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
        effects::request_preview(app, ctx, s);
    }
    result
}
