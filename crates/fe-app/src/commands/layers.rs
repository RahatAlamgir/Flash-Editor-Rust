//! Layer commands (Layers panel, Layer tab, right-click menu, shortcuts). Each change is one undo step.

use fe_core::document::{LayerId, layer_ops};
use fe_core::history::MergeKey;
use fe_core::imaging::BlendMode;

use crate::app::FlashApp;

pub fn new_layer(app: &mut FlashApp) {
    app.project_mut().edit("New Layer", None, layer_ops::new_layer);
}

pub fn duplicate(app: &mut FlashApp) {
    let count = app.doc().selected_ids().len();
    let name = if count > 1 { "Duplicate Layers" } else { "Duplicate Layer" };
    app.project_mut().edit(name, None, layer_ops::duplicate_selected);
}

pub fn delete(app: &mut FlashApp) {
    let deletable =
        app.doc().selected_ids().iter().filter(|id| app.doc().layer(**id).is_some_and(|l| !l.locked)).count();
    if deletable == 0 || app.doc().layers().len() <= 1 {
        app.status = "The last layer, and locked layers, cannot be deleted.".into();
        return;
    }
    let name = if deletable > 1 { "Delete Layers" } else { "Delete Layer" };
    app.project_mut().edit(name, None, layer_ops::delete_selected);
}

/// `direction`: +1 = up (forward), -1 = down (backward).
pub fn move_layer(app: &mut FlashApp, direction: i32) {
    let index = app.doc().active_index().unwrap_or(0) as i32;
    let target = index + direction;
    if target < 0 || target >= app.doc().layers().len() as i32 {
        return;
    }
    let name = if direction > 0 { "Move Layer Up" } else { "Move Layer Down" };
    app.project_mut().edit(name, None, |doc| layer_ops::move_active(doc, direction));
}

pub fn bring_to_end(app: &mut FlashApp, front: bool) {
    let index = app.doc().active_index().unwrap_or(0);
    let target = if front { app.doc().layers().len() - 1 } else { 0 };
    if index == target {
        return;
    }
    let name = if front { "Bring to Front" } else { "Send to Back" };
    app.project_mut().edit(name, None, |doc| layer_ops::move_active_to_end(doc, front));
}

/// Drops the dragged layers into a gap of the Layers panel list.
pub fn move_to_gap(app: &mut FlashApp, moved: Vec<LayerId>, gap: usize) {
    let order: Vec<LayerId> = app.doc().layers().iter().map(|l| l.id()).collect();
    if fe_core::document::layer_order::move_to_gap(&order, &moved, gap).is_none() {
        return;
    }
    let name = if moved.len() > 1 { "Move Layers" } else { "Move Layer" };
    app.project_mut().edit(name, None, |doc| layer_ops::move_to_gap(doc, &moved, gap));
}

/// Merge Down, or Merge Layers when several layers are selected (Ctrl+E).
pub fn merge(app: &mut FlashApp) {
    if app.doc().selected_ids().len() >= 2 {
        merge_selected(app);
    } else if app.doc().active_index().is_some_and(|i| i > 0) {
        app.project_mut().edit("Merge Down", None, layer_ops::merge_down);
    }
}

pub fn merge_selected(app: &mut FlashApp) {
    if layer_ops::merge_candidates(app.doc()).len() < 2 {
        app.status =
            "Select at least two visible, unlocked layers to merge (Ctrl+click or Shift+click in the Layers panel)."
                .into();
        return;
    }
    let count = app.project_mut().edit("Merge Layers", None, layer_ops::merge_selected);
    app.status =
        format!("Merged {count} layers into one. Ctrl+Z undoes it. (Group, Ctrl+G, keeps objects editable instead.)");
}

pub fn flatten(app: &mut FlashApp) {
    if app.doc().layers().len() > 1 {
        app.project_mut().edit("Flatten Image", None, layer_ops::flatten);
    }
}

pub fn set_lock_of_selected(app: &mut FlashApp, locked: bool) {
    let doc = app.doc();
    let to_change = doc.selected_ids().iter().filter(|id| doc.layer(**id).is_some_and(|l| l.locked != locked)).count();
    let verb = if locked { "Locked" } else { "Unlocked" };
    if to_change == 0 {
        app.status = format!("{verb} 0 layers: they already were.");
        return;
    }
    let name = if locked { "Lock Layers" } else { "Unlock Layers" };
    let changed = app.project_mut().edit(name, None, |doc| layer_ops::set_lock_of_selected(doc, locked));
    app.status = format!("{verb} {changed} layer(s).");
}

/// Lock / Unlock (Ctrl+L): locks the selected layers, or unlocks them when they are all locked.
pub fn toggle_lock(app: &mut FlashApp) {
    let all_locked = app.doc().selected_ids().iter().all(|id| app.doc().layer(*id).is_some_and(|l| l.locked));
    set_lock_of_selected(app, !all_locked);
}

pub fn select_all(app: &mut FlashApp) {
    let page = app.project_mut().active_page_mut();
    let ids: Vec<LayerId> = page.doc.layers().iter().map(|l| l.id()).collect();
    let active = page.doc.active_id();
    page.doc.set_selection(&ids, active);
}

pub fn set_visible(app: &mut FlashApp, id: LayerId, visible: bool) {
    let name = if visible { "Show Layer" } else { "Hide Layer" };
    app.project_mut().edit(name, None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            l.visible = visible;
        }
    });
}

pub fn set_locked(app: &mut FlashApp, id: LayerId, locked: bool) {
    let name = if locked { "Lock Layer" } else { "Unlock Layer" };
    app.project_mut().edit(name, None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            l.locked = locked;
        }
    });
}

pub fn rename(app: &mut FlashApp, id: LayerId, name: String) {
    let name = name.trim().to_owned();
    if name.is_empty() || app.doc().layer(id).is_some_and(|l| l.name == name) {
        return;
    }
    app.project_mut().edit("Rename Layer", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            l.name = name;
        }
    });
}

/// Opacity 0..1 of the active layer. A slider drag becomes one undo step.
pub fn set_opacity(app: &mut FlashApp, opacity: f64) {
    let Some(id) = app.doc().active_id() else { return };
    let merge = Some(MergeKey { layer: id, what: "opacity" });
    app.project_mut().edit("Layer Opacity", merge, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            l.set_opacity(opacity);
        }
    });
}

pub fn set_blend(app: &mut FlashApp, mode: BlendMode) {
    let Some(id) = app.doc().active_id() else { return };
    if app.doc().layer(id).is_some_and(|l| l.blend == mode) {
        return;
    }
    app.project_mut().edit("Blend Mode", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            l.blend = mode;
        }
    });
}
