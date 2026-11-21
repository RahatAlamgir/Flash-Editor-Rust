//! Title bars of the right-hand panels (PROPERTIES, LAYERS, HISTORY) and small section titles.

use egui::{Align, FontFamily, FontId, Layout, RichText, Sense, Ui, UiBuilder, Vec2};

use crate::theme::{colors, fonts, metrics};

/// A 28 point high title bar with the title on the left and optional content (buttons, a hint) on the right.
pub fn panel_title_bar(ui: &mut Ui, title: &str, right: impl FnOnce(&mut Ui)) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), metrics::PANEL_TITLE), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0, colors::PANEL_HEADER);
    painter.hline(rect.x_range(), rect.bottom() - 0.5, egui::Stroke::new(1.0, colors::DIVIDER));
    let mut inner = ui.new_child(UiBuilder::new().max_rect(rect.shrink2(Vec2::new(10.0, 0.0))));
    inner.with_layout(Layout::left_to_right(Align::Center), |ui| {
        ui.label(section_title(title).color(colors::TEXT));
        ui.with_layout(Layout::right_to_left(Align::Center), right);
    });
}

/// Small semibold uppercase text, like the C# panel titles.
pub fn section_title(text: &str) -> RichText {
    RichText::new(text.to_uppercase())
        .font(FontId::new(11.0, FontFamily::Name(fonts::SEMIBOLD.into())))
        .color(colors::SUBTLE_TEXT)
}
