//! The Curve tool (C# `CurveTool`): draw a shape or line made of Bezier curves, and edit it afterwards.
//! Click = a corner point; click and drag = a smooth point (the drag pulls out the curve handles). Click the first
//! point to close the shape (it gets a fill); double-click or Enter ends an open line. Backspace removes the last
//! point, Esc cancels. With a curve selected: drag its points and handles, double-click a point to make it smooth
//! or sharp, double-click the line to add a point, Delete removes the selected point, Alt bends one side only.
//! The result is a normal shape: fill, border, dashes, gradient and the other settings work on it.

mod drawing;
mod editing;
mod overlay;

use egui::Key;
use fe_core::document::LayerId;
use fe_core::imaging::PointD;
use fe_core::objects::CurveNode;

pub use overlay::{CurveOverlay, overlay};

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// The Curve tool's state.
#[derive(Default)]
pub struct CurveTool {
    /// The curve being drawn, in canvas pixels.
    drawing: Vec<CurveNode>,
    /// The button is held after placing a point: dragging pulls out its handles.
    dragging: bool,
    mouse: PointD,
    /// The selected point of the curve being edited.
    selected: Option<(LayerId, usize)>,
    /// A point or handle being dragged.
    grab: Option<editing::Grab>,
}

impl CurveTool {
    pub fn cancel(&mut self) {
        self.drawing.clear();
        self.dragging = false;
        self.grab = None;
    }

    pub fn is_drawing(&self) -> bool {
        !self.drawing.is_empty()
    }
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    if tool != ToolKind::Curve {
        return;
    }
    app.tools.curve.mouse = p.pos;
    if p.pressed {
        press(app, p);
    }
    editing::follow(app, p);
    if app.tools.curve.dragging {
        if p.down && !p.pressed {
            drawing::pull_handle(app, p);
        }
        if p.released || !p.down {
            app.tools.curve.dragging = false;
        }
    }
    if p.double_click {
        double_click(app, p);
    }
    if app.tools.curve.is_drawing() {
        let n = app.tools.curve.drawing.len();
        app.status = format!(
            "{n} point{}. Click the first point to close the shape, double-click or Enter to finish a line.",
            if n == 1 { "" } else { "s" }
        );
    }
}

fn press(app: &mut FlashApp, p: &Pointer) {
    if !app.tools.curve.is_drawing()
        && let Some(target) = editing::target(app)
    {
        if editing::start(app, &target, p) {
            return;
        }
        // On the line itself a double-click adds a point: no new curve starts there.
        if editing::near_line(&target, p).is_some() {
            return;
        }
    }
    drawing::press(app, p);
}

fn double_click(app: &mut FlashApp, p: &Pointer) {
    if app.tools.curve.is_drawing() {
        drawing::finish(app, false);
    } else if let Some(target) = editing::target(app) {
        editing::double_click(app, &target, p);
    }
}

/// Enter, Esc, Backspace while drawing; Delete removes the selected point. True when the key was used.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    if app.tools.active() != ToolKind::Curve {
        return false;
    }
    if app.tools.curve.is_drawing() {
        match key {
            Key::Enter => drawing::finish(app, false),
            Key::Escape => {
                app.tools.curve.cancel();
                app.status = "Curve cancelled.".into();
            }
            Key::Backspace => {
                app.tools.curve.drawing.pop();
            }
            _ => return false,
        }
        return true;
    }
    if key == Key::Delete {
        return editing::delete_point(app);
    }
    false
}
