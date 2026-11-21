//! Properties > Transform: position, size (with Keep proportions), rotation, 90° turns, flips and Reshape.

use egui::{DragValue, Ui};
use fe_core::objects::{ObjectBox, handles};
use fe_ui_kit::widgets::number_slider;

use super::label;
use crate::app::FlashApp;
use crate::commands::objects as cmd;

/// Sets the active layer's box (a pixel layer becomes a picture first).
fn set_box(app: &mut FlashApp, what: &'static str, b: ObjectBox) {
    if app.doc().active_layer().is_some_and(|l| l.object().is_none()) && !cmd::make_picture(app) {
        return;
    }
    cmd::change_object(app, what, |o| o.set_box(b));
}

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let single = app.doc().selected_ids().len() == 1;
    let doc = app.doc();
    let Some(id) = doc.active_id() else { return };
    let part = crate::commands::parts::active_part(app);
    let Some(frame) = crate::tools::moving::transform::frame_of_part(doc, id, part) else {
        super::note(ui, "This layer is empty.");
        return;
    };
    let object = crate::commands::parts::target_object(app);
    ui.add_enabled_ui(single, |ui| {
        let mut b = frame;
        ui.horizontal(|ui| {
            label(ui, "Position");
            ui.label("X");
            let x = ui.add(DragValue::new(&mut b.x).speed(1.0).max_decimals(0));
            ui.label("Y");
            let y = ui.add(DragValue::new(&mut b.y).speed(1.0).max_decimals(0));
            if x.changed() || y.changed() {
                set_box(app, "Move", b);
            }
        });
        let keep = app.tools.mover.keep_proportions;
        ui.horizontal(|ui| {
            label(ui, "Size");
            ui.label("W");
            let mut w = b.width;
            let wr = ui.add(DragValue::new(&mut w).speed(1.0).range(1.0..=100_000.0).max_decimals(0));
            ui.label("H");
            let mut h = b.height;
            let hr = ui.add(DragValue::new(&mut h).speed(1.0).range(1.0..=100_000.0).max_decimals(0));
            if wr.changed() || hr.changed() {
                if keep && wr.changed() {
                    h = (b.height * w / b.width.max(1e-9)).max(1.0);
                } else if keep && hr.changed() {
                    w = (b.width * h / b.height.max(1e-9)).max(1.0);
                }
                set_box(app, "Resize", ObjectBox::from_center(b.center(), w, h, b.rotation));
            }
        });
        ui.checkbox(&mut app.tools.mover.keep_proportions, "Keep proportions")
            .on_hover_text("Corner handles and the size boxes keep the width / height ratio (Shift does the opposite)");
        ui.horizontal(|ui| {
            label(ui, "Rotate");
            let mut r = b.rotation;
            if number_slider(ui, &mut r, -180.0..=180.0, "°", 84.0, 50.0).changed() {
                set_box(app, "Rotate", ObjectBox { rotation: r, ..b });
            }
        });
    })
    .response
    .on_disabled_hover_text("Position and size can only be set for one layer at a time (use Align to line several up)");
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        label(ui, "Turn");
        for (text, by) in [("90° left", -90.0), ("90° right", 90.0)] {
            if ui.button(text).clicked() {
                let r = ObjectBox::normalize_angle(frame.rotation + by);
                set_box(app, "Rotate", ObjectBox { rotation: r, ..frame });
            }
        }
    });
    ui.horizontal(|ui| {
        label(ui, "Flip");
        for (text, horizontal) in [("Horizontal", true), ("Vertical", false)] {
            if !ui.button(text).clicked() {
                continue;
            }
            if part.is_some() {
                cmd::change_object(app, "Flip", |o| {
                    let (x, y) = o.flips();
                    o.set_flips(x ^ horizontal, y ^ !horizontal);
                });
            } else {
                cmd::flip(app, horizontal);
            }
        }
    });
    let can_reshape = part.is_none() && object.as_ref().is_none_or(|o| handles::reshape_points(o).is_some());
    if can_reshape {
        ui.horizontal(|ui| {
            label(ui, "Reshape");
            let on = app.tools.mover.reshaping;
            let toggle = ui
                .selectable_label(on, "Reshape corners")
                .on_hover_text("Drag the corner points of the object on the canvas (Esc ends)");
            if toggle.clicked() {
                if !on && object.is_none() && !cmd::make_picture(app) {
                    return;
                }
                app.tools.mover.reshaping = !on;
                app.tools.activate(crate::tools::ToolKind::Move);
            }
            let custom = object.as_ref().is_some_and(handles::has_custom_outline);
            if ui.add_enabled(custom, egui::Button::new("Reset")).on_hover_text("Back to the normal form").clicked() {
                cmd::change_object(app, "Reset Shape", |o| *o = handles::without_reshape(o));
            }
        });
    }
}
