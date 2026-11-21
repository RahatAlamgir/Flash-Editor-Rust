//! A part of a group, shown indented under the group's row (C# `GroupChildViewModel`). Click picks the part: the
//! handles, Properties and Delete then work on it, like double-clicking it on the canvas.

use egui::{FontId, Sense, Ui, pos2, vec2};
use fe_ui_kit::theme::colors;

const HEIGHT: f32 = 28.0;

/// Draws one part row; true when it was clicked.
pub fn show(ui: &mut Ui, name: &str, selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::click());
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 0, colors::ACCENT_SOFT);
    } else if response.hovered() {
        painter.rect_filled(rect, 0, colors::HOVER);
    }
    // A small elbow, then the name.
    let x = rect.left() + 40.0;
    let stroke = egui::Stroke::new(1.0, colors::DIVIDER);
    painter.line_segment([pos2(x, rect.top()), pos2(x, rect.center().y)], stroke);
    painter.line_segment([pos2(x, rect.center().y), pos2(x + 10.0, rect.center().y)], stroke);
    let text = name.lines().next().unwrap_or_default();
    painter.with_clip_rect(rect).text(
        pos2(x + 16.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        FontId::proportional(12.0),
        colors::TEXT,
    );
    response
        .on_hover_text("A part of the group. Click to edit it on its own; Esc goes back to the whole group.")
        .clicked()
}
