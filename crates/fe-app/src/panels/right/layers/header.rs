//! The top of the Layers panel: Mix (blend mode) and Opacity of the active layer, then Select all / Lock all /
//! Unlock all / Merge.

use egui::{ComboBox, RichText, Ui, vec2};
use fe_core::document::layer_ops;
use fe_core::imaging::BlendMode;
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::number_slider;

use crate::app::FlashApp;
use crate::commands::layers as cmd;

fn label(ui: &mut Ui, text: &str) {
    ui.add_sized([54.0, 22.0], egui::Label::new(RichText::new(text).size(12.0).color(colors::SUBTLE_TEXT)));
}

pub fn mix_and_opacity(ui: &mut Ui, app: &mut FlashApp) {
    let Some(layer) = app.doc().active_layer() else { return };
    let (blend, opacity) = (layer.blend, layer.opacity());
    ui.add_space(8.0);
    let mut new_blend = blend;
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        label(ui, "Mix");
        ComboBox::from_id_salt("blend-mode")
            .width(ui.available_width() - 14.0)
            .selected_text(blend.display_name())
            .show_ui(ui, |ui| {
                for mode in BlendMode::ALL {
                    ui.selectable_value(&mut new_blend, mode, mode.display_name()).on_hover_text(mode.description());
                }
            })
            .response
            .on_hover_text("Blend mode: how this layer's colors mix with the layers below it");
    });
    if new_blend != blend {
        cmd::set_blend(app, new_blend);
    }
    ui.add_space(4.0);
    let mut percent = (opacity * 100.0).round();
    let changed = ui
        .horizontal(|ui| {
            ui.add_space(10.0);
            label(ui, "Opacity");
            number_slider(ui, &mut percent, 0.0..=100.0, "%", 120.0, 48.0).changed()
        })
        .inner;
    if changed {
        cmd::set_opacity(app, percent / 100.0);
    }
    ui.add_space(6.0);
}

pub fn quick_buttons(ui: &mut Ui, app: &mut FlashApp) {
    let can_merge = layer_ops::merge_candidates(app.doc()).len() >= 2;
    let buttons: [(&str, &str, bool); 4] = [
        ("Select all", "Select every layer (Ctrl+A with the Select / Move tool)", true),
        (
            "Lock all",
            "Lock every selected layer that is not locked. Locked layers cannot be picked on the canvas or changed.",
            true,
        ),
        ("Unlock all", "Unlock every selected layer that is locked", true),
        (
            "Merge",
            "Quick merge: select two or more layers, then click to merge them into one layer (Ctrl+E). Use Group (Ctrl+G) to keep them editable.",
            can_merge,
        ),
    ];
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.spacing_mut().item_spacing.x = 3.0;
        let width = (ui.available_width() - 10.0 - 9.0) / 4.0;
        for (text, tip, enabled) in buttons {
            let button = egui::Button::new(RichText::new(text).size(12.0)).min_size(vec2(width, 24.0));
            if ui.add_enabled(enabled, button).on_hover_text(tip).on_disabled_hover_text(tip).clicked() {
                clicked = Some(text);
            }
        }
    });
    match clicked {
        Some("Select all") => cmd::select_all(app),
        Some("Lock all") => cmd::set_lock_of_selected(app, true),
        Some("Unlock all") => cmd::set_lock_of_selected(app, false),
        Some("Merge") => cmd::merge_selected(app),
        _ => {}
    }
    ui.add_space(6.0);
}
