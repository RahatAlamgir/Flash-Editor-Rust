//! Drawing a new curve: placing points, pulling out handles, finishing it as a shape.

use fe_core::objects::curve::{fit, smooth_node};
use fe_core::objects::{CurveNode, ObjectBox, ShapeData, ShapeKind, StoredObject};

use crate::app::FlashApp;
use crate::tools::Pointer;

pub fn press(app: &mut FlashApp, p: &Pointer) {
    let zoom = p.zoom;
    let c = &mut app.tools.curve;
    if c.drawing.len() >= 3 && c.drawing[0].point.distance_to(&p.pos) <= 9.0 / zoom {
        finish(app, true);
        return;
    }
    // The second press of a double-click lands on the last point: the double-click finishes the line instead.
    if c.drawing.last().is_some_and(|n| n.point.distance_to(&p.pos) <= 2.0 / zoom) {
        return;
    }
    c.drawing.push(CurveNode::corner(p.pos));
    c.dragging = true;
}

/// Dragging after a click pulls the handles of the point just placed.
pub fn pull_handle(app: &mut FlashApp, p: &Pointer) {
    let c = &mut app.tools.curve;
    if let Some(last) = c.drawing.last_mut() {
        let at = last.point;
        *last = if at.distance_to(&p.pos) > 3.0 / p.zoom { smooth_node(at, p.pos) } else { CurveNode::corner(at) };
    }
}

/// The curve becomes a shape on a new layer (closed = filled shape, open = a line).
pub fn finish(app: &mut FlashApp, closed: bool) {
    let nodes = std::mem::take(&mut app.tools.curve.drawing);
    app.tools.curve.dragging = false;
    if nodes.len() < if closed { 3 } else { 2 } {
        return;
    }
    let (b, unit) = fit(&nodes, closed);
    let color = fe_core::imaging::ColorRgba { a: 255, ..app.foreground };
    let mut shape = ShapeData::new(
        ShapeKind::Path,
        closed.then_some(color),
        (!closed).then_some(color),
        if closed { 0.0 } else { 4.0 },
    );
    shape.nodes = Some(unit);
    shape.closed = closed;
    let object = StoredObject::new_shape(ObjectBox::new(b.x, b.y, b.width, b.height, 0.0), &shape);
    let name = if closed { "Curve" } else { "Line" };
    crate::commands::objects::add_object_here(app, object, name, if closed { "Add Curve" } else { "Add Line" });
    app.tools.curve.selected = None;
    app.status = if closed {
        "Curve added. Drag its points to reshape it; change fill and border in Properties."
    } else {
        "Line added. Drag its points to reshape it; change color and width in Properties."
    }
    .into();
}
