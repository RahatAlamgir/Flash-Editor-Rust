//! Properties > Align: a 3 x 3 grid to move the layer to a corner, an edge or the middle of the canvas, the six
//! align buttons, and Distribute for three or more layers.

use egui::Ui;
use fe_core::document::object_ops::AlignKind::{self, *};
use fe_ui_kit::icons;
use fe_ui_kit::theme::colors;

use crate::app::FlashApp;
use crate::commands::objects as cmd;

fn icon_button(ui: &mut Ui, icon: &str, tip: &str) -> bool {
    let image = icons::icon(icon, 16.0, colors::ICON);
    ui.add_sized([34.0, 28.0], egui::Button::image(image)).on_hover_text(tip).clicked()
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    super::note(ui, "Line the selected layer up with the canvas (several layers line up with each other)");
    ui.add_space(4.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.label("Position");
            let names = [("top", Top), ("middle", CenterVertically), ("bottom", Bottom)];
            for (vn, v) in names {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    for (hn, h) in [("left", Left), ("center", CenterHorizontally), ("right", Right)] {
                        let tip = format!("Move to the {vn} {hn} of the canvas");
                        let size = egui::vec2(34.0, 28.0);
                        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
                        let fill = if response.hovered() { colors::HOVER } else { colors::INPUT };
                        ui.painter().rect_filled(rect, 3, fill);
                        // A small square showing where the layer goes.
                        let (fx, fy) = (pos(h), pos(v));
                        let dot = egui::Rect::from_center_size(
                            egui::pos2(rect.left() + 7.0 + fx * 20.0, rect.top() + 6.0 + fy * 16.0),
                            egui::vec2(8.0, 6.0),
                        );
                        ui.painter().rect_filled(dot, 1, colors::ACCENT);
                        if response.on_hover_text(tip).clicked() {
                            cmd::align_both(app, h, v);
                        }
                    }
                });
            }
        });
        ui.add_space(10.0);
        ui.vertical(|ui| {
            ui.label("Horizontal");
            ui.horizontal(|ui| {
                for (icon, kind, tip) in [
                    ("AlignLeft", Left, "Align to the left edge"),
                    ("AlignCenter", CenterHorizontally, "Center horizontally"),
                    ("AlignRight", Right, "Align to the right edge"),
                ] {
                    if icon_button(ui, icon, tip) {
                        cmd::align(app, kind);
                    }
                }
            });
            ui.label("Vertical");
            ui.horizontal(|ui| {
                for (icon, kind, tip) in [
                    ("AlignTop", Top, "Align to the top edge"),
                    ("AlignMiddle", CenterVertically, "Center vertically"),
                    ("AlignBottom", Bottom, "Align to the bottom edge"),
                ] {
                    if icon_button(ui, icon, tip) {
                        cmd::align(app, kind);
                    }
                }
            });
        });
    });
    if app.doc().selected_ids().len() >= 3 {
        ui.horizontal(|ui| {
            if ui.button("Distribute horizontally").clicked() {
                cmd::distribute(app, true);
            }
            if ui.button("Distribute vertically").clicked() {
                cmd::distribute(app, false);
            }
        });
    }
}

/// 0, 0.5 or 1 for start, middle, end.
fn pos(kind: AlignKind) -> f32 {
    match kind {
        Left | Top => 0.0,
        CenterHorizontally | CenterVertically => 0.5,
        Right | Bottom => 1.0,
    }
}
