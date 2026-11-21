//! The strip under the canvas: "PAGES / n in this project / Grid view" on the left, then the page cards with a
//! "+" in every gap, and "Add page" at the end.

use egui::{Align, Align2, Color32, FontId, Layout, Rect, RichText, Sense, Stroke, Ui, Vec2, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::{colors, metrics};
use fe_ui_kit::widgets::section_title;

use super::{card, interact};
use crate::app::FlashApp;
use crate::commands::pages as cmd;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let bar = ui.max_rect();
    ui.painter().hline(bar.x_range(), bar.top() + 0.5, Stroke::new(1.0, colors::DIVIDER));
    let count = app.project().pages().len();
    let mut cards: Vec<(usize, Rect)> = Vec::new();
    ui.horizontal(|ui| {
        ui.add_space(10.0);
        ui.allocate_ui_with_layout(vec2(92.0, bar.height()), Layout::top_down(Align::Min), |ui| {
            ui.add_space(bar.height() / 2.0 - 34.0);
            ui.label(section_title("Pages"));
            ui.label(RichText::new(format!("{count} in this project")).size(11.0).color(colors::SUBTLE_TEXT));
            ui.add_space(4.0);
            let grid = egui::Button::new(RichText::new("Grid view").size(12.0))
                .min_size(vec2(0.0, 24.0))
                .selected(app.pages_ui.grid);
            if ui
                .add(grid)
                .on_hover_text(
                    "See all pages as big pictures; drag them to change the order, double-click one to open it",
                )
                .clicked()
            {
                app.pages_ui.grid = !app.pages_ui.grid;
            }
        });
        ui.add_space(4.0);
        egui::ScrollArea::horizontal().id_salt("page-strip").show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let [w, h] = metrics::PAGE_CARD;
                for index in 0..count {
                    if card::gap_button(ui, 8.0 + h, index) {
                        cmd::add_at(app, index);
                    }
                    let events = card::show(ui, app, index, Vec2::new(w, h));
                    cards.push((index, events.response.rect));
                    interact::card_events(ui, app, index, &events);
                    if app.project().pages().len() != count {
                        return; // the pages changed: draw again next frame
                    }
                }
                ui.add_space(20.0);
                add_page_card(ui, app);
            });
        });
    });
    interact::drag_feedback(ui, app, &cards);
}

fn add_page_card(ui: &mut Ui, app: &mut FlashApp) {
    let (rect, response) = ui.allocate_exact_size(vec2(84.0, 8.0 + metrics::PAGE_CARD[1]), Sense::click());
    let card = Rect::from_min_size(rect.min + vec2(0.0, 8.0), vec2(84.0, metrics::PAGE_CARD[1]));
    let border = if response.hovered() { colors::SUBTLE_TEXT } else { colors::DIVIDER };
    let painter = ui.painter();
    painter.rect(card, 4, Color32::TRANSPARENT, Stroke::new(1.0, border), egui::StrokeKind::Inside);
    painter.text(
        card.center() - Vec2::new(0.0, 8.0),
        Align2::CENTER_CENTER,
        glyphs::ADD,
        glyphs::font(16.0),
        colors::ICON,
    );
    painter.text(
        card.center() + Vec2::new(0.0, 14.0),
        Align2::CENTER_CENTER,
        "Add page",
        FontId::proportional(12.0),
        colors::SUBTLE_TEXT,
    );
    if response.on_hover_text("Add a new page at the end (Ctrl+Alt+N adds one after the shown page)").clicked() {
        let end = app.project().pages().len();
        cmd::add_at(app, end);
    }
}
