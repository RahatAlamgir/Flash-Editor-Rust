//! The mouse while text is edited: a click puts the cursor there, dragging selects, a double-click picks a word, a
//! click outside the text finishes.

use fe_core::imaging::PointD;

use super::{TextEditor, canvas_point, finish, frame};
use crate::app::FlashApp;
use crate::tools::Pointer;

/// How far from a cursor place (in screen points) a click still counts as on the text.
const REACH: f64 = 8.0;

fn distance_to_segment(p: PointD, a: PointD, b: PointD) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 < 1e-9 { 0.0 } else { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) };
    p.distance_to(&PointD::new(a.x + t * dx, a.y + t * dy))
}

/// The place between letters nearest to a point on the canvas.
pub(super) fn index_at(app: &FlashApp, ed: &TextEditor, pos: PointD) -> Option<usize> {
    let (b, flips) = frame(app, ed)?;
    ed.carets
        .iter()
        .map(|c| {
            let (a, bb) = (canvas_point(&b, flips, ed.natural, c.a), canvas_point(&b, flips, ed.natural, c.b));
            (c.index, distance_to_segment(pos, a, bb))
        })
        .min_by(|x, y| x.1.total_cmp(&y.1))
        .map(|(index, _)| index)
}

/// True when the mouse is over the text box.
pub(super) fn inside(app: &FlashApp, ed: &TextEditor, pos: PointD, zoom: f64) -> bool {
    frame(app, ed).is_some_and(|(b, _)| b.contains(pos, REACH / zoom))
}

/// Gives the mouse to the editor. False when the click was outside the text: editing has finished and the click goes
/// on to the active tool.
pub fn pointer(app: &mut FlashApp, p: &Pointer) -> bool {
    let Some(mut ed) = app.text_edit.take() else { return false };
    let mut consumed = true;
    if p.pressed {
        if inside(app, &ed, p.pos, p.zoom) {
            if let Some(index) = index_at(app, &ed, p.pos) {
                ed.cursor.set_caret(index, p.mods.shift);
            }
            ed.drag = true;
            ed.last_input = -1.0; // the cursor shows right after a click (set to the time by `keys`)
        } else {
            app.text_edit = Some(ed);
            finish(app);
            return false;
        }
    } else if ed.drag {
        if p.down
            && let Some(index) = index_at(app, &ed, p.pos)
        {
            ed.cursor.set_caret(index, true);
        }
        if p.released || !p.down {
            ed.drag = false;
        }
    } else {
        consumed = p.down;
    }
    if p.double_click
        && inside(app, &ed, p.pos, p.zoom)
        && let Some(index) = index_at(app, &ed, p.pos)
    {
        ed.cursor.select_word(index);
    }
    app.text_edit = Some(ed);
    consumed
}
