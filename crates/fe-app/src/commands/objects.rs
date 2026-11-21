//! Object commands (C# `InsertCommands.AddShape`, `LayerCommands.Group / Ungroup / FlipLayer / ConvertToPixels`,
//! `AlignCommands`, Properties edits). Each is one undo step.

use fe_core::document::LayerId;
use fe_core::document::object_ops::{self, AlignKind, UngroupError};
use fe_core::history::MergeKey;
use fe_core::imaging::ColorRgba;
use fe_core::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject};

use crate::app::FlashApp;
use crate::tools::ToolKind;

/// Insert > Shapes: a shape in the middle of the picture (blue; a line is black).
pub fn add_shape(app: &mut FlashApp, kind: ShapeKind) {
    use ShapeKind::*;
    let (dw, dh) = (app.doc().width() as f64, app.doc().height() as f64);
    let size = (dw.min(dh) * 0.35).max(40.0);
    let line = kind == Line;
    let wide = matches!(
        kind,
        Rectangle | Parallelogram | Trapezoid | Arrow | DoubleArrow | SpeechBubble | OvalBubble | Cloud | Tag | Home
    );
    let w = if line {
        size * 1.4
    } else if wide {
        size * 1.3
    } else {
        size
    };
    let h = if line { 6.0 } else { size };
    let b = ObjectBox::new((dw - w) / 2.0, (dh - h) / 2.0, w, h, 0.0);
    let mut shape =
        ShapeData::new(kind, (!line).then_some(ColorRgba::rgb(59, 130, 246)), line.then_some(ColorRgba::BLACK), 4.0);
    if kind == Pie {
        shape.inner = 0.0;
    }
    let object = StoredObject::new_shape(b, &shape);
    add_object(app, object, kind.name(), &format!("Add {}", kind.name()));
    app.status = format!(
        "Added a {}. Drag the handles to resize or turn it; change colors and more in Properties.",
        kind.name().to_lowercase()
    );
}

/// Puts an object on a new layer at the very top of the page, selects it and switches to Select / Move.
pub fn add_object(app: &mut FlashApp, object: StoredObject, name: &str, step: &str) {
    add_object_here(app, object, name, step);
    app.tools.activate(ToolKind::Move);
}

/// Puts an object on a new layer at the very top of the page and selects it; the tool stays as it is.
pub fn add_object_here(app: &mut FlashApp, object: StoredObject, name: &str, step: &str) {
    let canvas = app.doc().bounds();
    let layer = object_ops::object_layer(name, object, canvas);
    app.project_mut().edit(step, None, |doc| {
        let top = doc.layers().len();
        doc.insert_layer(top, layer);
        doc.set_selected_area(None);
    });
}

/// Insert > Gradient Background: a rectangle over the whole picture with a gradient, behind everything (change its
/// colors in Properties > Gradient).
pub fn gradient_background(app: &mut FlashApp) {
    let (w, h) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    let look = &fe_core::objects::GRADIENT_PRESETS[1]; // Ocean
    let shape = ShapeData::new(ShapeKind::Rectangle, Some(look.colors[0]), None, 0.0);
    let mut object = StoredObject::new_shape(ObjectBox::new(0.0, 0.0, w, h, 0.0), &shape);
    object.set_gradient(Some(&look.create()));
    let canvas = app.doc().bounds();
    let layer = object_ops::object_layer("Gradient Background", object, canvas);
    app.project_mut().edit("Add Gradient Background", None, |doc| {
        doc.insert_layer(0, layer);
        doc.set_selected_area(None);
    });
    app.status = "Added a gradient background behind everything. Change its colors in Properties > Gradient.".into();
}

/// Ctrl+G: groups the selected layers.
pub fn group(app: &mut FlashApp) {
    let ids = app.doc().selected_ids();
    if ids.iter().any(|id| app.doc().layer(*id).is_some_and(|l| l.locked)) {
        app.status = "A locked layer cannot be grouped. Unlock it first (Ctrl+L).".into();
        return;
    }
    let mut trial = app.doc().clone();
    if object_ops::group(&mut trial, &ids).is_none() {
        app.status = "Select at least two visible, non-empty layers to group (Shift+click on the canvas, or Ctrl+click                       in the Layers panel)."
            .into();
        return;
    }
    app.project_mut().edit("Group", None, |doc| *doc = trial);
    app.status =
        format!("Grouped {} layers. They now move, resize and rotate together. Ctrl+Shift+G ungroups.", ids.len());
}

/// Ctrl+Shift+G: the selected group becomes separate layers again.
pub fn ungroup(app: &mut FlashApp) {
    let Some(id) = app.doc().active_layer().filter(|l| l.object().is_some_and(|o| o.kind() == "group")).map(|l| l.id())
    else {
        app.status = "Select a group first.".into();
        return;
    };
    if app.doc().layer(id).is_some_and(|l| l.locked) {
        app.status = "The group is locked. Unlock it first (Ctrl+L).".into();
        return;
    }
    let doc = app.doc().clone();
    let mut trial = doc.clone();
    match object_ops::ungroup(&mut trial, id) {
        Ok(parts) => {
            app.project_mut().edit("Ungroup", None, |d| *d = trial);
            app.status = format!("Ungrouped into {} layers.", parts.len());
        }
        Err(UngroupError::CannotDrawPart) => {
            app.status = "This group holds a chart or a code: taking it apart comes with charts and codes in a later \
                          step of the Rust version. It can be moved, resized and turned as a whole."
                .into();
        }
        Err(UngroupError::NotAGroup) => app.status = "Select a group first.".into(),
    }
}

/// Group when several layers are selected, ungroup a group.
pub fn toggle_group(app: &mut FlashApp) {
    let is_group = app.doc().active_layer().and_then(|l| l.object()).is_some_and(|o| o.kind() == "group");
    if is_group && app.doc().selected_ids().len() == 1 { ungroup(app) } else { group(app) }
}

/// Flips the selected layers (objects in place, pixels across the canvas).
pub fn flip(app: &mut FlashApp, horizontal: bool) {
    let ids: Vec<LayerId> =
        app.doc().selected_ids().into_iter().filter(|id| app.doc().layer(*id).is_some_and(|l| !l.locked)).collect();
    if ids.is_empty() {
        return;
    }
    let name = if horizontal { "Flip Horizontal" } else { "Flip Vertical" };
    app.project_mut().edit(name, None, |doc| {
        for id in &ids {
            object_ops::flip(doc, *id, horizontal);
        }
    });
}

/// Turns the selected object into plain pixels.
pub fn convert_to_pixels(app: &mut FlashApp) {
    let Some(id) = app.doc().active_layer().filter(|l| l.object().is_some()).map(|l| l.id()) else { return };
    app.project_mut().edit("Convert to Pixels", None, |doc| object_ops::convert_to_pixels(doc, id));
    app.status = "The layer is now plain pixels: paint on it freely. Ctrl+Z makes it editable again.".into();
}

/// The selected, unlocked, visible layers with their areas.
fn movable(app: &FlashApp) -> (Vec<LayerId>, Vec<fe_core::imaging::PixelRect>) {
    let doc = app.doc();
    doc.selected_ids()
        .into_iter()
        .filter_map(|id| doc.layer(id).filter(|l| !l.locked && l.visible && !l.is_adjustment()))
        .map(|l| (l.id(), object_ops::layer_area(l)))
        .unzip()
}

fn apply_moves(app: &mut FlashApp, name: &str, ids: &[LayerId], moves: &[(i32, i32)]) {
    if moves.iter().all(|m| *m == (0, 0)) {
        return;
    }
    app.project_mut().edit(name, None, |doc| {
        for (id, (dx, dy)) in ids.iter().zip(moves) {
            if (*dx, *dy) != (0, 0) {
                object_ops::move_layer(doc, *id, *dx, *dy);
            }
        }
    });
}

/// Lines up the selected layers (one with the canvas, several with each other).
pub fn align(app: &mut FlashApp, kind: AlignKind) {
    let (ids, areas) = movable(app);
    let moves = object_ops::align_moves(&areas, app.doc().bounds(), kind);
    apply_moves(app, "Align", &ids, &moves);
}

/// The 3 x 3 grid of the Align section: both directions at once.
pub fn align_both(app: &mut FlashApp, horizontal: AlignKind, vertical: AlignKind) {
    let (ids, areas) = movable(app);
    let canvas = app.doc().bounds();
    let across = object_ops::align_moves(&areas, canvas, horizontal);
    let down = object_ops::align_moves(&areas, canvas, vertical);
    let moves: Vec<(i32, i32)> = across.iter().zip(&down).map(|(a, d)| (a.0, d.1)).collect();
    apply_moves(app, "Align", &ids, &moves);
}

/// Equal gaps between three or more selected layers.
pub fn distribute(app: &mut FlashApp, horizontal: bool) {
    let (ids, areas) = movable(app);
    if ids.len() < 3 {
        app.status = "Select at least three layers to space them evenly.".into();
        return;
    }
    let moves = object_ops::distribute_moves(&areas, horizontal);
    apply_moves(app, "Distribute", &ids, &moves);
}

/// A Properties change of the selected object (or of the active part of a group). With several layers selected the
/// change goes to all of them in one undo step (a setting that does not fit a kind of object leaves it alone).
/// Slider drags with the same `what` merge into one step.
pub fn change_object(app: &mut FlashApp, what: &'static str, mut change: impl FnMut(&mut StoredObject)) {
    let Some(active) = app.doc().active_id() else { return };
    let part = super::parts::active_part(app);
    let ids = match app.doc().selected_ids() {
        ids if ids.len() > 1 && part.is_none() && ids.contains(&active) => ids,
        _ => vec![active],
    };
    let mut edits = Vec::new();
    for id in ids {
        let Some(layer) = app.doc().layer(id).filter(|l| !l.locked) else { continue };
        let Some(whole) = layer.object().cloned() else { continue };
        let Some(mut target) = (match part {
            Some(i) => whole.part(i).map(|m| m.object),
            None => Some(whole.clone()),
        }) else {
            continue;
        };
        change(&mut target);
        let Some(object) = super::parts::with_target(&whole, part, target) else { continue };
        if object != whole {
            edits.push((id, object));
        }
    }
    if edits.is_empty() {
        return;
    }
    let canvas = app.doc().bounds();
    app.project_mut().edit(what, Some(MergeKey { layer: active, what }), |doc| {
        for (id, object) in edits {
            if let Some(l) = doc.layer_mut(id) {
                object_ops::set_object(l, object, canvas);
            }
        }
    });
}

/// Turns the active pixel layer into a placed picture (so it gets the same Properties as objects).
pub fn make_picture(app: &mut FlashApp) -> bool {
    let Some(id) = app.doc().active_layer().filter(|l| l.object().is_none() && !l.locked).map(|l| l.id()) else {
        return false;
    };
    if app.doc().layer(id).is_some_and(|l| l.visible_bounds().is_empty()) {
        return false;
    }
    app.project_mut().edit("Make Editable", None, |doc| object_ops::make_picture(doc, id))
}
