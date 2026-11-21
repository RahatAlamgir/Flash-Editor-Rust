//! The box with handles around a selected object (C# `TransformHandles`): where the handles are, which one is
//! under the mouse, and the new box after dragging a resize or rotate handle. Also the Reshape corner points.

use super::geometry_box::ObjectBox;
use super::{ShapeKind, StoredObject};
use crate::imaging::PointD;

/// Handle size and rotate-handle distance, in screen points.
pub const HANDLE_SIZE: f64 = 10.0;
pub const ROTATE_DISTANCE: f64 = 28.0;

/// A handle of the box (or the box itself).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handle {
    Move,
    NW,
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    Rotate,
}

impl Handle {
    /// The direction of a resize handle (-1, 0 or 1 on each axis) in box space.
    pub fn direction(self) -> (i32, i32) {
        match self {
            Handle::NW => (-1, -1),
            Handle::N => (0, -1),
            Handle::NE => (1, -1),
            Handle::E => (1, 0),
            Handle::SE => (1, 1),
            Handle::S => (0, 1),
            Handle::SW => (-1, 1),
            Handle::W => (-1, 0),
            _ => (0, 0),
        }
    }

    pub fn is_resize(self) -> bool {
        !matches!(self, Handle::Move | Handle::Rotate)
    }
}

/// True for a straight line (only its two ends can be dragged).
pub fn is_line_like(obj: Option<&StoredObject>) -> bool {
    obj.and_then(StoredObject::shape).is_some_and(|s| s.is_line())
}

/// The handles shown for an object: a line gets its ends, text its corners, everything else all eight.
pub fn handles_for(obj: Option<&StoredObject>) -> &'static [Handle] {
    use Handle::*;
    if is_line_like(obj) {
        &[W, E]
    } else if obj.is_some_and(|o| o.kind() == "text") {
        &[NW, NE, SE, SW]
    } else {
        &[NW, N, NE, E, SE, S, SW, W]
    }
}

/// Where a handle is on the canvas (the rotate handle sits above the box, at a fixed screen distance).
pub fn position(b: &ObjectBox, handle: Handle, zoom: f64) -> PointD {
    if handle == Handle::Rotate {
        return b.to_canvas(0.0, -b.height / 2.0 - ROTATE_DISTANCE / zoom);
    }
    let (sx, sy) = handle.direction();
    b.to_canvas(sx as f64 * b.width / 2.0, sy as f64 * b.height / 2.0)
}

/// Which handle (or the box itself) is under the point; `None` outside.
pub fn hit_test(b: &ObjectBox, obj: Option<&StoredObject>, p: PointD, zoom: f64) -> Option<Handle> {
    let radius = (HANDLE_SIZE / 2.0 + 3.0) / zoom;
    if position(b, Handle::Rotate, zoom).distance_to(&p) <= radius + 2.0 / zoom {
        return Some(Handle::Rotate);
    }
    if let Some(h) = handles_for(obj).iter().copied().find(|h| position(b, *h, zoom).distance_to(&p) <= radius) {
        return Some(h);
    }
    let margin = if is_line_like(obj) { 6.0 } else { 2.0 } / zoom;
    b.contains(p, margin).then_some(Handle::Move)
}

/// The box after dragging a resize handle to `mouse`; the opposite side stays where it was. `keep_ratio` keeps
/// the width / height ratio (corners only).
pub fn resize(start: &ObjectBox, handle: Handle, mouse: PointD, keep_ratio: bool) -> ObjectBox {
    let (sx, sy) = handle.direction();
    let local = start.to_local(mouse);
    let (mut w, mut h) = (start.width, start.height);
    if sx != 0 {
        w = (sx as f64 * local.x + start.width / 2.0).max(1.0);
    }
    if sy != 0 {
        h = (sy as f64 * local.y + start.height / 2.0).max(1.0);
    }
    if keep_ratio && sx != 0 && sy != 0 {
        let scale = (w / start.width).max(h / start.height);
        w = (start.width * scale).max(1.0);
        h = (start.height * scale).max(1.0);
    }
    let center = start.to_canvas(sx as f64 * (w - start.width) / 2.0, sy as f64 * (h - start.height) / 2.0);
    ObjectBox::from_center(center, w, h, start.rotation)
}

/// The rotation for the rotate handle at `mouse`; `snap` = 15 degree steps.
pub fn rotation_for(b: &ObjectBox, mouse: PointD, snap: bool) -> f64 {
    let c = b.center();
    let mut angle = (mouse.y - c.y).atan2(mouse.x - c.x).to_degrees() + 90.0;
    if snap {
        angle = (angle / 15.0).round() * 15.0;
    }
    ObjectBox::normalize_angle(angle)
}

/// The mouse cursor direction for a resize handle (0 = left-right, 1 = NW-SE, 2 = up-down, 3 = NE-SW).
pub fn cursor_octant(handle: Handle, rotation: f64) -> i32 {
    let (sx, sy) = handle.direction();
    let angle = (sy as f64).atan2(sx as f64).to_degrees() + rotation;
    ((((angle % 180.0) + 180.0) % 180.0 / 45.0).round() as i32) % 4
}

/// A point of the canvas as a place in the object (0, 0 = top-left of its box, 1, 1 = bottom-right).
pub fn to_unit(obj: &StoredObject, p: PointD) -> PointD {
    let b = obj.box_().unwrap_or_default();
    let (fx, fy) = obj.flips();
    let l = b.to_local(p);
    let (x, y) = (if fx { -l.x } else { l.x }, if fy { -l.y } else { l.y });
    PointD::new(x / b.width.max(1e-9) + 0.5, y / b.height.max(1e-9) + 0.5)
}

const DEFAULT_CORNERS: [PointD; 4] =
    [PointD::new(0.0, 0.0), PointD::new(1.0, 0.0), PointD::new(1.0, 1.0), PointD::new(0.0, 1.0)];

/// The points Reshape drags: a straight-sided shape's own corners, otherwise the object's four corners. `None`
/// for a line.
pub fn reshape_points(obj: &StoredObject) -> Option<Vec<PointD>> {
    if let Some(shape) = obj.shape() {
        if shape.is_line() {
            return None;
        }
        if shape.kind.is_polygonal() {
            let b = obj.box_().unwrap_or_default();
            return shape.vertices.clone().or_else(|| crate::render::geometry::vertices(&shape, &b));
        }
    }
    Some(obj.warp().unwrap_or_else(|| DEFAULT_CORNERS.to_vec()))
}

/// The object with one reshape point moved to `mouse` (C# `WithMovedCorner`). Points may go a few box sizes
/// outside the box; `snap` rounds to a coarse grid.
pub fn with_moved_corner(obj: &StoredObject, index: usize, mouse: PointD, snap: bool) -> StoredObject {
    let Some(mut points) = reshape_points(obj) else { return obj.clone() };
    if index >= points.len() {
        return obj.clone();
    }
    let u = to_unit(obj, mouse);
    let (mut x, mut y) = (u.x.clamp(-3.0, 4.0), u.y.clamp(-3.0, 4.0));
    if snap {
        x = (x * 20.0).round() / 20.0;
        y = (y * 20.0).round() / 20.0;
    }
    points[index] = PointD::new(x, y);
    let mut changed = obj.clone();
    match obj.shape().filter(|s| s.kind.is_polygonal()) {
        Some(mut shape) => {
            shape.vertices = Some(points);
            shape.corner_radius = 0.0;
            changed.set_shape(&shape);
        }
        None => changed.set_warp(Some(&points)),
    }
    changed
}

/// The object's normal form back (no custom corners).
pub fn without_reshape(obj: &StoredObject) -> StoredObject {
    let mut plain = obj.clone();
    plain.set_warp(None);
    if let Some(mut shape) = obj.shape() {
        shape.vertices = None;
        plain.set_shape(&shape);
    }
    plain
}

/// True once the corners were dragged.
pub fn has_custom_outline(obj: &StoredObject) -> bool {
    obj.warp().is_some() || obj.shape().is_some_and(|s| s.vertices.is_some())
}

/// True when corner handles keep the width / height ratio by default (pictures, text, groups; not shapes).
pub fn keeps_aspect(obj: &StoredObject) -> bool {
    obj.kind() != "shape"
}

/// A shape is a line or open curve with arrow heads etc.; used by callers that need the kind quickly.
pub fn shape_kind(obj: &StoredObject) -> Option<ShapeKind> {
    obj.shape().map(|s| s.kind)
}
