//! The top row of the ribbon: File button, tab names, search box and the hide / show arrow
//! (C# `RibbonTabControl`, `RibbonTab`, `RibbonFileButton` styles).

use egui::{Align2, Color32, FontId, Id, Rect, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2, pos2, vec2};
use fe_ui_kit::icons::glyphs;
use fe_ui_kit::theme::{colors, metrics};

use super::model::TABS;
use super::{RibbonAction, RibbonState};

const TAB_FONT: f32 = 12.5;
const TAB_TOP: f32 = 3.0;

/// Draws the row into `row` (a rect already allocated by the caller).
pub fn show(ui: &mut Ui, row: Rect, state: &mut RibbonState) -> Option<RibbonAction> {
    ui.painter().rect_filled(row, 0, colors::WINDOW);
    let mut action = None;
    if file_button(ui, row) {
        action = Some(RibbonAction::File);
    }
    tabs(ui, row, state);
    collapse_button(ui, row, state);
    search_box(ui, row, state);
    action
}

fn file_button(ui: &mut Ui, row: Rect) -> bool {
    let font = FontId::proportional(TAB_FONT);
    let text_w = ui.fonts_mut(|f| f.layout_no_wrap("File".into(), font.clone(), colors::TEXT).size().x);
    let size = vec2(10.0 + 13.0 + 7.0 + text_w + 12.0, row.height() - TAB_TOP);
    let rect = Rect::from_min_size(pos2(row.left() + 6.0, row.top() + TAB_TOP), size);
    let response =
        ui.interact(rect, Id::unique("ribbon-file"), Sense::click()).on_hover_text("Open, save, export and more");
    let fill = if response.hovered() { colors::ACCENT_SOFT } else { colors::HOVER };
    let painter = ui.painter();
    painter.rect(rect, top_corners(), fill, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
    let y = rect.center().y;
    painter.text(
        pos2(rect.left() + 10.0, y + 1.0),
        Align2::LEFT_CENTER,
        glyphs::MENU,
        glyphs::font(13.0),
        colors::TEXT,
    );
    painter.text(pos2(rect.left() + 30.0, y), Align2::LEFT_CENTER, "File", font, colors::TEXT);
    response.clicked()
}

fn tabs(ui: &mut Ui, row: Rect, state: &mut RibbonState) {
    let font = FontId::proportional(TAB_FONT);
    let mut x = row.left() + metrics::RIBBON_TABS_LEFT;
    for (index, tab) in TABS.iter().enumerate() {
        let galley = ui.fonts_mut(|f| f.layout_no_wrap(tab.name.into(), font.clone(), Color32::WHITE));
        let selected = index == state.tab && !state.collapsed;
        let height = row.height() - TAB_TOP + if selected { 1.0 } else { 0.0 };
        let rect = Rect::from_min_size(pos2(x, row.top() + TAB_TOP), vec2(galley.size().x + 24.0, height));
        let response = ui.interact(rect, Id::unique(("ribbon-tab", index)), Sense::click());
        if response.double_clicked() {
            state.collapsed = !state.collapsed;
            state.tab = index;
        } else if response.clicked() {
            state.tab = index;
            state.collapsed = false;
        }
        let fill = if selected {
            colors::PANEL
        } else if response.hovered() {
            colors::HOVER
        } else {
            Color32::TRANSPARENT
        };
        let text_color = if selected || response.hovered() { colors::TEXT } else { colors::SUBTLE_TEXT };
        let painter = ui.painter();
        painter.rect(rect, top_corners(), fill, Stroke::new(1.0, colors::DIVIDER), egui::StrokeKind::Inside);
        if selected {
            // Hide the border under the selected tab so it joins the body.
            let gap = Rect::from_min_max(
                pos2(rect.left() + 1.0, rect.bottom() - 1.0),
                pos2(rect.right() - 1.0, rect.bottom()),
            );
            painter.rect_filled(gap, 0, colors::PANEL);
            let line_y = rect.bottom() - 2.0;
            let line = Rect::from_min_max(pos2(rect.left() + 8.0, line_y), pos2(rect.right() - 8.0, line_y + 2.0));
            painter.rect_filled(line, 0, colors::ACCENT);
        }
        let text_pos = pos2(rect.center().x - galley.size().x / 2.0, rect.top() + 3.0);
        painter.galley_with_override_text_color(text_pos, galley, text_color);
        x = rect.right() + 2.0;
    }
}

fn collapse_button(ui: &mut Ui, row: Rect, state: &mut RibbonState) {
    let rect = Rect::from_min_size(pos2(row.right() - 8.0 - 24.0, row.top() + 3.0), vec2(24.0, 22.0));
    let response = ui
        .interact(rect, Id::unique("ribbon-collapse"), Sense::click())
        .on_hover_text("Hide or show the ribbon (double-click a tab does the same)");
    if response.clicked() {
        state.collapsed = !state.collapsed;
    }
    let painter = ui.painter();
    if response.hovered() {
        painter.rect_filled(rect, metrics::RADIUS, colors::HOVER);
    }
    let glyph = if state.collapsed { glyphs::CHEVRON_DOWN } else { glyphs::CHEVRON_UP };
    painter.text(rect.center(), Align2::CENTER_CENTER, glyph, glyphs::font(11.0), colors::SUBTLE_TEXT);
}

fn search_box(ui: &mut Ui, row: Rect, state: &mut RibbonState) {
    let rect = Rect::from_min_size(
        pos2(row.right() - 40.0 - metrics::SEARCH_W, row.top() + 3.0),
        vec2(metrics::SEARCH_W, 24.0),
    );
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
    let edit = TextEdit::singleline(&mut state.search)
        .hint_text(egui::RichText::new("Search commands").color(colors::SUBTLE_TEXT))
        .font(FontId::proportional(12.0))
        .margin(egui::Margin { left: 26, right: 6, top: 4, bottom: 4 })
        .desired_width(metrics::SEARCH_W);
    child
        .add_sized(Vec2::new(metrics::SEARCH_W, 24.0), edit)
        .on_hover_text("Type what you want to do, for example rotate, blur or export");
    ui.painter().text(
        pos2(rect.left() + 9.0, rect.center().y),
        Align2::LEFT_CENTER,
        glyphs::SEARCH,
        glyphs::font(11.0),
        colors::SUBTLE_TEXT,
    );
}

fn top_corners() -> egui::CornerRadius {
    egui::CornerRadius { nw: 4, ne: 4, sw: 0, se: 0 }
}
