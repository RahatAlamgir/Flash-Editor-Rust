//! Small square buttons with a line icon: the plain `IconButton` and the `AidToggle` above the canvas.

use egui::{Color32, Rect, Response, Sense, Ui, Vec2};

use crate::icons;
use crate::theme::{colors, metrics};

/// A 28 x 28 button with a line icon; hover shows a grey background.
pub fn icon_button(ui: &mut Ui, icon: &str, tooltip: &str) -> Response {
    icon_button_enabled(ui, icon, tooltip, true)
}

/// Like [`icon_button`], dimmed and not clickable when `enabled` is false.
pub fn icon_button_enabled(ui: &mut Ui, icon: &str, tooltip: &str, enabled: bool) -> Response {
    square(ui, icon, Vec2::splat(metrics::ICON_BUTTON), false, enabled).on_hover_text(tooltip)
}

/// A 28 x 26 on / off button (rulers, grid, guides, snapping). Checked = soft blue background.
pub fn aid_toggle(ui: &mut Ui, icon: &str, checked: &mut bool, tooltip: &str) -> Response {
    let response = square(ui, icon, Vec2::new(28.0, 26.0), *checked, true).on_hover_text(tooltip);
    if response.clicked() {
        *checked = !*checked;
    }
    response
}

fn square(ui: &mut Ui, icon: &str, size: Vec2, checked: bool, enabled: bool) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(size, sense);
    if ui.is_rect_visible(rect) {
        let fill = if checked {
            colors::ACCENT_SOFT
        } else if response.hovered() && enabled {
            colors::HOVER
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, metrics::RADIUS, fill);
        let color = if enabled { colors::ICON } else { colors::ICON.gamma_multiply(0.35) };
        icons::icon20(icon, color).paint_at(ui, Rect::from_center_size(rect.center(), Vec2::splat(metrics::ICON)));
    }
    response
}
