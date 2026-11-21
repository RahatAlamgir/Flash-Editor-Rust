//! The right-click menu on the canvas (C# `CanvasView.ContextMenu`, the parts that exist so far): clipboard,
//! duplicate, split, delete pixels, layer order, select layer content and deselect.

use egui::Response;

use crate::app::FlashApp;
use crate::commands::{layers, objects, selection};
use fe_core::document::object_ops::AlignKind;

/// Shows the menu on right-click and runs what was picked.
pub fn show(response: &Response, app: &mut FlashApp) {
    let has_selection = app.doc().selected_area().is_some();
    let selected = app.doc().selected_ids().len();
    let active_object = app.doc().active_layer().and_then(|l| l.object()).map(|o| o.kind().to_owned());
    let mut picked: Option<&'static str> = None;
    response.context_menu(|ui| {
        ui.set_min_width(220.0);
        let mut item = |ui: &mut egui::Ui, text: &'static str, keys: &str| {
            if ui.add(egui::Button::new(text).shortcut_text(keys).frame(false)).clicked() {
                picked = Some(text);
                ui.close();
            }
        };
        item(ui, if has_selection { "Cut Selection" } else { "Cut" }, "Ctrl+X");
        item(ui, if has_selection { "Copy Selection" } else { "Copy" }, "Ctrl+C");
        item(ui, "Paste", "Ctrl+V");
        item(ui, "Duplicate", "Ctrl+D");
        if selected >= 2 {
            item(ui, "Group", "Ctrl+G");
        }
        if active_object.as_deref() == Some("group") {
            item(ui, "Ungroup", "Ctrl+Shift+G");
        }
        if has_selection {
            item(ui, "Split to New Layer", "");
            item(ui, "Delete Selected Pixels", "Delete");
        }
        ui.separator();
        item(ui, "Bring to Front", "Ctrl+Shift+]");
        item(ui, "Bring Forward", "Ctrl+]");
        item(ui, "Send Backward", "Ctrl+[");
        item(ui, "Send to Back", "Ctrl+Shift+[");
        ui.menu_button("Align", |ui| {
            for text in ["Left", "Center", "Right", "Top", "Middle", "Bottom"] {
                item(ui, text, "");
            }
            if selected > 2 {
                ui.separator();
                item(ui, "Distribute Horizontally", "");
                item(ui, "Distribute Vertically", "");
            }
        });
        ui.separator();
        item(ui, "Flip Horizontal", "");
        item(ui, "Flip Vertical", "");
        if active_object.is_some() {
            item(ui, "Convert to Pixels", "");
        }
        ui.separator();
        item(ui, "Select Layer Content", "");
        if has_selection {
            item(ui, "Invert Selection", "Ctrl+Shift+I");
            item(ui, "Deselect", "Ctrl+Shift+D");
        }
    });
    match picked {
        Some("Cut Selection" | "Cut") => selection::copy(app, true),
        Some("Copy Selection" | "Copy") => selection::copy(app, false),
        Some("Paste") => selection::paste(app),
        Some("Duplicate") => layers::duplicate(app),
        Some("Split to New Layer") => selection::split(app),
        Some("Delete Selected Pixels") => {
            selection::delete_pixels(app);
        }
        Some("Bring to Front") => layers::bring_to_end(app, true),
        Some("Bring Forward") => layers::move_layer(app, 1),
        Some("Send Backward") => layers::move_layer(app, -1),
        Some("Send to Back") => layers::bring_to_end(app, false),
        Some("Select Layer Content") => selection::layer_content(app),
        Some("Group") => objects::group(app),
        Some("Ungroup") => objects::ungroup(app),
        Some("Flip Horizontal") => objects::flip(app, true),
        Some("Flip Vertical") => objects::flip(app, false),
        Some("Convert to Pixels") => objects::convert_to_pixels(app),
        Some("Left") => objects::align(app, AlignKind::Left),
        Some("Center") => objects::align(app, AlignKind::CenterHorizontally),
        Some("Right") => objects::align(app, AlignKind::Right),
        Some("Top") => objects::align(app, AlignKind::Top),
        Some("Middle") => objects::align(app, AlignKind::CenterVertically),
        Some("Bottom") => objects::align(app, AlignKind::Bottom),
        Some("Distribute Horizontally") => objects::distribute(app, true),
        Some("Distribute Vertically") => objects::distribute(app, false),
        Some("Invert Selection") => selection::invert(app),
        Some("Deselect") => selection::deselect(app),
        _ => {}
    }
}
