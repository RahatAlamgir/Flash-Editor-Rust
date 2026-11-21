//! Checks before a command changes a layer (C# `TryGetEditableLayer`, `TryGetVisibleLayer`): they say in the
//! status bar why nothing happens.

use fe_core::document::LayerId;

use crate::app::FlashApp;

/// The active layer if there is one and it is visible.
pub fn visible_layer(app: &mut FlashApp) -> Option<LayerId> {
    let Some(layer) = app.doc().active_layer() else {
        app.status = "Select a layer first.".into();
        return None;
    };
    if !layer.visible {
        app.status = "The active layer is hidden. Show it (eye icon in the Layers panel) to edit it.".into();
        return None;
    }
    Some(layer.id())
}

/// The active layer if its pixels may be changed: visible, not locked, not an adjustment layer.
/// (Changing the pixels of an object layer turns it into plain pixels, like in C#.)
pub fn editable_layer(app: &mut FlashApp) -> Option<LayerId> {
    let id = visible_layer(app)?;
    let layer = app.doc().layer(id)?;
    if layer.locked {
        app.status = format!(
            "\"{}\" is locked. Unlock it with the lock icon in the Layers panel, right-click > Unlock, or Ctrl+L.",
            layer.name
        );
        return None;
    }
    if layer.is_adjustment() {
        app.status = "This is an adjustment layer: it changes the colors of the layers below. Change its sliders in \
                      Properties, or paint on another layer."
            .into();
        return None;
    }
    Some(id)
}
