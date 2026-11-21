//! What happens when an AI job is finished (the window gets the result at the start of the next frame).

use egui::Modifiers;
use fe_core::imaging::PixelBuffer;

use super::Pending;
use crate::app::FlashApp;
use crate::dialogs::{Answer, Ask, Dialog};

/// The result of an AI job. `doc` is the id of the picture it was made for: when another picture is shown by then, the
/// result is dropped.
pub enum AiDone {
    Downloaded {
        result: Result<(), String>,
        then: Option<Pending>,
    },
    /// A live preview of Remove Background.
    BackgroundPreview {
        generation: u64,
        result: Result<PixelBuffer, String>,
    },
    Subject {
        doc: u64,
        result: Result<Vec<f32>, String>,
    },
    Fill {
        doc: u64,
        result: Result<Option<PixelBuffer>, String>,
        big: bool,
    },
    Upscale {
        doc: u64,
        result: Result<super::upscale::Upscaled, String>,
    },
    Click {
        doc: u64,
        result: Result<Vec<u8>, String>,
        mods: Modifiers,
    },
}

/// A short window with one OK button.
pub fn tell(app: &mut FlashApp, title: &str, text: impl Into<String>) {
    app.dialog = Some(Dialog::Ask(Ask::new(title, text, &[("OK", Answer::Nothing)])));
}

/// Handles a finished job.
pub fn on_done(app: &mut FlashApp, done: AiDone) {
    // A preview is not "the" job: it never takes the job slot.
    if !matches!(done, AiDone::BackgroundPreview { .. }) {
        app.ai.job = None;
    }
    match done {
        AiDone::Downloaded { result, then } => match result {
            Ok(()) => match then {
                Some(then) => super::pending::execute(app, then),
                None => app.status = "Download finished.".into(),
            },
            Err(e) if e == "Cancelled." => app.status = "Download cancelled.".into(),
            Err(e) => tell(app, "Download failed", format!("The model could not be downloaded:\n\n{e}")),
        },
        AiDone::BackgroundPreview { generation, result } => super::remove_bg::on_preview(app, generation, result),
        AiDone::Subject { doc, result } => super::subject::on_done(app, doc, result),
        AiDone::Fill { doc, result, big } => super::fill::on_done(app, doc, result, big),
        AiDone::Upscale { doc, result } => super::upscale::on_done(app, doc, result),
        AiDone::Click { doc, result, mods } => super::click::on_done(app, doc, result, mods),
    }
}

/// The message for a failed job ("Cancelled." is not an error).
pub fn failed(app: &mut FlashApp, title: &str, error: &str) {
    if error == "Cancelled." {
        app.status = format!("{title} cancelled. Nothing was changed.");
    } else {
        tell(app, title, format!("The AI could not finish:\n\n{error}"));
    }
}
