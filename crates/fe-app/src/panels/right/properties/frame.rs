//! Properties > Picture in frame (C# `ObjectPropertiesViewModel`, frame part): zoom and position of the picture that
//! fills a shape, and taking it out again.

use egui::Ui;

use super::shape::slider;
use crate::app::FlashApp;
use crate::commands::{objects as cmd, parts};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    let Some(fill) = parts::target_object(app).and_then(|o| o.picture_fill()) else { return };
    super::note(
        ui,
        &format!(
            "\"{}\" fills this shape. Double-click the frame, then drag to move the picture; the wheel zooms.",
            fill.name
        ),
    );
    let (mut zoom, mut x, mut y) = (fill.zoom * 100.0, fill.pan_x * 100.0, fill.pan_y * 100.0);
    let mut changed = slider(ui, "Zoom", &mut zoom, 25.0..=500.0, "%");
    changed |= slider(ui, "Left / right", &mut x, -100.0..=100.0, "%");
    changed |= slider(ui, "Up / down", &mut y, -100.0..=100.0, "%");
    if changed {
        cmd::change_object(app, "Picture in Frame", |o| {
            o.set_picture_view(zoom.round() / 100.0, x.round() / 100.0, y.round() / 100.0)
        });
    }
    ui.horizontal(|ui| {
        if ui.button("Center").clicked() {
            cmd::change_object(app, "Picture in Frame", |o| o.set_picture_view(fill.zoom, 0.0, 0.0));
        }
        if ui.button("Remove picture").on_hover_text("The frame is empty again").clicked() {
            cmd::change_object(app, "Remove Picture", |o| o.set_picture_fill(None));
        }
    });
}
