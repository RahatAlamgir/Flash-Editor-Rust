//! Lists and small windows that open over the main window: the list under a ribbon "▾" button (Order for now)
//! and Settings > Undo Steps.

use egui::{Id, RichText, Ui};
use fe_core::history::{MAX_STEPS, MIN_STEPS};
use fe_ui_kit::theme::colors;
use fe_ui_kit::widgets::{number_slider, popup_below};

use super::ribbon::model::TABS;
use crate::app::FlashApp;
use crate::commands::{layers as cmd, mask};

pub fn show(ui: &mut Ui, app: &mut FlashApp) {
    ribbon_menu(ui, app);
    undo_steps_dialog(ui, app);
}

fn ribbon_menu(ui: &mut Ui, app: &mut FlashApp) {
    let Some((tab, group, item, rect)) = app.ribbon_menu else { return };
    let label = TABS[tab].groups[group].items[item].label;
    let mut open = true;
    let mut chosen = None;
    let mut shape = None;
    let mut frame = None;
    let mut chart = None;
    let mut image_op: Option<i32> = None;
    let mut crop_op: Option<i32> = None;
    let mut barcode = None;
    let mut icons = std::mem::take(&mut app.shape_icons);
    popup_below(ui, Id::unique("ribbon-menu"), rect, &mut open, |ui| {
        ui.set_min_width(240.0);
        if label == "Order ▾" {
            for (text, keys, value) in [
                ("Bring to Front", "Ctrl+Shift+]", 0),
                ("Bring Forward (Move Up)", "Ctrl+]", 1),
                ("Send Backward (Move Down)", "Ctrl+[", 2),
                ("Send to Back", "Ctrl+Shift+[", 3),
            ] {
                if ui.add(egui::Button::new(text).shortcut_text(keys).frame(false)).clicked() {
                    chosen = Some(value);
                }
            }
        }
        if label == "Shapes ▾"
            && let Some(kind) = super::shape_menu::show(ui, &mut icons)
        {
            shape = Some(kind);
        }
        if label == "Frame ▾" {
            for (text, tip, kind) in [
                ("Rectangle", "A plain frame", "Rectangle"),
                ("Rounded", "A frame with rounded corners", "Rounded"),
                ("Circle", "A round frame", "Circle"),
                ("Heart", "A heart-shaped frame", "Heart"),
                ("Star", "A star-shaped frame", "Star"),
            ] {
                if ui.add(egui::Button::new(text).frame(false)).on_hover_text(tip).clicked() {
                    frame = Some(kind);
                }
            }
        }
        if label == "Crop Layer ▾" {
            if ui
                .add(egui::Button::new("Crop This Object...").frame(false))
                .on_hover_text("Cut the selected layer or object down with a box")
                .clicked()
            {
                crop_op = Some(1);
            }
            let has_selection = app.doc().selected_area().is_some();
            if ui.add_enabled(has_selection, egui::Button::new("Crop Layer to Selection").frame(false)).clicked() {
                crop_op = Some(2);
            }
        }
        if label == "Rotate ▾" {
            for (text, value) in [
                ("Rotate 90° right", 1),
                ("Rotate 90° left", 2),
                ("Rotate 180°", 3),
                ("Rotate any angle...", 4),
                ("Flip Horizontal", 5),
                ("Flip Vertical", 6),
            ] {
                if ui.add(egui::Button::new(text).frame(false)).clicked() {
                    image_op = Some(value);
                }
            }
        }
        if label == "Chart ▾" {
            for kind in fe_core::objects::ChartKind::ALL {
                if ui.add(egui::Button::new(kind.name()).frame(false)).clicked() {
                    chart = Some(kind);
                }
            }
        }
        if label == "Barcode ▾" {
            for kind in fe_core::codes::BarcodeKind::ALL {
                if ui.add(egui::Button::new(kind.name()).frame(false)).on_hover_text(kind.describe()).clicked() {
                    barcode = Some(kind);
                }
            }
        }
        if label == "Adjustment ▾" {
            for (text, tip, value) in [
                ("Add Levels Layer", "Black point, white point and middle tones, as a layer you can change later", 20),
                (
                    "Add Curves Layer",
                    "Bend the tone curve of all colors or of red, green or blue, as a layer you can change later",
                    21,
                ),
                (
                    "Add Adjustment Layer",
                    "Brightness, contrast, saturation and hue for everything below it, without changing any picture",
                    22,
                ),
                ("Add Gradient Background", "A gradient behind everything", 23),
            ] {
                if ui.add(egui::Button::new(text).frame(false)).on_hover_text(tip).clicked() {
                    chosen = Some(value);
                }
            }
        }
        if label == "Layer Mask ▾" {
            let doc = app.doc();
            let layer = doc.active_layer();
            let has_mask = layer.is_some_and(|l| l.mask().is_some());
            let can_add = layer.is_some_and(|l| !l.is_adjustment() && !has_mask);
            let has_selection = doc.selected_area().is_some() && layer.is_some_and(|l| !l.is_adjustment());
            let items: [(&str, &str, i32, bool); 8] = [
                (
                    "Add Mask: Show All",
                    "A mask that shows everything: paint black on it (Mask Brush, K) to hide parts",
                    10,
                    can_add,
                ),
                ("Add Mask: Hide All", "A mask that hides everything: paint white on it to show parts", 11, can_add),
                ("Add Mask: Show Selection Only", "The layer shows only the selected area", 12, has_selection),
                ("Add Mask: Hide Selection", "Hide the selected area of the layer", 13, has_selection),
                ("Invert Mask", "", 14, has_mask),
                ("Switch Mask On / Off", "See the layer without its mask, and back", 15, has_mask),
                ("Apply Mask to Pixels", "Make the hidden parts transparent pixels for good", 16, has_mask),
                ("Delete Mask", "", 17, has_mask),
            ];
            for (i, (text, tip, value, enabled)) in items.into_iter().enumerate() {
                if i == 4 {
                    ui.separator();
                }
                let mut response = ui.add_enabled(enabled, egui::Button::new(text).frame(false));
                if !tip.is_empty() {
                    response = response.on_hover_text(tip);
                }
                if response.clicked() {
                    chosen = Some(value);
                }
            }
        }
    });
    app.shape_icons = icons;
    if let Some(op) = crop_op {
        if op == 1 {
            crate::tools::crop::crop_active_layer(app);
        } else {
            crate::commands::image::crop_layer_to_selection(app);
        }
        app.ribbon_menu = None;
        return;
    }
    if let Some(op) = image_op {
        use crate::commands::image;
        match op {
            1 => image::rotate_quarter(app, 90),
            2 => image::rotate_quarter(app, -90),
            3 => image::rotate_quarter(app, 180),
            4 => {
                app.dialog = Some(crate::dialogs::Dialog::ImageOp(crate::dialogs::ImageOpForm::new(
                    crate::dialogs::ImageOp::Rotate,
                )));
            }
            5 => image::flip(app, true),
            _ => image::flip(app, false),
        }
        app.ribbon_menu = None;
        return;
    }
    if let Some(kind) = chart {
        crate::commands::insert::add_chart(app, kind);
        app.ribbon_menu = None;
        return;
    }
    if let Some(kind) = barcode {
        crate::commands::insert::add_barcode(app, kind);
        app.ribbon_menu = None;
        return;
    }
    if let Some(kind) = frame {
        crate::commands::frames::add_frame(app, kind);
        app.ribbon_menu = None;
        return;
    }
    if let Some(kind) = shape {
        crate::commands::objects::add_shape(app, kind);
        app.ribbon_menu = None;
        return;
    }
    match chosen {
        Some(0) => cmd::bring_to_end(app, true),
        Some(1) => cmd::move_layer(app, 1),
        Some(2) => cmd::move_layer(app, -1),
        Some(3) => cmd::bring_to_end(app, false),
        Some(10) => mask::add(app, true),
        Some(11) => mask::add(app, false),
        Some(12) => mask::from_selection(app, true),
        Some(13) => mask::from_selection(app, false),
        Some(14) => mask::invert(app),
        Some(15) => mask::toggle(app),
        Some(16) => mask::apply(app),
        Some(17) => mask::delete(app),
        Some(20) => crate::commands::adjustments::add_levels(app),
        Some(21) => crate::commands::adjustments::add_curves(app),
        Some(22) => crate::commands::adjustments::add_basic(app),
        Some(23) => crate::commands::objects::gradient_background(app),
        _ => {}
    }
    if !open || chosen.is_some() {
        app.ribbon_menu = None;
    }
}

/// Settings > Undo Steps: how many steps to keep (1 to 60). More steps need more memory.
fn undo_steps_dialog(ui: &mut Ui, app: &mut FlashApp) {
    if !app.undo_dialog {
        return;
    }
    let mut open = true;
    let mut steps = app.project().history().max_steps() as f64;
    egui::Window::new("Undo Steps").open(&mut open).collapsible(false).resizable(false).show(ui.ctx(), |ui| {
        ui.label("How many changes Ctrl+Z can take back, for each project:");
        ui.add_space(6.0);
        number_slider(ui, &mut steps, MIN_STEPS as f64..=MAX_STEPS as f64, "steps", 180.0, 48.0);
        ui.add_space(6.0);
        ui.label(
            RichText::new("More steps need more memory. The newest steps are always kept.").color(colors::SUBTLE_TEXT),
        );
    });
    let steps = steps.round() as usize;
    if steps != app.project().history().max_steps() {
        for project in &mut app.projects {
            project.history_mut().set_max_steps(steps);
        }
    }
    app.undo_dialog = open;
}
