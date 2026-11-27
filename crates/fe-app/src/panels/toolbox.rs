//! The toolbox on the left: one button per tool group (the corner triangle or a right-click opens the
//! group's list), and the foreground / background colors at the bottom (C# `ToolboxPanel`).

use egui::{Align, Area, Color32, Frame, Id, Layout, Order, Rect, Sense, Stroke, Ui, Vec2, pos2, vec2};
use fe_core::imaging::ColorRgba;
use fe_ui_kit::icons;
use fe_ui_kit::theme::{colors, metrics};
use fe_ui_kit::widgets::tool_button;

use crate::app::FlashApp;
use crate::tools::catalog::GROUPS;

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let rect = ui.max_rect();
    ui.painter().vline(rect.right() - 0.5, rect.y_range(), Stroke::new(1.0, colors::DIVIDER));
    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        ui.add_space(12.0);
        color_swatches(ui, app);
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(6.0);
            ui.spacing_mut().item_spacing.y = 2.0;
            for index in 0..GROUPS.len() {
                group_button(ui, app, index);
            }
        });
    });
}

fn group_button(ui: &mut Ui, app: &mut FlashApp, index: usize) {
    let tool = app.tools.current_of(index);
    let has_more = GROUPS[index].len() > 1;
    let active = app.tools.is_group_active(index);
    let result = tool_button(ui, tool.icon(), colors::ICON, active, has_more);
    let tip = if has_more {
        format!("{}  -  click the corner triangle for more tools", tool.tooltip())
    } else {
        tool.tooltip()
    };
    let response = result.response.on_hover_text(tip);
    if result.triangle_clicked || (has_more && response.secondary_clicked()) {
        app.tools.open_group = if app.tools.open_group == Some(index) { None } else { Some(index) };
    } else if response.clicked() {
        app.tools.activate(tool);
    }
    if app.tools.open_group == Some(index) {
        group_list(ui, app, index, response.rect);
    }
}

/// The pop-out list of a tool group, to the right of its button.
fn group_list(ui: &mut Ui, app: &mut FlashApp, index: usize, button: Rect) {
    let area = Area::new(Id::unique(("tool-group-list", index)))
        .order(Order::Foreground)
        .fixed_pos(pos2(button.right() + 4.0, button.top()))
        .show(ui.ctx(), |ui| {
            Frame::popup(ui.style()).fill(colors::PANEL).inner_margin(4.0).show(ui, |ui| {
                for &tool in GROUPS[index] {
                    let (rect, row) = ui.allocate_exact_size(vec2(200.0, 30.0), Sense::click());
                    if row.hovered() {
                        ui.painter().rect_filled(rect, metrics::RADIUS, colors::HOVER);
                    }
                    let icon_rect = Rect::from_min_size(rect.min + vec2(8.0, 5.0), Vec2::splat(metrics::ICON));
                    icons::icon20(tool.icon(), colors::ICON).paint_at(ui, icon_rect);
                    ui.painter().text(
                        pos2(icon_rect.right() + 10.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        tool.tooltip(),
                        egui::FontId::proportional(12.5),
                        colors::TEXT,
                    );
                    if row.clicked() {
                        app.tools.activate(tool);
                    }
                }
            });
        });
    // Close when clicking anywhere else.
    if ui.input(|i| i.pointer.any_pressed())
        && !area.response.contains_pointer()
        && !button.contains(ui.input(|i| i.pointer.interact_pos()).unwrap_or_default())
    {
        app.tools.open_group = None;
    }
}

/// The paint color: used by the Brush, the Curve pen and as the starting color of new text and icons.
fn color_swatches(ui: &mut Ui, app: &mut FlashApp) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
    let clicked = swatch(ui, rect, app.foreground, Color32::from_gray(0xEE), "Paint color (click to change)");
    if clicked {
        use crate::dialogs::{ColorPicker, ColorTarget, Dialog};
        app.dialog = Some(Dialog::Color(ColorPicker::new(ColorTarget::Foreground, app.foreground)));
    }
}

/// A color square; true when clicked. The background square is only clickable where the front one does not
/// cover it.
fn swatch(ui: &mut Ui, rect: Rect, color: ColorRgba, border: Color32, tip: &str) -> bool {
    let fill = Color32::from_rgba_unmultiplied(color.r, color.g, color.b, color.a);
    ui.painter().rect(rect, 0, fill, Stroke::new(1.0, border), egui::StrokeKind::Inside);
    ui.interact(rect, Id::unique(tip), Sense::click()).on_hover_text(tip).clicked()
}
