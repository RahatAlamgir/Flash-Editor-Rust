//! Layer > Layer Mask (C# `MaskCommands`): hide or show parts of a layer without erasing them. Each command is
//! one undo step; the status bar says what happened.

use fe_core::document::mask_ops;

use crate::app::FlashApp;
use crate::tools::ToolKind;

/// The active layer, if it can have a mask (not an adjustment layer).
fn maskable(app: &mut FlashApp) -> Option<fe_core::document::LayerId> {
    let Some(layer) = app.doc().active_layer() else {
        app.status = "Select a layer first.".into();
        return None;
    };
    if layer.is_adjustment() {
        app.status = "An adjustment layer has no pixels to mask.".into();
        return None;
    }
    Some(layer.id())
}

/// Add Mask: Show All / Hide All. Switches to the Mask Brush.
pub fn add(app: &mut FlashApp, reveal: bool) {
    let Some(id) = maskable(app) else { return };
    if app.doc().layer(id).is_some_and(|l| l.mask().is_some()) {
        app.status = "This layer already has a mask. Paint on it with the Mask Brush (K), or delete it first.".into();
        return;
    }
    app.project_mut().edit("Add Layer Mask", None, |doc| mask_ops::add(doc, id, reveal));
    app.status = if reveal {
        "Added a mask that shows everything. Paint on it with the Mask Brush (K): black hides, white shows.".into()
    } else {
        "Added a mask that hides everything. Paint on it with the Mask Brush (K) in white to show parts.".into()
    };
    app.tools.activate(ToolKind::MaskBrush);
}

/// Shows only the selected area (a new mask), or hides the selected area (also on an existing mask).
pub fn from_selection(app: &mut FlashApp, reveal_selected: bool) {
    let Some(selection) = app.doc().selected_area().cloned() else {
        app.status = "Select an area first.".into();
        return;
    };
    let Some(id) = maskable(app) else { return };
    let name = if reveal_selected { "Mask: Show Selection Only" } else { "Mask: Hide Selection" };
    app.project_mut().edit(name, None, |doc| mask_ops::from_selection(doc, id, &selection, reveal_selected));
    app.status = if reveal_selected {
        "The layer now shows only the selected area.".into()
    } else {
        "Hid the selected area of the layer (its pixels are still there).".into()
    };
}

/// Runs a command on the active layer's mask; says so when there is no mask.
fn with_mask(
    app: &mut FlashApp,
    name: &str,
    change: fn(&mut fe_core::document::Document, fe_core::document::LayerId) -> bool,
) -> bool {
    let Some(id) = app.doc().active_layer().filter(|l| l.mask().is_some()).map(|l| l.id()) else {
        app.status = "This layer has no mask. Add one with Layer > Layer Mask.".into();
        return false;
    };
    app.project_mut().edit(name, None, |doc| change(doc, id));
    true
}

pub fn invert(app: &mut FlashApp) {
    with_mask(app, "Invert Layer Mask", mask_ops::invert);
}

pub fn toggle(app: &mut FlashApp) {
    let on = app.doc().active_layer().is_some_and(|l| l.mask_enabled);
    with_mask(app, if on { "Switch Mask Off" } else { "Switch Mask On" }, mask_ops::toggle);
}

pub fn delete(app: &mut FlashApp) {
    if with_mask(app, "Delete Layer Mask", mask_ops::delete) {
        app.status = "Deleted the mask. The layer shows all its pixels again.".into();
    }
}

pub fn apply(app: &mut FlashApp) {
    if with_mask(app, "Apply Layer Mask", mask_ops::apply) {
        app.status = "Applied the mask: the hidden parts are now transparent pixels. Ctrl+Z undoes it.".into();
    }
}

/// Clip to Layer Below: the layer shows only inside the shape of the layer under it.
pub fn clip_to_below(app: &mut FlashApp) {
    let Some(id) = maskable(app) else { return };
    let index = app.doc().index_of(id).unwrap_or(0);
    if index == 0 || app.doc().layers()[index - 1].is_adjustment() {
        app.status = "There is no picture layer below this one to clip to.".into();
        return;
    }
    let names = app.project_mut().edit("Clip to Layer Below", None, |doc| mask_ops::clip_to_below(doc, id));
    if let Some((layer, below)) = names {
        app.status = format!(
            "\"{layer}\" now shows only inside \"{below}\". Ctrl+Z undoes it; the layer's pixels are not changed."
        );
    }
}
