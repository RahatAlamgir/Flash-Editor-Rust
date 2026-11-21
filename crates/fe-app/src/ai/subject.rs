//! Select > Select Subject (AI): the AI finds the main subject of what you see and selects it.

use fe_core::paint::compositor;

use super::job::{self, Progress};
use super::masks::selection_from_canvas_mask;
use super::outcome::{self, AiDone};
use super::pending::{self, Pending};
use crate::app::FlashApp;

/// The menu command: makes sure the model is there, then runs.
pub fn start(app: &mut FlashApp) {
    pending::run(app, Pending::SelectSubject);
}

pub fn run(app: &mut FlashApp) {
    let doc = app.doc().uid();
    let flat = compositor::flatten(app.doc());
    let segmenter = app.ai.services.subject(&app.ai.store);
    app.status = "The AI is finding the subject (a few seconds the first time)...".into();
    job::start(app, "Finding the subject", move |progress: &Progress| {
        let result = segmenter.predict_mask(&flat, progress.flag());
        AiDone::Subject { doc, result }
    });
}

pub fn on_done(app: &mut FlashApp, doc: u64, result: Result<Vec<f32>, String>) {
    let mask = match result {
        Ok(mask) => mask,
        Err(e) => return outcome::failed(app, "Select Subject (AI)", &e),
    };
    if app.doc().uid() != doc {
        app.status = "Another picture is shown now, so the subject was not selected.".into();
        return;
    }
    let (w, h) = (app.doc().width(), app.doc().height());
    let bytes: Vec<u8> = mask.iter().map(|m| if *m >= 0.5 { 255 } else { 0 }).collect();
    let selection = selection_from_canvas_mask(&bytes, w, h);
    app.status = if selection.is_none() {
        "No clear subject found.".into()
    } else {
        "Subject selected. Copy (Ctrl+C), delete, or apply an adjustment to just the subject.".into()
    };
    app.project_mut().active_page_mut().doc.set_selected_area(selection);
}
