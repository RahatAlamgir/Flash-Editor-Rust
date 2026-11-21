//! Editing one part inside a group (C# `EditTarget`, `MainViewModel.SelectPart`): double-click a group to go
//! inside it, then the handles, Properties and Delete work on the part. Esc goes back to the whole group.

use fe_core::document::LayerId;
use fe_core::objects::StoredObject;

use crate::app::FlashApp;

/// The part being edited: its index in the active layer's group, or `None` (the whole layer is edited).
pub fn active_part(app: &FlashApp) -> Option<usize> {
    active_part_of(&app.tools, app.doc())
}

/// The same, from the tools and the document (for code that holds a borrow of the document).
pub fn active_part_of(tools: &crate::tools::ToolState, doc: &fe_core::document::Document) -> Option<usize> {
    let (id, index) = tools.mover.part?;
    let layer = doc.active_layer().filter(|l| l.id() == id)?;
    let count = layer.object().and_then(StoredObject::group_members).map_or(0, |m| m.len());
    (index < count).then_some(index)
}

/// Goes inside a group (or to another part), or back out with `None`.
pub fn select_part(app: &mut FlashApp, part: Option<(LayerId, usize)>) {
    app.tools.mover.part = part;
    app.tools.mover.reshaping = false;
}

/// The object Properties and the handles work on: the active part of a group, or the layer's own object.
pub fn target_object(app: &FlashApp) -> Option<StoredObject> {
    let layer = app.doc().active_layer()?;
    let object = layer.object()?;
    match active_part(app) {
        Some(i) => object.part(i).map(|m| m.object),
        None => Some(object.clone()),
    }
}

/// The layer's object after the part (when one is active) was replaced by `changed`.
pub fn with_target(whole: &StoredObject, part: Option<usize>, changed: StoredObject) -> Option<StoredObject> {
    match part {
        Some(i) => whole.with_part(i, changed),
        None => Some(changed),
    }
}

/// Delete while a part is selected: the part goes, the group stays (a group's last two parts stay a group).
pub fn delete_part(app: &mut FlashApp) -> bool {
    let Some(index) = active_part(app) else { return false };
    let Some(layer) = app.doc().active_layer().filter(|l| !l.locked) else { return true };
    let (id, Some(group)) = (layer.id(), layer.object().and_then(|o| o.without_part(index))) else {
        app.status = "A group needs at least one part. Delete the whole group instead.".into();
        return true;
    };
    let canvas = app.doc().bounds();
    app.project_mut().edit("Delete Part", None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            fe_core::document::object_ops::set_object(l, group, canvas);
        }
    });
    select_part(app, None);
    true
}
