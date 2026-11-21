//! The History panel: a title bar with an arrow that opens / closes the list, and Undo / Redo buttons. The list
//! shows "Open" and then every step of the project; click one to go back (or forward) to it. Steps that can
//! be redone are shown dimmed.

use egui::{Align2, FontFamily, FontId, Id, Rect, RichText, Sense, Ui, Vec2, pos2, vec2};
use fe_ui_kit::icons;
use fe_ui_kit::theme::{colors, fonts, metrics};
use fe_ui_kit::widgets::icon_button_enabled;

use crate::app::FlashApp;
use crate::commands::edit;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let area = ui.max_rect();
    let bar = Rect::from_min_size(area.min, vec2(area.width(), metrics::PANEL_TITLE));
    title_bar(ui, app, bar);
    if app.history_expanded {
        let list = Rect::from_min_max(pos2(area.left(), bar.bottom()), area.max);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(list));
        steps(&mut child, app);
    }
}

fn title_bar(ui: &mut Ui, app: &mut FlashApp, bar: Rect) {
    let (can_undo, can_redo) = (app.project().history().can_undo(), app.project().history().can_redo());
    let mut buttons =
        ui.new_child(egui::UiBuilder::new().max_rect(bar).layout(egui::Layout::right_to_left(egui::Align::Center)));
    buttons.add_space(6.0);
    buttons.spacing_mut().item_spacing.x = 0.0;
    if icon_button_enabled(&mut buttons, "Redo", "Redo (Ctrl+Y)", can_redo).clicked() {
        edit::redo(app);
    }
    if icon_button_enabled(&mut buttons, "Undo", "Undo (Ctrl+Z)", can_undo).clicked() {
        edit::undo(app);
    }
    let toggle = Rect::from_min_max(bar.min, pos2(bar.right() - 70.0, bar.bottom()));
    let response =
        ui.interact(toggle, Id::unique("history-toggle"), Sense::click()).on_hover_text("Show / hide the history list");
    if response.clicked() {
        app.history_expanded = !app.history_expanded;
    }
    let chevron = Rect::from_center_size(pos2(bar.left() + 16.0, bar.center().y), Vec2::splat(16.0));
    let rotation = if app.history_expanded { 0.0 } else { -std::f32::consts::FRAC_PI_2 };
    icons::icon("Chevron", 16.0, colors::SUBTLE_TEXT).rotate(rotation, Vec2::splat(0.5)).paint_at(ui, chevron);
    let font = FontId::new(11.0, FontFamily::Name(fonts::SEMIBOLD.into()));
    ui.painter().text(
        pos2(chevron.right() + 4.0, bar.center().y),
        Align2::LEFT_CENTER,
        "HISTORY",
        font,
        colors::SUBTLE_TEXT,
    );
}

fn steps(ui: &mut Ui, app: &mut FlashApp) {
    let history = app.project().history();
    let position = history.position();
    let names: Vec<String> =
        std::iter::once("Open".to_owned()).chain(history.steps().iter().map(|s| s.name.clone())).collect();
    let mut jump = None;
    egui::ScrollArea::vertical().auto_shrink(false).stick_to_bottom(true).show(ui, |ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for (index, name) in names.iter().enumerate() {
            let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click());
            let current = index == position;
            if current {
                ui.painter().rect_filled(rect, 0, colors::ACCENT_SOFT);
            } else if response.hovered() {
                ui.painter().rect_filled(rect, 0, colors::HOVER);
            }
            let color = if index > position { colors::SUBTLE_TEXT.gamma_multiply(0.7) } else { colors::TEXT };
            let text = RichText::new(name).size(12.5).color(color);
            ui.painter().text(
                rect.left_center() + vec2(12.0, 0.0),
                Align2::LEFT_CENTER,
                text.text(),
                FontId::proportional(12.5),
                color,
            );
            if response.clicked() && !current {
                jump = Some(index);
            }
        }
    });
    if let Some(index) = jump {
        app.project_mut().jump_to(index);
    }
}
