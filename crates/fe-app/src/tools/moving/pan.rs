//! Moving the picture inside a frame (C# `MoveTool.PicturePanned`): double-click a filled frame, then drag to move
//! the picture and use the mouse wheel to zoom it. Esc or a click elsewhere is done.

use fe_core::document::{Document, LayerId};
use fe_core::imaging::PointD;
use fe_core::objects::StoredObject;

use crate::app::FlashApp;
use crate::commands::objects::change_object;
use crate::tools::Pointer;

/// A picture being dragged inside its frame.
pub struct PanDrag {
    layer: LayerId,
    before: Document,
    start: StoredObject,
    origin: PointD,
}

/// The frame under the mouse that has a picture (the layer to edit).
pub fn begin(app: &mut FlashApp, id: LayerId, origin: PointD) -> Option<PanDrag> {
    let layer = app.doc().layer(id).filter(|l| !l.locked)?;
    let start = layer.object()?.clone();
    start.picture_fill()?;
    Some(PanDrag { layer: id, before: app.doc().clone(), start, origin })
}

/// The new pan of the picture for a mouse at `mouse` (the picture follows the pointer).
fn panned(d: &PanDrag, mouse: PointD) -> Option<(f64, f64)> {
    let frame = d.start.box_()?;
    let fill = d.start.picture_fill()?;
    let image = fe_core::objects::pictures::decoded(d.start.picture()?)?;
    let (a, b) = (frame.to_local(d.origin), frame.to_local(mouse));
    let (fx, fy) = d.start.flips();
    let dx = (b.x - a.x) * if fx { -1.0 } else { 1.0 };
    let dy = (b.y - a.y) * if fy { -1.0 } else { 1.0 };
    let (_, _, w, h) = fill.place(f64::from(image.width()), f64::from(image.height()), frame.width, frame.height);
    let (room_x, room_y) = ((w - frame.width).abs() / 2.0, (h - frame.height).abs() / 2.0);
    let pan_x = if room_x > 1e-6 { (fill.pan_x + dx / room_x).clamp(-1.0, 1.0) } else { fill.pan_x };
    let pan_y = if room_y > 1e-6 { (fill.pan_y + dy / room_y).clamp(-1.0, 1.0) } else { fill.pan_y };
    Some((pan_x, pan_y))
}

/// Follows the mouse; at release the drag becomes one undo step.
pub fn follow(app: &mut FlashApp, mut d: PanDrag, p: &Pointer) -> Option<PanDrag> {
    if let Some((pan_x, pan_y)) = panned(&d, p.pos) {
        let zoom = d.start.picture_fill().map_or(1.0, |f| f.zoom);
        let mut object = d.start.clone();
        object.set_picture_view(zoom, pan_x, pan_y);
        let canvas = app.doc().bounds();
        if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(d.layer) {
            fe_core::document::object_ops::set_object(layer, object, canvas);
        }
    }
    if p.released || !p.down {
        let before = std::mem::replace(&mut d.before, Document::new(1, 1));
        app.project_mut().record("Move Picture in Frame", before);
        return None;
    }
    Some(d)
}

/// The mouse wheel over a frame being edited zooms the picture (a notch = 10 %).
pub fn wheel(app: &mut FlashApp, notches: f32) {
    let factor = 1.1f64.powf(f64::from(notches));
    change_object(app, "Zoom Picture in Frame", |o| {
        if let Some(f) = o.picture_fill() {
            o.set_picture_view(f.zoom * factor, f.pan_x, f.pan_y);
        }
    });
}
