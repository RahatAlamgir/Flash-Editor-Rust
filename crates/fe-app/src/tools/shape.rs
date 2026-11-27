//! The Shape tool: pick a shape in the options bar, then drag on the picture to draw it (Shift = square / circle, or a line in
//! steps of 15 degrees). A click without dragging puts the shape in the default size around the click. The new shape is selected
//! and the tool goes back to Select / Move, so its handles can be used at once.

use egui::{Color32, Painter, Pos2, Shape, Stroke};
use fe_core::imaging::PointD;
use fe_core::objects::{ObjectBox, ShapeKind};
use fe_gpu::ViewTransform;

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// Screen points the mouse must travel before a press counts as a drag.
const DRAG_START: f64 = 4.0;
/// How thick a drawn line is, in picture pixels.
const LINE_THICKNESS: f64 = 6.0;

/// The Shape tool's state.
pub struct ShapeTool {
    /// What the next drag draws.
    pub kind: ShapeKind,
    /// The drag in progress: where it started and where the mouse is.
    drag: Option<(PointD, PointD)>,
    /// The shape gallery under the options bar is open.
    pub open: bool,
    /// The pictures of the shapes for the gallery.
    pub icons: crate::panels::shape_menu::ShapeIcons,
    shift: bool,
    zoom: f64,
}

impl Default for ShapeTool {
    fn default() -> Self {
        Self { kind: ShapeKind::Rectangle, drag: None, open: false, icons: Default::default(), shift: false, zoom: 1.0 }
    }
}

/// The box a drag from `start` to `end` makes, or `None` while it is still a click.
fn drag_box(kind: ShapeKind, start: PointD, end: PointD, shift: bool, zoom: f64) -> Option<ObjectBox> {
    let (dx, dy) = (end.x - start.x, end.y - start.y);
    if dx.hypot(dy) * zoom < DRAG_START {
        return None;
    }
    if kind == ShapeKind::Line {
        let mut angle = dy.atan2(dx).to_degrees();
        if shift {
            angle = (angle / 15.0).round() * 15.0;
        }
        let length = dx.hypot(dy);
        let rad = angle.to_radians();
        let end = PointD::new(start.x + length * rad.cos(), start.y + length * rad.sin());
        let mid = PointD::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
        return Some(ObjectBox::from_center(mid, length.max(2.0), LINE_THICKNESS, angle));
    }
    let (mut w, mut h) = (dx.abs(), dy.abs());
    if shift {
        w = w.max(h);
        h = w;
    }
    let x = if dx < 0.0 { start.x - w } else { start.x };
    let y = if dy < 0.0 { start.y - h } else { start.y };
    Some(ObjectBox::new(x, y, w.max(2.0), h.max(2.0), 0.0))
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    if tool != ToolKind::Shape {
        app.tools.shape.drag = None;
        return;
    }
    let state = &mut app.tools.shape;
    (state.shift, state.zoom) = (p.mods.shift, p.zoom);
    if p.pressed {
        state.drag = Some((p.pos, p.pos));
    }
    let Some((start, _)) = state.drag else { return };
    state.drag = Some((start, p.pos));
    if p.released || !p.down {
        state.drag = None;
        let kind = state.kind;
        match drag_box(kind, start, p.pos, p.mods.shift, p.zoom) {
            Some(b) => crate::commands::objects::add_shape_box(app, kind, b),
            None => crate::commands::objects::add_shape_at(app, kind, Some(start)),
        }
    }
}

/// Esc while drawing.
pub fn key(app: &mut FlashApp, key: egui::Key) -> bool {
    app.tools.active() == ToolKind::Shape && key == egui::Key::Escape && app.tools.shape.drag.take().is_some()
}

/// The outline of the shape being dragged.
pub fn paint_preview(painter: &Painter, t: ViewTransform, app: &FlashApp) {
    let s = &app.tools.shape;
    let Some((start, end)) = s.drag else { return };
    let Some(b) = drag_box(s.kind, start, end, s.shift, s.zoom) else { return };
    let stroke = Stroke::new(1.5, fe_ui_kit::theme::colors::ACCENT);
    let to_screen = |x: f64, y: f64| -> Pos2 { t.to_screen(x as f32, y as f32) };
    let c = b.center();
    let rad = b.rotation.to_radians();
    let (sin, cos) = (rad.sin(), rad.cos());
    // A point of the box, turned around its center.
    let at = |u: f64, v: f64| {
        let (px, py) = (u - b.width / 2.0, v - b.height / 2.0);
        to_screen(c.x + px * cos - py * sin, c.y + px * sin + py * cos)
    };
    let outline = vec![at(0.0, 0.0), at(b.width, 0.0), at(b.width, b.height), at(0.0, b.height), at(0.0, 0.0)];
    painter.add(Shape::line(outline, stroke));
    if s.kind == ShapeKind::Ellipse {
        let ring: Vec<Pos2> = (0..=48)
            .map(|i| {
                let a = std::f64::consts::TAU * f64::from(i) / 48.0;
                at(b.width / 2.0 * (1.0 + a.cos()), b.height / 2.0 * (1.0 + a.sin()))
            })
            .collect();
        painter.add(Shape::line(ring, Stroke::new(1.5, Color32::from_rgba_unmultiplied(0x3B, 0x82, 0xF6, 0xD0))));
    }
}
