//! Adjustment layers (C# `LayerCommands.AddAdjustmentLayer / AddLevelsLayer / AddCurvesLayer`): a layer without a
//! picture that changes the colors of everything below it, so nothing in the pictures changes and it can be tuned or
//! deleted at any time.

use fe_core::document::{AdjustmentStyle, Layer};
use fe_core::history::MergeKey;

use crate::app::FlashApp;

fn insert_above_active(app: &mut FlashApp, name: &str, step: &str, style: AdjustmentStyle) {
    let layer = Layer::adjustment(name, style);
    app.project_mut().edit(step, None, |doc| {
        let index = doc.active_index().map_or(doc.layers().len(), |i| i + 1);
        doc.insert_layer(index, layer);
    });
}

pub fn add_basic(app: &mut FlashApp) {
    let style = AdjustmentStyle { contrast: 15.0, saturation: 15.0, ..AdjustmentStyle::default() };
    insert_above_active(app, "Adjustment", "Add Adjustment Layer", style);
    app.status = "Added an adjustment layer. Use the sliders in Properties; it changes every layer below it.".into();
}

pub fn add_levels(app: &mut FlashApp) {
    insert_above_active(app, "Levels", "Add Levels Layer", AdjustmentStyle::new_levels());
    app.status =
        "Added a Levels layer. Move the black and white points and the middle tones in Properties; it changes every layer below it."
            .into();
}

pub fn add_curves(app: &mut FlashApp) {
    insert_above_active(app, "Curves", "Add Curves Layer", AdjustmentStyle::new_curves());
    app.status = "Added a Curves layer. Click the curve in Properties to add a point and drag it; it changes every layer below it."
        .into();
}

/// A change of the active adjustment layer. Slider drags with the same `what` are one undo step.
pub fn change(app: &mut FlashApp, what: &'static str, f: impl FnOnce(&mut AdjustmentStyle)) {
    let Some(id) = app.doc().active_id() else { return };
    app.project_mut().edit(what, Some(MergeKey { layer: id, what }), |doc| {
        if let Some(style) = doc.layer_mut(id).and_then(|l| l.adjustment.as_mut()) {
            f(style);
        }
    });
}
