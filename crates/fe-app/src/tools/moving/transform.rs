//! Resizing, turning and reshaping the selected layer with its handles (C# `MoveTool.BeginObjectDrag`,
//! `Dragged`, `WithMovedCorner`). A plain pixel layer first becomes a placed picture, so it can be resized again
//! later without losing quality ("every layer has the same handles"). Each drag is one undo step.

use fe_core::document::{Document, LayerId, object_ops};
use fe_core::imaging::{PointD, TileMap};
use fe_core::objects::handles::{self, Handle};
use fe_core::objects::{ObjectBox, StoredObject};

use crate::app::FlashApp;
use crate::tools::Pointer;

/// A resize, turn or reshape in progress.
pub struct TransformDrag {
    layer: LayerId,
    before: Document,
    /// The object being dragged: the layer's own, or the part of a group being edited.
    start: StoredObject,
    /// The layer's whole object (the group, when a part is dragged).
    whole: StoredObject,
    part: Option<usize>,
    /// Where the mouse went down (for moving a part).
    origin: PointD,
    start_pixels: TileMap,
    handle: Handle,
    /// Set while a Reshape corner is dragged.
    corner: Option<usize>,
    snap: Option<fe_core::layout::snapping::SnapSession>,
}

/// The frame of a layer: its object's box, or the area of its visible pixels.
pub fn frame_of(doc: &Document, id: LayerId) -> Option<ObjectBox> {
    let layer = doc.layer(id).filter(|l| l.visible && !l.is_adjustment())?;
    if let Some(b) = layer.object().and_then(StoredObject::box_) {
        return Some(b);
    }
    let r = layer.visible_bounds();
    (!r.is_empty()).then(|| ObjectBox::new(r.x as f64, r.y as f64, r.width as f64, r.height as f64, 0.0))
}

/// The frame of one part of a group, or of the layer when `part` is `None`.
pub fn frame_of_part(doc: &Document, id: LayerId, part: Option<usize>) -> Option<ObjectBox> {
    let Some(index) = part else { return frame_of(doc, id) };
    let layer = doc.layer(id).filter(|l| l.visible)?;
    layer.object()?.part(index)?.object.box_()
}

/// Starts a drag of `handle` (or of Reshape corner `corner`) on layer `id`. `None` when nothing can be dragged.
pub fn begin(
    app: &mut FlashApp,
    id: LayerId,
    handle: Handle,
    corner: Option<usize>,
    origin: PointD,
) -> Option<TransformDrag> {
    let before = app.doc().clone();
    let doc = &mut app.project_mut().active_page_mut().doc;
    if doc.layer(id)?.object().is_none() && !object_ops::make_picture(doc, id) {
        return None;
    }
    let snap = crate::tools::snap::session(app, &[id]);
    let part = crate::commands::parts::active_part(app).filter(|_| app.doc().active_id() == Some(id));
    let layer = app.doc().layer(id)?;
    let whole = layer.object()?.clone();
    let start = match part {
        Some(i) => whole.part(i)?.object,
        None => whole.clone(),
    };
    Some(TransformDrag {
        snap,
        layer: id,
        before,
        start,
        whole,
        part,
        origin,
        start_pixels: layer.pixels().clone(),
        handle,
        corner,
    })
}

/// The object as the mouse has dragged it so far.
fn dragged(app: &mut FlashApp, d: &TransformDrag, p: &Pointer) -> StoredObject {
    if let Some(index) = d.corner {
        app.status = "Reshape: drag the corner points. Shift snaps them. Esc ends Reshape.".into();
        return handles::with_moved_corner(&d.start, index, p.pos, p.mods.shift);
    }
    let start_box = d.start.box_().unwrap_or_default();
    let mut moved = d.start.clone();
    if d.handle == Handle::Move {
        let (mut dx, mut dy) = ((p.pos.x - d.origin.x).round(), (p.pos.y - d.origin.y).round());
        if p.mods.shift {
            if dx.abs() > dy.abs() { dy = 0.0 } else { dx = 0.0 }
        }
        app.status = format!("Move part: {dx:+.0}, {dy:+.0} px");
        moved.set_box(start_box.offset(dx, dy));
        return moved;
    }
    if d.handle == Handle::Rotate {
        let angle = handles::rotation_for(&start_box, p.pos, p.mods.shift);
        app.status = format!("Rotation: {angle:.0}°");
        moved.set_box(ObjectBox { rotation: angle, ..start_box });
        return moved;
    }
    // Pictures and text keep their shape by default; the Properties switch keeps shapes' too. Shift does the opposite.
    let keep = (handles::keeps_aspect(&d.start) || app.tools.mover.keep_proportions) != p.mods.shift;
    let at = crate::tools::snap::snap_point(app, d.snap.as_ref(), p.pos, p.zoom, p.mods.alt);
    let mut b = handles::resize(&start_box, d.handle, at, keep);
    if handles::is_line_like(Some(&d.start)) {
        b = ObjectBox::from_center(b.center(), b.width, start_box.height, b.rotation);
    }
    app.status = format!("{:.0} x {:.0} px", b.width, b.height);
    moved.set_box(b);
    // Text: the font size follows the height so the text stays sharp.
    if d.start.kind() == "text"
        && let Some(size) = d.start.json.get("FontSize").and_then(serde_json::Value::as_f64)
        && let Some(map) = moved.json.as_object_mut()
    {
        let ratio = b.height / start_box.height.max(1e-9);
        map.insert("FontSize".into(), fe_core::objects::json::number((size * ratio).max(1.0)));
    }
    moved
}

/// Follows the mouse; at release the drag becomes one undo step.
pub fn pointer(app: &mut FlashApp, mut d: TransformDrag, p: &Pointer) -> Option<TransformDrag> {
    let part_object = dragged(app, &d, p);
    let object = crate::commands::parts::with_target(&d.whole, d.part, part_object)?;
    let canvas = app.doc().bounds();
    let pixels = object_ops::drawing_for(&object, Some((&d.start_pixels, &d.whole)), canvas);
    if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(d.layer) {
        layer.set_pixels_with_object(pixels, Some(object));
    }
    if p.released || !p.down {
        let name = match (d.corner, d.handle) {
            (Some(_), _) => "Reshape",
            (_, Handle::Rotate) => "Rotate",
            (_, Handle::Move) => "Move",
            _ => "Resize",
        };
        crate::tools::snap::end(app);
        let before = std::mem::replace(&mut d.before, Document::new(1, 1));
        app.project_mut().record(name, before);
        return None;
    }
    Some(d)
}

/// The Reshape points of the active layer's object on the canvas.
pub fn reshape_points(doc: &Document) -> Vec<PointD> {
    let Some(obj) = doc.active_layer().and_then(|l| l.object()) else { return Vec::new() };
    handles::reshape_points(obj).unwrap_or_default().iter().map(|u| fe_core::render::draw::from_unit(obj, *u)).collect()
}
