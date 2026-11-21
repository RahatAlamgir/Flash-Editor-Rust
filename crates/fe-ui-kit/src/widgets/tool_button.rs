//! One button in the toolbox (C# `ToolboxPanel`): a line icon, a soft blue background with a left accent bar
//! when active, and a small corner triangle when the group holds more tools.

use egui::{Color32, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, pos2};

use crate::icons;
use crate::theme::{colors, metrics};

/// What happened to a tool button.
pub struct ToolButtonResponse {
    /// The whole button.
    pub response: Response,
    /// The corner triangle was clicked (open the group list).
    pub triangle_clicked: bool,
}

/// Draws a tool button with `icon` (a line icon name).
pub fn tool_button(ui: &mut Ui, icon: &str, icon_color: Color32, active: bool, has_more: bool) -> ToolButtonResponse {
    let size = Vec2::from(metrics::TOOL_BUTTON);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let triangle_rect = Rect::from_min_max(rect.max - Vec2::splat(14.0), rect.max);
    let triangle_clicked = has_more
        && response.clicked()
        && ui.input(|i| i.pointer.interact_pos()).is_some_and(|p| triangle_rect.contains(p));
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        if active {
            painter.rect_filled(rect, 5, colors::ACCENT_SOFT);
            let bar = Rect::from_min_max(rect.min, pos2(rect.min.x + 2.0, rect.max.y));
            painter.rect_filled(bar, 0, colors::ACCENT);
        } else if response.hovered() {
            painter.rect_filled(rect, 5, colors::HOVER);
        }
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(metrics::ICON));
        icons::icon20(icon, icon_color).paint_at(ui, icon_rect);
        if has_more {
            let corner = rect.max - Vec2::new(3.0, 3.0);
            let points = vec![corner - Vec2::new(6.0, 0.0), corner, corner - Vec2::new(0.0, 6.0)];
            ui.painter().add(Shape::convex_polygon(points, colors::ICON, Stroke::NONE));
        }
    }
    ToolButtonResponse { response, triangle_clicked }
}
