//! Lasso (C# `LassoTool`): drag around an area, releasing closes the shape. Polygon Lasso (C# `PolygonLassoTool`):
//! click corner after corner; double-click, Enter or a click on the first point closes it, Backspace removes the
//! last point, Esc cancels. Shift when starting = add, Alt = subtract.

use egui::{Key, Modifiers};
use fe_core::imaging::PointD;
use fe_core::selection::Selection;

use super::{Preview, apply};
use crate::app::FlashApp;
use crate::tools::Pointer;

/// The points of a lasso being drawn.
#[derive(Default)]
pub struct Lasso {
    points: Vec<PointD>,
    /// Freehand: the button is held. Polygon: corners have been placed.
    drawing: bool,
    start_mods: Modifiers,
    /// Where the mouse is (the polygon's rubber band line).
    mouse: PointD,
}

impl Lasso {
    pub fn cancel(&mut self) {
        self.points.clear();
        self.drawing = false;
    }

    pub fn preview(&self, polygon: bool) -> Option<Preview> {
        if !self.drawing || self.points.is_empty() {
            return None;
        }
        let mut points = self.points.clone();
        if polygon {
            points.push(self.mouse);
        }
        Some(Preview::Path { points, closed: true })
    }

    fn take(&mut self) -> (Vec<PointD>, Modifiers) {
        let points = std::mem::take(&mut self.points);
        self.drawing = false;
        (points, self.start_mods)
    }
}

fn finish(app: &mut FlashApp, points: Vec<PointD>, mods: Modifiers) {
    let (w, h) = (app.doc().width(), app.doc().height());
    let drawn = if points.len() >= 3 { Selection::from_polygon(&points, w, h) } else { None };
    apply::apply(app, drawn, mods);
}

/// Freehand lasso.
pub fn freehand(app: &mut FlashApp, p: &Pointer) {
    let l = &mut app.tools.select.lasso;
    if p.pressed {
        l.points = vec![p.pos];
        l.drawing = true;
        l.start_mods = p.mods;
        return;
    }
    if !l.drawing {
        return;
    }
    // Skip points closer than about one screen pixel to keep the polygon small.
    if l.points.last().is_none_or(|last| last.distance_to(&p.pos) >= 1.0 / p.zoom) {
        l.points.push(p.pos);
    }
    if p.released {
        let (points, mods) = l.take();
        finish(app, points, mods);
    }
}

/// Polygon lasso.
pub fn polygon(app: &mut FlashApp, p: &Pointer) {
    let l = &mut app.tools.select.lasso;
    l.mouse = p.pos;
    if p.double_click && l.drawing {
        let (points, mods) = l.take();
        finish(app, points, mods);
        return;
    }
    if !p.pressed {
        if l.drawing {
            app.status = format!("{} point(s). Double-click or Enter to finish.", l.points.len());
        }
        return;
    }
    if !l.drawing {
        l.start_mods = p.mods;
        l.drawing = true;
    }
    let close = 8.0 / p.zoom;
    if l.points.len() >= 3 && l.points[0].distance_to(&p.pos) <= close {
        let (points, mods) = l.take();
        finish(app, points, mods);
        return;
    }
    // The second press of a double-click lands on the last point: it finishes on release instead.
    if l.points.last().is_none_or(|last| last.distance_to(&p.pos) > 2.0 / p.zoom) {
        l.points.push(p.pos);
    }
}

/// Enter, Esc and Backspace while a polygon is drawn. True when the key was used.
pub fn polygon_key(app: &mut FlashApp, key: Key) -> bool {
    let l = &mut app.tools.select.lasso;
    if !l.drawing {
        return false;
    }
    match key {
        Key::Enter => {
            let (points, mods) = l.take();
            finish(app, points, mods);
        }
        Key::Escape => l.cancel(),
        Key::Backspace => {
            l.points.pop();
            if l.points.is_empty() {
                l.cancel();
            }
        }
        _ => return false,
    }
    true
}
