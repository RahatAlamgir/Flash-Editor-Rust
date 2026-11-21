//! Editing text right on the canvas, like Canva or Office: double-click text (or click it with the Text tool) and
//! type. A blinking cursor, selecting with the mouse or Shift + arrows, Ctrl+A / C / X / V, Esc or a click outside
//! to finish. The text object is updated while typing; the whole edit is one undo step.

mod input;
mod mouse;
pub mod paint;

use fe_core::document::{Document, LayerId, object_ops};
use fe_core::imaging::PointD;
use fe_core::objects::curve::to_canvas;
use fe_core::objects::{ObjectBox, StoredObject};
use fe_core::text::{CaretPos, TextCursor, TextData, lay_out};

pub use input::keys;
pub use mouse::pointer;

use crate::app::FlashApp;
use crate::commands::{parts, text as text_cmd};
use crate::tools::ToolKind;

/// The text being edited.
pub struct TextEditor {
    layer: LayerId,
    part: Option<usize>,
    cursor: TextCursor,
    /// The document as it was before the edit (the undo step goes from there).
    before: Document,
    /// The text object was made by this edit (an empty one is thrown away again).
    created: bool,
    changed: bool,
    /// Where the cursor can stand, and the natural size of the drawing they are measured in.
    carets: Vec<CaretPos>,
    natural: (f64, f64),
    /// Which project and page the text is on.
    place: (usize, usize),
    /// The mouse button is held: dragging selects.
    drag: bool,
    /// When the cursor last moved or text was typed (the cursor shows right after).
    last_input: f64,
}

/// Where the cursor starts.
#[derive(Clone, Copy, Debug)]
pub enum Start {
    /// At the end of the text.
    End,
    /// Everything selected.
    All,
    /// At a spot on the canvas; `true` selects the word there (a double-click).
    At(PointD, bool),
}

/// True while text is being edited.
pub fn active(app: &FlashApp) -> bool {
    app.text_edit.is_some()
}

/// The edited text object as it is on the canvas now (the part of a group, or the layer's own object).
fn target(app: &FlashApp, layer: LayerId, part: Option<usize>) -> Option<(StoredObject, TextData)> {
    let whole = app.doc().layer(layer)?.object()?;
    let object = match part {
        Some(i) => whole.part(i)?.object,
        None => whole.clone(),
    };
    let text = object.text()?;
    Some((object, text))
}

/// The box and mirroring of the edited text.
fn frame(app: &FlashApp, ed: &TextEditor) -> Option<(ObjectBox, (bool, bool))> {
    let (object, _) = target(app, ed.layer, ed.part)?;
    Some((object.box_()?, object.flips()))
}

/// A place in the text's natural drawing as a point on the canvas.
fn canvas_point(b: &ObjectBox, flips: (bool, bool), natural: (f64, f64), p: (f64, f64)) -> PointD {
    to_canvas(b, flips, PointD::new(p.0 / natural.0.max(1.0), p.1 / natural.1.max(1.0)))
}

fn relayout(ed: &mut TextEditor, text: &TextData) {
    if let Some(laid) = lay_out(text) {
        ed.carets = laid.carets;
        ed.natural = (laid.width, laid.height);
    }
}

/// Starts editing the text of `layer` (or of one of its parts, when it is a group).
pub fn begin(app: &mut FlashApp, layer: LayerId, part: Option<usize>, start: Start) {
    let before = app.doc().clone();
    begin_with(app, layer, part, start, before, false);
}

fn begin_with(app: &mut FlashApp, layer: LayerId, part: Option<usize>, start: Start, before: Document, created: bool) {
    finish(app);
    if app.doc().layer(layer).is_some_and(|l| l.locked) {
        app.status = "This text is locked. Unlock it first (Ctrl+L).".into();
        return;
    }
    let Some((_, text)) = target(app, layer, part) else { return };
    app.project_mut().active_page_mut().doc.set_active(Some(layer));
    parts::select_part(app, part.map(|i| (layer, i)));
    let mut ed = TextEditor {
        layer,
        part,
        cursor: TextCursor::new(&text.text),
        before,
        created,
        changed: created,
        carets: Vec::new(),
        natural: (1.0, 1.0),
        place: (app.active_project, app.project().active_index()),
        drag: false,
        last_input: 0.0,
    };
    relayout(&mut ed, &text);
    match start {
        Start::End => {}
        Start::All => ed.cursor.select_all(),
        Start::At(pos, word) => {
            if let Some(index) = mouse::index_at(app, &ed, pos) {
                ed.cursor.set_caret(index, false);
                if word {
                    ed.cursor.select_word(index);
                }
            }
        }
    }
    app.status = "Type to edit the text. Esc or a click outside finishes. Ctrl+A selects all.".into();
    app.text_edit = Some(ed);
}

/// A new, empty text object at `at` in the Text tool's font, size and style, ready to type into.
pub fn create_new(app: &mut FlashApp, at: PointD) {
    finish(app);
    let before = app.doc().clone();
    let o = &app.tools.text;
    let mut data = TextData::new("", &o.family, o.size, app.foreground);
    data.bold = o.bold;
    data.italic = o.italic;
    let (w, h) = fe_core::render::measure_text(&data);
    let object = StoredObject::new_text(ObjectBox::new(at.x.round(), at.y.round(), w, h, 0.0), &data);
    let canvas = app.doc().bounds();
    let layer = object_ops::object_layer("Text", object, canvas);
    let id = layer.id();
    let top = app.doc().layers().len(); // a new object is always the top layer
    app.project_mut().active_page_mut().doc.insert_layer(top, layer);
    app.project_mut().active_page_mut().doc.set_selected_area(None);
    begin_with(app, id, None, Start::End, before, true);
}

/// Insert > Heading / Subheading / Body Text: `data` in the middle of the picture, all selected so typing replaces it.
pub fn create_centered(app: &mut FlashApp, data: TextData) {
    finish(app);
    let before = app.doc().clone();
    let (w, h) = fe_core::render::measure_text(&data);
    let (dw, dh) = (f64::from(app.doc().width()), f64::from(app.doc().height()));
    let b = ObjectBox::new(((dw - w) / 2.0).round(), ((dh - h) / 2.0).round(), w, h, 0.0);
    let object = StoredObject::new_text(b, &data);
    let canvas = app.doc().bounds();
    let layer = object_ops::object_layer(text_cmd::layer_name(&data.text), object, canvas);
    let id = layer.id();
    let top = app.doc().layers().len();
    app.project_mut().active_page_mut().doc.insert_layer(top, layer);
    app.project_mut().active_page_mut().doc.set_selected_area(None);
    begin_with(app, id, None, Start::All, before, true);
}

/// Puts the words typed so far into the text object (the box follows the text) and measures the cursor places.
fn apply(app: &mut FlashApp, ed: &mut TextEditor) {
    let canvas = app.doc().bounds();
    let Some(whole) = app.doc().layer(ed.layer).and_then(|l| l.object()).cloned() else { return };
    let Some((mut object, mut text)) = target(app, ed.layer, ed.part) else { return };
    text.text = ed.cursor.text();
    object.set_text(&text);
    fe_core::text::refit(&mut object, &text);
    let Some(updated) = parts::with_target(&whole, ed.part, object) else { return };
    if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(ed.layer) {
        object_ops::set_object(layer, updated, canvas);
    }
    ed.changed = true;
    relayout(ed, &text);
}

/// Finishes: the edit becomes one undo step. A new text that stayed empty is thrown away.
pub fn finish(app: &mut FlashApp) {
    let Some(ed) = app.text_edit.take() else { return };
    if ed.place != (app.active_project, app.project().active_index()) {
        return;
    }
    let words = ed.cursor.text();
    if ed.created && words.trim().is_empty() {
        app.project_mut().active_page_mut().doc = ed.before;
        app.status = "Empty text removed.".into();
        return;
    }
    if !ed.created && (!ed.changed || words == target_words(&ed.before, ed.layer, ed.part)) {
        return; // nothing changed: no undo step
    }
    if ed.created {
        let name = text_cmd::layer_name(&words);
        if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(ed.layer) {
            layer.name = name;
        }
        app.tools.activate(ToolKind::Move);
        app.status =
            "Text added. Drag to move, corners to resize; change font and color in the Properties panel.".into();
    }
    let name = if ed.created { "Add Text" } else { "Edit Text" };
    app.project_mut().record(name, ed.before);
}

/// The words the text had in `doc`.
fn target_words(doc: &Document, layer: LayerId, part: Option<usize>) -> String {
    let Some(whole) = doc.layer(layer).and_then(|l| l.object()) else { return String::new() };
    match part {
        Some(i) => whole.part(i).and_then(|m| m.object.text()),
        None => whole.text(),
    }
    .map(|t| t.text)
    .unwrap_or_default()
}

/// Stops editing and puts everything back as it was (Ctrl+Z while editing).
pub fn cancel(app: &mut FlashApp) {
    let Some(ed) = app.text_edit.take() else { return };
    if ed.place == (app.active_project, app.project().active_index()) {
        app.project_mut().active_page_mut().doc = ed.before;
    }
}

/// Called every frame: editing ends when the user goes elsewhere (another layer, page or project).
pub fn check(app: &mut FlashApp) {
    let Some(ed) = &app.text_edit else { return };
    let here = (app.active_project, app.project().active_index());
    if ed.place != here || app.doc().layer(ed.layer).is_none() {
        app.text_edit = None;
        return;
    }
    if app.doc().active_id() != Some(ed.layer) {
        finish(app);
    }
}
