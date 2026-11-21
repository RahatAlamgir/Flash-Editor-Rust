//! Pen (C# `PenTool`): draw a smooth outline with curves, then turn it into a selection. Click = corner point,
//! click and drag = smooth point. Click the first point, double-click or press Enter to select the shape.
//! Backspace removes the last point, Esc cancels. Shift on the first click = add, Alt = subtract.

use egui::{Key, Modifiers};
use fe_core::imaging::PointD;
use fe_core::selection::Selection;
use fe_core::selection::bezier::{Anchor, flatten_cubic, path_points};

use super::{Preview, apply};
use crate::app::FlashApp;
use crate::tools::Pointer;

/// How close the path follows the curves when it becomes a selection (C#: 0.25 px).
const FLATNESS: f64 = 0.25;

/// A pen path being drawn.
#[derive(Default)]
pub struct Pen {
    anchors: Vec<Anchor>,
    /// The button is held after placing a point: dragging pulls out its curve handle.
    dragging: bool,
    mouse: PointD,
    start_mods: Modifiers,
}

impl Pen {
    pub fn cancel(&mut self) {
        self.anchors.clear();
        self.dragging = false;
    }

    pub fn is_drawing(&self) -> bool {
        !self.anchors.is_empty()
    }

    /// The path so far, with a curve to the mouse while not dragging.
    pub fn preview(&self) -> Option<Preview> {
        let last = self.anchors.last()?;
        let mut points = path_points(&self.anchors, false, 0.5);
        if !self.dragging {
            flatten_cubic(last.point, last.handle_out(), self.mouse, self.mouse, 0.5, &mut points);
        }
        Some(Preview::Path { points, closed: false })
    }

    /// The points and their handles (drawn as small squares and circles).
    pub fn anchors(&self) -> &[Anchor] {
        &self.anchors
    }
}

fn finish(app: &mut FlashApp) {
    let pen = &mut app.tools.select.pen;
    let anchors = std::mem::take(&mut pen.anchors);
    pen.dragging = false;
    let mods = pen.start_mods;
    if anchors.len() < 2 {
        return;
    }
    let points = path_points(&anchors, true, FLATNESS);
    let (w, h) = (app.doc().width(), app.doc().height());
    let drawn = if points.len() >= 3 { Selection::from_polygon(&points, w, h) } else { None };
    apply::apply(app, drawn, mods);
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    let pen = &mut app.tools.select.pen;
    pen.mouse = p.pos;
    if p.double_click && pen.is_drawing() {
        finish(app);
        return;
    }
    if p.pressed {
        if !pen.is_drawing() {
            pen.start_mods = p.mods;
        }
        if pen.anchors.len() >= 2 && pen.anchors[0].point.distance_to(&p.pos) <= 8.0 / p.zoom {
            finish(app);
            return;
        }
        // The second press of a double-click lands on the last point: it finishes on release instead.
        if pen.anchors.last().is_some_and(|a| a.point.distance_to(&p.pos) <= 2.0 / p.zoom) {
            return;
        }
        pen.anchors.push(Anchor::corner(p.pos));
        pen.dragging = true;
    } else if pen.dragging
        && p.down
        && let Some(last) = pen.anchors.last_mut()
    {
        last.out = (last.point.distance_to(&p.pos) > 2.0 / p.zoom).then_some(p.pos);
    }
    if p.released {
        pen.dragging = false;
    }
    if pen.is_drawing() {
        app.status = format!("{} point(s). Click the first point (or Enter) to finish.", pen.anchors.len());
    }
}

/// Enter, Esc and Backspace while a path is drawn. True when the key was used.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    let pen = &mut app.tools.select.pen;
    if !pen.is_drawing() {
        return false;
    }
    match key {
        Key::Enter => finish(app),
        Key::Escape => {
            pen.cancel();
            app.status = "Pen path cancelled.".into();
        }
        Key::Backspace => {
            pen.anchors.pop();
        }
        _ => return false,
    }
    true
}
