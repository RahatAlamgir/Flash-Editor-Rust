//! Remove / Fill Selection (AI) and the Remove Brush: the AI paints in what would plausibly be where the selection is.
//! The result goes on a new layer "AI Fill" above the active layer, so the original is never changed.

use fe_core::document::Layer;
use fe_core::paint::compositor;

use super::job::{self, Progress};
use super::masks::canvas_mask;
use super::outcome::{self, AiDone};
use super::pending::{self, Pending};
use crate::app::FlashApp;

const TITLE: &str = "Remove / Fill Selection (AI)";

/// The menu command (and the end of a Remove Brush stroke).
pub fn start(app: &mut FlashApp) {
    if app.doc().selected_area().is_none() {
        outcome::tell(
            app,
            TITLE,
            "First select the area you want to remove or fill:\n\n  \u{2022} Lasso (L) or Rectangle (M): draw around it\n  \
             \u{2022} Magic Wand (W): click on it\n  \u{2022} Select > Select Subject (AI): for a whole person or object\n\n\
             Select a little more than the object itself (include its shadow). Then run this again: the AI paints in what \
             would be behind it.",
        );
        return;
    }
    pending::run(app, Pending::FillSelection);
}

pub fn run(app: &mut FlashApp) {
    let Some(selection) = app.doc().selected_area().cloned() else { return };
    let (w, h) = (app.doc().width(), app.doc().height());
    let doc = app.doc().uid();
    let flat = compositor::flatten(app.doc()); // the AI sees what you see, all visible layers together
    let mask = canvas_mask(&selection, w, h);
    let bounds = selection.bounds();
    let big = i64::from(bounds.width) * i64::from(bounds.height) > i64::from(w) * i64::from(h) / 3;
    let inpainter = app.ai.services.inpaint(&app.ai.store);
    app.status =
        "The AI is filling the selection (the first time takes a little longer while the model loads)...".into();
    job::start(app, "Filling the selection", move |progress: &Progress| {
        let result = inpainter.fill(&flat, &mask, &|f| progress.report(f), progress.flag());
        AiDone::Fill { doc, result, big }
    });
}

pub fn on_done(app: &mut FlashApp, doc: u64, result: Result<Option<fe_core::imaging::PixelBuffer>, String>, big: bool) {
    let fill = match result {
        Ok(Some(fill)) => fill,
        Ok(None) => {
            app.status = "Nothing was selected, so nothing was filled.".into();
            return;
        }
        Err(e) => return outcome::failed(app, TITLE, &e),
    };
    if app.doc().uid() != doc {
        app.status = "Another picture is shown now, so the fill was not added.".into();
        return;
    }
    app.project_mut().edit("Remove / Fill (AI)", None, |doc| {
        let top = doc.active_index().unwrap_or(0);
        let at = (top + 1).min(doc.layers().len());
        let id = doc.insert_layer(at, Layer::from_buffer("AI Fill", &fill));
        doc.set_selection(&[id], Some(id));
    });
    app.status = if big {
        "Filled. A large area is filled piece by piece; if some part looks off, undo and fill that part on its own. Ctrl+Z undoes it."
            .into()
    } else {
        "Filled on a new layer \"AI Fill\". Hide it to compare, or press Ctrl+Z to undo.".into()
    };
}
