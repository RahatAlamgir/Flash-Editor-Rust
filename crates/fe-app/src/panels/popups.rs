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
    file_menu(ui, app);
    ribbon_menu(ui, app);
    undo_steps_dialog(ui, app);
}

/// The list under the File button (the sliding File drawer of the C# version comes in Phase 15).
fn file_menu(ui: &mut Ui, app: &mut FlashApp) {
    if !app.file_menu {
        return;
    }
    let anchor = egui::Rect::from_min_size(egui::pos2(6.0, 3.0), egui::vec2(66.0, 24.0));
    let mut open = true;
    let mut chosen = None;
    popup_below(ui, Id::unique("file-menu"), anchor, &mut open, |ui| {
        ui.set_min_width(250.0);
        let items = [
            ("New project...", "Ctrl+N"),
            ("Open...", "Ctrl+O"),
            ("Project Manager", ""),
            ("", ""),
            ("Save", "Ctrl+S"),
            ("Save As...", "Ctrl+Shift+S"),
            ("Save as Template...", ""),
            ("", ""),
            ("Export Picture...", "Ctrl+Alt+S"),
            ("Export For...", ""),
            ("Export All Pages...", ""),
            ("Export as PDF...", ""),
            ("Export as Animated GIF...", ""),
            ("Export as PowerPoint...", ""),
            ("Export as SVG...", ""),
            ("Export as Photoshop (PSD)...", ""),
            ("Batch Convert...", ""),
            ("", ""),
            ("Add Page", "Ctrl+Alt+N"),
            ("Close Project", "Ctrl+W"),
            ("", ""),
            ("Exit", "Alt+F4"),
        ];
        for (text, keys) in items {
            if text.is_empty() {
                ui.separator();
            } else if ui.add(egui::Button::new(text).shortcut_text(keys).frame(false)).clicked() {
                chosen = Some(text);
            }
        }
    });
    if !open || chosen.is_some() {
        app.file_menu = false;
    }
    let ctx = ui.ctx().clone();
    match chosen {
        Some("New project...") => {
            app.dialog = Some(crate::dialogs::Dialog::NewProject(crate::dialogs::NewProjectForm::new(app.background)));
        }
        Some("Open...") => crate::commands::file::open_dialog(app, &ctx),
        Some("Project Manager") => crate::dialogs::project_manager::open(app, &ctx),
        Some("Save") => crate::commands::file::save(app, &ctx),
        Some("Save As...") => {
            let index = app.active_project;
            crate::commands::file::save_as(app, &ctx, index, crate::app::jobs::AfterSave::Nothing);
        }
        Some("Save as Template...") => {
            app.dialog = Some(crate::dialogs::Dialog::SaveTemplate(app.project().name.clone()))
        }
        Some("Export Picture...") => crate::commands::export::open_export(app),
        Some("Export For...") => crate::commands::export::open_export_for(app),
        Some("Export All Pages...") => crate::commands::export::open_export_all(app),
        Some("Export as PDF...") => {
            crate::commands::export::open_pages_export(app, crate::commands::export::PagesKind::Pdf)
        }
        Some("Export as Animated GIF...") => {
            crate::commands::export::open_pages_export(app, crate::commands::export::PagesKind::Gif);
        }
        Some("Export as PowerPoint...") => {
            crate::commands::export::open_pages_export(app, crate::commands::export::PagesKind::Pptx);
        }
        Some("Export as SVG...") => crate::commands::export::export_document(app, true),
        Some("Export as Photoshop (PSD)...") => crate::commands::export::export_document(app, false),
        Some("Batch Convert...") => crate::commands::export::open_batch(app),
        Some("Add Page") => crate::commands::pages::add_after_current(app),
        Some("Close Project") => {
            let index = app.active_project;
            crate::commands::file::close_project(app, index);
        }
        Some("Exit") => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        _ => {}
    }
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
