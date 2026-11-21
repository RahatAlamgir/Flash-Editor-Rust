//! Big ribbon button: a glyph from the Windows icon font above a small label (C# style `RibbonLarge`).

use egui::{Align2, Color32, FontId, Rect, Response, Sense, Ui, Vec2, WidgetText};

use crate::icons::glyphs;
use crate::theme::{colors, metrics};

/// What a ribbon button shows.
#[derive(Clone, Copy, Debug)]
pub struct RibbonButton<'a> {
    pub glyph: char,
    pub label: &'a str,
    /// Icon color (the plain gray or the colorful style color).
    pub color: Color32,
    pub enabled: bool,
    /// Shown pressed (for toggle buttons such as Rulers).
    pub checked: bool,
}

/// Draws a big ribbon button; returns its response (check `.clicked()`).
pub fn ribbon_button(ui: &mut Ui, button: RibbonButton<'_>) -> Response {
    let label_font = FontId::proportional(11.0);
    let galley = WidgetText::from(button.label).into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        metrics::RIBBON_LABEL_MAX_W,
        label_font,
    );
    let width = (galley.size().x + 6.0).max(metrics::RIBBON_BUTTON_MIN_W);
    let height = metrics::RIBBON_BODY - 8.0;
    let sense = if button.enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), sense);
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let painter = ui.painter();
    let fill = if button.checked || (response.is_pointer_button_down_on() && button.enabled) {
        colors::ACCENT_SOFT
    } else if response.hovered() && button.enabled {
        colors::HOVER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, metrics::RADIUS, fill);
    let fade = |c: Color32| if button.enabled { c } else { c.gamma_multiply(0.38) };
    let glyph_center = egui::pos2(rect.center().x, rect.top() + 4.0 + metrics::RIBBON_GLYPH / 2.0);
    painter.text(
        glyph_center,
        Align2::CENTER_CENTER,
        button.glyph,
        glyphs::font(metrics::RIBBON_GLYPH),
        fade(button.color),
    );
    let text_top = rect.top() + 6.0 + metrics::RIBBON_GLYPH;
    let text_rect = Rect::from_min_size(egui::pos2(rect.center().x - galley.size().x / 2.0, text_top), galley.size());
    painter.galley(text_rect.min, galley, fade(colors::TEXT));
    response
}
