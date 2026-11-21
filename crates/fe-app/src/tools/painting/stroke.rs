//! Brush, Eraser (C# `PaintToolBase`) and Mask Brush (C# `MaskBrushTool`): one stroke per mouse drag, one undo
//! step per stroke. Shift+click draws a straight line from the end of the previous stroke.

use fe_core::document::{Document, LayerId, mask_ops};
use fe_core::paint::stroke::{Effect, MaskStroke, PaintStroke};

use super::super::{Pointer, ToolKind};
use crate::app::FlashApp;
use crate::commands::checks;

enum Kind {
    Paint(PaintStroke),
    Mask(MaskStroke),
}

/// A stroke in progress.
pub struct ActiveStroke {
    kind: Kind,
    layer: LayerId,
    /// The page before the stroke (the undo step goes back to it).
    before: Document,
    name: &'static str,
}

/// Starts a stroke. `None` when the active layer cannot be painted on (the status bar says why).
fn begin(app: &mut FlashApp, tool: ToolKind) -> Option<ActiveStroke> {
    let canvas = app.doc().bounds();
    let selection = app.doc().selected_area().cloned();
    if tool == ToolKind::MaskBrush {
        let Some(layer) = app.doc().active_layer() else {
            app.status = "Select a layer first.".into();
            return None;
        };
        if layer.is_adjustment() {
            app.status = "An adjustment layer has no pixels to mask.".into();
            return None;
        }
        if !layer.visible {
            app.status = "The active layer is hidden. Show it (eye icon) to paint on its mask.".into();
            return None;
        }
        let id = layer.id();
        if layer.mask().is_none() {
            app.project_mut().edit("Add Layer Mask", None, |doc| mask_ops::add(doc, id, true));
        }
        if let Some(l) = app.project_mut().active_page_mut().doc.layer_mut(id) {
            l.mask_enabled = true;
        }
        let grey = MaskStroke::grey_of(app.foreground);
        let brush = app.tools.paint.mask.settings();
        let mask = app.doc().layer(id)?.mask()?.tiles();
        let stroke = MaskStroke::new(mask, brush, grey, selection, canvas);
        return Some(ActiveStroke {
            kind: Kind::Mask(stroke),
            layer: id,
            before: app.doc().clone(),
            name: "Mask Brush",
        });
    }
    let id = checks::editable_layer(app)?;
    let (options, effect, name) = if tool == ToolKind::Eraser {
        (&app.tools.paint.eraser, Effect::Erase, "Eraser")
    } else {
        (&app.tools.paint.brush, Effect::Paint(app.foreground), "Brush")
    };
    let brush = options.settings();
    let stroke = PaintStroke::new(app.doc().layer(id)?.pixels(), brush, effect, selection, canvas);
    Some(ActiveStroke { kind: Kind::Paint(stroke), layer: id, before: app.doc().clone(), name })
}

/// Adds a mouse position to the stroke and updates the layer (or its mask).
fn add(app: &mut FlashApp, stroke: &mut ActiveStroke, at: fe_core::imaging::PointD) {
    let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(stroke.layer) else { return };
    match &mut stroke.kind {
        Kind::Paint(s) => {
            s.add_point(layer.pixels_mut(), at);
        }
        Kind::Mask(s) => {
            if let Some(mask) = layer.mask_mut() {
                s.add_point(mask.tiles_mut(), at);
            }
        }
    }
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    if p.pressed {
        let Some(mut stroke) = begin(app, tool) else { return };
        if p.mods.shift
            && let Some(previous) = app.tools.paint.last_end
        {
            add(app, &mut stroke, previous);
        }
        add(app, &mut stroke, p.pos);
        app.tools.paint.stroke = Some(stroke);
    }
    let Some(mut stroke) = app.tools.paint.stroke.take() else { return };
    if !p.pressed {
        add(app, &mut stroke, p.pos);
    }
    if p.released || !p.down {
        app.tools.paint.last_end = Some(p.pos);
        app.project_mut().record(stroke.name, stroke.before);
    } else {
        app.tools.paint.stroke = Some(stroke);
    }
}
