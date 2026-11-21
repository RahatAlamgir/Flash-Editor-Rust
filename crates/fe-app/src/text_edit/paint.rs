//! Draws the selection and the blinking cursor of the text being edited.

use egui::{Color32, Painter, Shape, Stroke, pos2};
use fe_gpu::ViewTransform;
use fe_ui_kit::theme::colors;

use super::{TextEditor, canvas_point, frame};
use crate::app::FlashApp;

/// How long the cursor stays on, then off, in seconds.
const BLINK: f64 = 0.53;

/// Paints over the canvas. `time` is the program time in seconds.
pub fn paint(painter: &Painter, t: ViewTransform, app: &FlashApp, time: f64) {
    let Some(ed) = &app.text_edit else { return };
    let Some((b, flips)) = frame(app, ed) else { return };
    let at = |p: (f64, f64)| {
        let c = canvas_point(&b, flips, ed.natural, p);
        t.to_screen(c.x as f32, c.y as f32)
    };
    // A thin frame shows where the text box is.
    let corners: Vec<egui::Pos2> = b.corners().iter().map(|c| t.to_screen(c.x as f32, c.y as f32)).collect();
    painter.add(Shape::closed_line(corners, Stroke::new(1.0, colors::SELECTION_ACCENT)));
    paint_selection(painter, ed, &at);
    // The cursor shows while the user types or moves it, then blinks.
    let since = (time - ed.last_input).max(0.0);
    if ((since / BLINK) as i64) % 2 == 0
        && let Some(c) = ed.carets.get(ed.cursor.caret)
    {
        let (top, bottom) = (at(c.a), at(c.b));
        let width = (t.zoom * 1.5).clamp(1.5, 4.0);
        painter.line_segment([top, bottom], Stroke::new(width + 2.0, Color32::from_black_alpha(150)));
        painter.line_segment([top, bottom], Stroke::new(width, Color32::WHITE));
    }
}

/// A translucent highlight behind the selected letters, line by line.
fn paint_selection(painter: &Painter, ed: &TextEditor, at: &impl Fn((f64, f64)) -> egui::Pos2) {
    if !ed.cursor.has_selection() {
        return;
    }
    let (lo, hi) = ed.cursor.range();
    let color = colors::SELECTION_ACCENT.gamma_multiply(0.45);
    for i in lo..hi {
        let (Some(c), Some(d)) = (ed.carets.get(i), ed.carets.get(i + 1)) else { continue };
        if c.line == d.line {
            let quad = vec![at(c.a), at(d.a), at(d.b), at(c.b)];
            painter.add(Shape::convex_polygon(quad, color, Stroke::NONE));
        } else {
            // A selected line break: a small highlight at the end of the line.
            let (a, bb) = (at(c.a), at(c.b));
            let w = ((bb.y - a.y).abs() * 0.3).max(4.0);
            painter.add(Shape::convex_polygon(
                vec![a, pos2(a.x + w, a.y), pos2(bb.x + w, bb.y), bb],
                color,
                Stroke::NONE,
            ));
        }
    }
}
