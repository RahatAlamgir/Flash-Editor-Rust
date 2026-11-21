//! Moving objects along with a change of the whole picture (C# `ObjectTransforms`): flip, turn around the middle.
//! Shadow, inner shadow and 3D directions follow, so the look stays the same.

use super::{Looks, ObjectBox, StoredObject};
use crate::imaging::PointD;

fn change_angles(o: &mut StoredObject, f: impl Fn(f64) -> f64) {
    let mut l: Looks = o.looks();
    if let Some(s) = &mut l.shadow {
        s.angle = ObjectBox::normalize_angle(f(s.angle));
    }
    if let Some(s) = &mut l.inner_shadow {
        s.angle = ObjectBox::normalize_angle(f(s.angle));
    }
    if let Some(s) = &mut l.three_d {
        s.angle = ObjectBox::normalize_angle(f(s.angle));
    }
    if l != o.looks() {
        o.set_looks(&l);
    }
}

/// Mirrors left-right across a canvas `canvas_width` wide.
pub fn flip_horizontal(o: &StoredObject, canvas_width: f64) -> StoredObject {
    let mut out = o.clone();
    if let Some(b) = o.box_() {
        let c = b.center();
        let mut nb = b.with_center(PointD::new(canvas_width - c.x, c.y));
        nb.rotation = -b.rotation;
        out.set_box(nb);
        let (fx, fy) = o.flips();
        out.set_flips(!fx, fy);
    }
    change_angles(&mut out, |a| 180.0 - a);
    out
}

/// Mirrors top-bottom across a canvas `canvas_height` tall.
pub fn flip_vertical(o: &StoredObject, canvas_height: f64) -> StoredObject {
    let mut out = o.clone();
    if let Some(b) = o.box_() {
        let c = b.center();
        let mut nb = b.with_center(PointD::new(c.x, canvas_height - c.y));
        nb.rotation = -b.rotation;
        out.set_box(nb);
        let (fx, fy) = o.flips();
        out.set_flips(fx, !fy);
    }
    change_angles(&mut out, |a| -a);
    out
}

/// Turns around the old canvas middle by `degrees` (clockwise) when the canvas changes from the old size to the new
/// one (the new canvas is centered on the old one).
pub fn rotate(o: &StoredObject, degrees: f64, old: (f64, f64), new: (f64, f64)) -> StoredObject {
    let mut out = o.clone();
    if let Some(b) = o.box_() {
        let (sin, cos) = degrees.to_radians().sin_cos();
        let c = b.center();
        let (dx, dy) = (c.x - old.0 / 2.0, c.y - old.1 / 2.0);
        let center = PointD::new(new.0 / 2.0 + dx * cos - dy * sin, new.1 / 2.0 + dx * sin + dy * cos);
        let mut nb = b.with_center(center);
        nb.rotation = ObjectBox::normalize_angle(b.rotation + degrees);
        out.set_box(nb);
    }
    change_angles(&mut out, |a| a + degrees);
    out
}
