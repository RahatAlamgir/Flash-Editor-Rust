//! Click Select (AI): click an object (or drag a box around it) and the AI selects just that object. Shift adds to the
//! selection, Alt takes away from it.

use egui::Modifiers;
use fe_ai::segmentation::ClickPrompt;
use fe_core::paint::compositor;
use fe_core::selection::Selection;

use super::job::{self, Progress};
use super::masks::selection_from_canvas_mask;
use super::outcome::{self, AiDone};
use super::pending::{self, Pending};
use crate::app::FlashApp;
use crate::tools::selecting::apply;

/// A click or a dragged box on the picture.
pub fn start(app: &mut FlashApp, points: Vec<(f64, f64, bool)>, boxed: Option<(f64, f64, f64, f64)>, mods: Modifiers) {
    pending::run(app, Pending::ClickSelect { points, boxed, shift: mods.shift, alt: mods.alt });
}

pub fn run(
    app: &mut FlashApp,
    points: Vec<(f64, f64, bool)>,
    boxed: Option<(f64, f64, f64, f64)>,
    shift: bool,
    alt: bool,
) {
    let doc = app.doc().uid();
    let flat = compositor::flatten(app.doc());
    let segmenter = app.ai.services.click(&app.ai.store);
    let mods = Modifiers { shift, alt, ..Modifiers::NONE };
    app.status = "The AI is outlining the object (the first click on a picture takes a few seconds)...".into();
    job::start(app, "Finding the object", move |progress: &Progress| {
        let prompt = ClickPrompt { points, boxed };
        let result = segmenter.predict(&flat, &prompt, progress.flag());
        AiDone::Click { doc, result, mods }
    });
}

pub fn on_done(app: &mut FlashApp, doc: u64, result: Result<Vec<u8>, String>, mods: Modifiers) {
    let mask = match result {
        Ok(mask) => mask,
        Err(e) => return outcome::failed(app, "Click Select (AI)", &e),
    };
    if app.doc().uid() != doc {
        app.status = "Another picture is shown now, so nothing was selected.".into();
        return;
    }
    let (w, h) = (app.doc().width(), app.doc().height());
    let drawn: Option<Selection> = selection_from_canvas_mask(&mask, w, h);
    apply::apply(app, drawn, mods);
}
