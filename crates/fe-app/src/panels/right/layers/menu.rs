//! The right-click menu of a layer row (C# `LayersPanel.ContextMenu`). Items for groups, rotating and flipping
//! join with their phases.

use egui::Ui;
use fe_core::document::LayerId;

use crate::app::FlashApp;
use crate::commands::layers as cmd;

/// What the menu needs to know about the clicked layer.
pub struct MenuInfo {
    pub visible: bool,
    pub locked: bool,
    pub selected_count: usize,
    pub can_merge_down: bool,
    pub has_mask: bool,
    /// Not an adjustment layer and there is a picture layer below.
    pub can_clip: bool,
}

/// A menu choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuItem {
    ToggleVisible,
    ToggleLock,
    Rename,
    Duplicate,
    Merge,
    BringToFront,
    BringForward,
    SendBackward,
    SendToBack,
    ClipBelow,
    AddMask,
    DeleteMask,
    Delete,
}

/// Draws the menu; returns the choice (the menu closes itself on a click).
pub fn show(ui: &mut Ui, info: &MenuInfo) -> Option<MenuItem> {
    let mut chosen = None;
    let mut item = |ui: &mut Ui, text: &str, shortcut: &str, value: MenuItem, enabled: bool| {
        let button = egui::Button::new(text).shortcut_text(shortcut);
        if ui.add_enabled(enabled, button).clicked() {
            chosen = Some(value);
            ui.close();
        }
    };
    ui.set_min_width(220.0);
    item(ui, if info.visible { "Hide" } else { "Show" }, "", MenuItem::ToggleVisible, true);
    item(ui, if info.locked { "Unlock" } else { "Lock" }, "Ctrl+L", MenuItem::ToggleLock, true);
    item(ui, "Rename", "F2", MenuItem::Rename, true);
    ui.separator();
    item(ui, "Duplicate", "Ctrl+D", MenuItem::Duplicate, true);
    if info.selected_count >= 2 {
        item(ui, &format!("Merge {} Layers", info.selected_count), "Ctrl+E", MenuItem::Merge, true);
    } else {
        item(ui, "Merge Down", "Ctrl+E", MenuItem::Merge, info.can_merge_down);
    }
    ui.separator();
    item(ui, "Bring to Front", "Ctrl+Shift+]", MenuItem::BringToFront, true);
    item(ui, "Bring Forward", "Ctrl+]", MenuItem::BringForward, true);
    item(ui, "Send Backward", "Ctrl+[", MenuItem::SendBackward, true);
    item(ui, "Send to Back", "Ctrl+Shift+[", MenuItem::SendToBack, true);
    ui.separator();
    item(ui, "Clip to Layer Below", "", MenuItem::ClipBelow, info.can_clip);
    if info.has_mask {
        item(ui, "Delete Layer Mask", "", MenuItem::DeleteMask, true);
    } else {
        item(ui, "Add Layer Mask", "", MenuItem::AddMask, true);
    }
    ui.separator();
    item(ui, "Delete", "", MenuItem::Delete, !info.locked);
    chosen
}

/// Runs a menu choice on the clicked layer (already made active by the caller).
pub fn run(app: &mut FlashApp, id: LayerId, item: MenuItem) {
    match item {
        MenuItem::ToggleVisible => {
            let visible = app.doc().layer(id).is_some_and(|l| l.visible);
            cmd::set_visible(app, id, !visible);
        }
        MenuItem::ToggleLock => cmd::toggle_lock(app),
        MenuItem::Rename => super::start_rename_active(app),
        MenuItem::Duplicate => cmd::duplicate(app),
        MenuItem::Merge => cmd::merge(app),
        MenuItem::BringToFront => cmd::bring_to_end(app, true),
        MenuItem::BringForward => cmd::move_layer(app, 1),
        MenuItem::SendBackward => cmd::move_layer(app, -1),
        MenuItem::SendToBack => cmd::bring_to_end(app, false),
        MenuItem::ClipBelow => crate::commands::mask::clip_to_below(app),
        MenuItem::AddMask => crate::commands::mask::add(app, true),
        MenuItem::DeleteMask => crate::commands::mask::delete(app),
        MenuItem::Delete => cmd::delete(app),
    }
}
