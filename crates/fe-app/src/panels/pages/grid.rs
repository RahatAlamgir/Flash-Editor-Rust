//! Grid view: covers the canvas with big pictures of all pages. Drag to reorder, double-click a page to open
//! it (the grid closes), right-click for the page menu.

use egui::{Rect, RichText, Ui, vec2};
use fe_ui_kit::theme::colors;

use super::{card, interact};
use crate::app::FlashApp;

const CARD: egui::Vec2 = egui::Vec2::new(240.0, 150.0);

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let area = ui.available_rect_before_wrap();
    ui.painter().rect_filled(area, 0, colors::CANVAS_AREA);
    let count = app.project().pages().len();
    let mut cards: Vec<(usize, Rect)> = Vec::new();
    let mut open = None;
    let mut add = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(area.shrink(16.0)), |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("All {count} pages")).heading());
            if ui.button("Close grid view").clicked() {
                app.pages_ui.grid = false;
            }
            if ui
                .add(egui::Button::new("New page").fill(colors::ACCENT))
                .on_hover_text("Add a blank page at the end")
                .clicked()
            {
                add = true;
            }
        });
        ui.add_space(8.0);
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(18.0, 14.0);
                for index in 0..count {
                    let events = card::show(ui, app, index, CARD);
                    cards.push((index, events.response.rect));
                    if events.response.double_clicked() {
                        open = Some(index);
                    } else {
                        interact::card_events(ui, app, index, &events, true);
                    }
                    if app.project().pages().len() != count {
                        return;
                    }
                }
                add |= add_card(ui);
            });
        });
    });
    interact::drag_feedback(ui, app, &cards);
    if add {
        crate::commands::pages::add_at(app, count);
    }
    if let Some(index) = open {
        crate::commands::pages::show(app, index);
        app.pages_ui.grid = false;
    }
}

/// The dashed "+" card after the last page. True when clicked.
fn add_card(ui: &mut Ui) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(CARD.x, 8.0 + CARD.y + 24.0), egui::Sense::click());
    let card = Rect::from_min_size(rect.min + vec2(0.0, 8.0), CARD);
    let border = if response.hovered() { colors::SUBTLE_TEXT } else { colors::DIVIDER };
    let painter = ui.painter();
    painter.rect(card, 4, egui::Color32::TRANSPARENT, egui::Stroke::new(2.0, border), egui::StrokeKind::Inside);
    painter.text(
        card.center() - vec2(0.0, 10.0),
        egui::Align2::CENTER_CENTER,
        fe_ui_kit::icons::glyphs::ADD,
        fe_ui_kit::icons::glyphs::font(26.0),
        colors::ICON,
    );
    painter.text(
        card.center() + vec2(0.0, 22.0),
        egui::Align2::CENTER_CENTER,
        "New page",
        egui::FontId::proportional(13.0),
        colors::SUBTLE_TEXT,
    );
    response.on_hover_text("Add a blank page at the end").clicked()
}
