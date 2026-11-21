//! The buttons along the bottom of the Layers panel: new, duplicate, up, down, merge, delete.

use egui::{Align, Layout, Stroke, Ui};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::icon_button_enabled;

use crate::app::FlashApp;
use crate::commands::layers as cmd;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0, colors::PANEL_FOOTER);
    ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, colors::DIVIDER));
    let doc = app.doc();
    let index = doc.active_index().unwrap_or(0);
    let count = doc.layers().len();
    let several = doc.selected_ids().len() >= 2;
    let buttons = [
        ("Delete", "Delete layer", count > 1),
        (
            "Merge",
            "Merge down (Ctrl+E). With several layers selected: merge all of them into one.",
            index > 0 || several,
        ),
        ("Down", "Move layer down", index > 0),
        ("Up", "Move layer up", index + 1 < count),
        ("Duplicate", "Duplicate layer (Ctrl+D)", true),
        ("Add", "New layer (Ctrl+Shift+N)", true),
    ];
    let mut clicked = None;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(6.0);
        ui.spacing_mut().item_spacing.x = 0.0;
        for (icon, tip, enabled) in buttons {
            if icon_button_enabled(ui, icon, tip, enabled).clicked() {
                clicked = Some(icon);
            }
        }
    });
    match clicked {
        Some("Delete") => cmd::delete(app),
        Some("Merge") => cmd::merge(app),
        Some("Down") => cmd::move_layer(app, -1),
        Some("Up") => cmd::move_layer(app, 1),
        Some("Duplicate") => cmd::duplicate(app),
        Some("Add") => cmd::new_layer(app),
        _ => {}
    }
}
