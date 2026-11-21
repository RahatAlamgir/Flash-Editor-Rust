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
    ui.scope_builder(egui::UiBuilder::new().max_rect(area.shrink(16.0)), |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("All {count} pages")).heading());
            if ui.button("Close grid view").clicked() {
                app.pages_ui.grid = false;
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
                        interact::card_events(ui, app, index, &events);
                    }
                    if app.project().pages().len() != count {
                        return;
                    }
                }
            });
        });
    });
    interact::drag_feedback(ui, app, &cards);
    if let Some(index) = open {
        crate::commands::pages::show(app, index);
        app.pages_ui.grid = false;
    }
}
