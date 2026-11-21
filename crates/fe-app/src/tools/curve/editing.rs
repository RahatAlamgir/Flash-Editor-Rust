//! Editing a curve that already exists: drag points and handles, make a point smooth or sharp, add and delete points.

use fe_core::document::{Document, LayerId, object_ops};
use fe_core::imaging::{PointD, TileMap};
use fe_core::objects::curve::{fit, is_smooth, moved, nearest, nodes_to_canvas, split, toggle_smooth};
use fe_core::objects::{CurveNode, ShapeKind, StoredObject};

use crate::app::FlashApp;
use crate::tools::Pointer;

/// A curve layer that can be edited.
pub struct Target {
    pub id: LayerId,
    pub object: StoredObject,
    pub closed: bool,
    /// The points in canvas pixels (the shape may be moved, turned or mirrored).
    pub nodes: Vec<CurveNode>,
}

/// A point or handle being dragged. `part`: 0 = the point, 1 = its handle in, 2 = its handle out.
pub struct Grab {
    id: LayerId,
    before: Document,
    start: StoredObject,
    start_pixels: TileMap,
    node: usize,
    part: u8,
    moved: bool,
}

/// The active layer as an editable curve (an unlocked, visible Path shape with at least two points).
pub fn target(app: &FlashApp) -> Option<Target> {
    if crate::commands::parts::active_part(app).is_some() {
        return None;
    }
    let layer = app.doc().active_layer().filter(|l| !l.locked && l.visible)?;
    let object = layer.object()?;
    let shape = object.shape().filter(|s| s.kind == ShapeKind::Path)?;
    let nodes = shape.nodes.filter(|n| n.len() >= 2)?;
    let canvas = nodes_to_canvas(&object.box_()?, object.flips(), &nodes);
    Some(Target { id: layer.id(), object: object.clone(), closed: shape.closed, nodes: canvas })
}

/// The shape with new points (given in canvas pixels): its box is fitted around them again and turning / mirroring
/// is baked in.
fn rebuild(old: &StoredObject, canvas_nodes: &[CurveNode]) -> Option<StoredObject> {
    let mut shape = old.shape()?;
    let (b, unit) = fit(canvas_nodes, shape.closed);
    shape.nodes = Some(unit);
    let mut object = old.clone();
    object.set_shape(&shape);
    object.set_box(b);
    object.set_flips(false, false);
    object.set_warp(None);
    Some(object)
}

fn commit(app: &mut FlashApp, t: &Target, nodes: &[CurveNode], name: &str) {
    let Some(object) = rebuild(&t.object, nodes) else { return };
    let (id, canvas) = (t.id, app.doc().bounds());
    app.project_mut().edit(name, None, |doc| {
        if let Some(l) = doc.layer_mut(id) {
            object_ops::set_object(l, object, canvas);
        }
    });
}

fn reach(p: &Pointer) -> f64 {
    8.0 / p.zoom
}

/// The place on the curve under the mouse (segment, position along it), if close enough.
pub fn near_line(t: &Target, p: &Pointer) -> Option<(usize, f64)> {
    nearest(&t.nodes, t.closed, p.pos).filter(|(_, _, d)| *d <= reach(p)).map(|(s, along, _)| (s, along))
}

/// A press on a point or handle of the curve starts dragging it. True when it did.
pub fn start(app: &mut FlashApp, t: &Target, p: &Pointer) -> bool {
    let r = reach(p);
    let selected = app.tools.curve.selected.filter(|(id, i)| *id == t.id && *i < t.nodes.len()).map(|(_, i)| i);
    // The handles of the selected point come first (they can lie on top of other points).
    if let Some(i) = selected {
        let n = &t.nodes[i];
        if n.has_out() && n.handle_out.distance_to(&p.pos) <= r {
            return grab(app, t, i, 2);
        }
        if n.has_in() && n.handle_in.distance_to(&p.pos) <= r {
            return grab(app, t, i, 1);
        }
    }
    match t.nodes.iter().rposition(|n| n.point.distance_to(&p.pos) <= r) {
        Some(i) => grab(app, t, i, 0),
        None => false,
    }
}

fn grab(app: &mut FlashApp, t: &Target, node: usize, part: u8) -> bool {
    let before = app.doc().clone();
    let Some(layer) = app.doc().layer(t.id) else { return false };
    let g = Grab {
        id: t.id,
        before,
        start: t.object.clone(),
        start_pixels: layer.pixels().clone(),
        node,
        part,
        moved: false,
    };
    app.tools.curve.selected = Some((t.id, node));
    app.tools.curve.grab = Some(g);
    true
}

/// The place on the other side of the point for a smooth join: opposite to `handle`, keeping its own `length`.
fn opposite(point: PointD, handle: PointD, length: f64) -> PointD {
    let (dx, dy) = (point.x - handle.x, point.y - handle.y);
    let d = (dx * dx + dy * dy).sqrt();
    if d < 1e-9 { point } else { PointD::new(point.x + dx / d * length, point.y + dy / d * length) }
}

/// While a point or handle is dragged: moves it with the mouse; at release the change is one undo step.
pub fn follow(app: &mut FlashApp, p: &Pointer) {
    let Some(mut g) = app.tools.curve.grab.take() else { return };
    let Some(shape) = g.start.shape() else { return };
    let (Some(b), Some(unit)) = (g.start.box_(), shape.nodes) else { return };
    let mut nodes = nodes_to_canvas(&b, g.start.flips(), &unit);
    let n = nodes[g.node];
    if g.part == 0 {
        nodes[g.node] = moved(&n, p.pos.x - n.point.x, p.pos.y - n.point.y);
    } else {
        let smooth = is_smooth(&n) && !p.mods.alt; // Alt bends one side only
        let other = if g.part == 1 { n.handle_out } else { n.handle_in };
        let length = n.point.distance_to(&other);
        let far = if smooth { opposite(n.point, p.pos, length) } else { other };
        nodes[g.node] = if g.part == 1 {
            CurveNode { point: n.point, handle_in: p.pos, handle_out: far }
        } else {
            CurveNode { point: n.point, handle_in: far, handle_out: p.pos }
        };
    }
    g.moved |= nodes[g.node] != n;
    if g.moved
        && let Some(object) = rebuild(&g.start, &nodes)
    {
        let canvas = app.doc().bounds();
        let pixels = object_ops::drawing_for(&object, Some((&g.start_pixels, &g.start)), canvas);
        if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(g.id) {
            layer.set_pixels_with_object(pixels, Some(object));
        }
    }
    if p.released || !p.down {
        if g.moved {
            let before = std::mem::replace(&mut g.before, Document::new(1, 1));
            app.project_mut().record("Edit Curve", before);
        }
    } else {
        app.tools.curve.grab = Some(g);
    }
}

/// A double-click on a point makes it smooth or sharp; on the line it adds a point.
pub fn double_click(app: &mut FlashApp, t: &Target, p: &Pointer) {
    let r = reach(p);
    if let Some(i) = t.nodes.iter().rposition(|n| n.point.distance_to(&p.pos) <= r) {
        app.tools.curve.selected = Some((t.id, i));
        let nodes = toggle_smooth(&t.nodes, t.closed, i);
        commit(app, t, &nodes, "Smooth Curve Point");
    } else if let Some((segment, along)) = near_line(t, p) {
        app.tools.curve.selected = Some((t.id, segment + 1));
        let nodes = split(&t.nodes, segment, along);
        commit(app, t, &nodes, "Add Curve Point");
    }
}

/// Delete removes the selected point (a line keeps two, a shape three).
pub fn delete_point(app: &mut FlashApp) -> bool {
    let Some(t) = target(app) else { return false };
    let Some(i) = app.tools.curve.selected.filter(|(id, i)| *id == t.id && *i < t.nodes.len()).map(|(_, i)| i) else {
        return false;
    };
    let minimum = if t.closed { 3 } else { 2 };
    if t.nodes.len() <= minimum {
        app.status = format!("A curve needs at least {minimum} points.");
        return true;
    }
    let mut nodes = t.nodes.clone();
    nodes.remove(i);
    app.tools.curve.selected = Some((t.id, i.min(nodes.len() - 1)));
    commit(app, &t, &nodes, "Delete Curve Point");
    true
}
