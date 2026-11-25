//! Work done on background threads (opening and saving files, autosave, listing projects). Results come back
//! as messages that the app handles at the start of the next frame, so the window never freezes.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use fe_core::imaging::PixelBuffer;
use fe_core::io::project::ProjectContent;

/// A file that was read.
pub enum OpenedFile {
    Project(ProjectContent),
    Picture(PixelBuffer),
    /// A Photoshop or SVG file: a document with layers.
    Document(fe_core::document::Document),
}

/// What to do after a save finished.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterSave {
    Nothing,
    CloseProject,
    /// Keep saving the other unsaved projects, then close the window.
    ContinueClosingWindow,
}

/// A finished job.
pub enum JobDone {
    Opened {
        path: PathBuf,
        result: Result<OpenedFile, String>,
        as_layers: bool,
    },
    Saved {
        project: u64,
        path: PathBuf,
        position: usize,
        result: Result<(), String>,
        then: AfterSave,
    },
    AutoSaved(Result<(), String>),
    ProjectList(Vec<crate::dialogs::project_manager::Entry>),
    /// Something the AI made (or a model that was downloaded).
    Ai(crate::ai::AiDone),
    /// The real file made for the Export window (file size and preview).
    ExportPreview {
        version: u64,
        settings: fe_core::io::export::ExportSettings,
        result: Result<(u64, fe_core::imaging::PixelBuffer, fe_core::imaging::PixelBuffer), String>,
    },
    /// The pieces found for the Split Asset Sheet window.
    SheetPieces {
        version: u64,
        result: Result<std::sync::Arc<Vec<fe_core::sheets::SheetPiece>>, String>,
    },
    /// A live preview of an effect.
    EffectPreview {
        generation: u64,
        tiles: fe_core::imaging::TileMap,
    },
}

/// The job queue.
pub struct Jobs {
    sender: Sender<JobDone>,
    receiver: Receiver<JobDone>,
    /// Jobs started and not finished yet.
    pub running: usize,
}

impl Default for Jobs {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver, running: 0 }
    }
}

impl Jobs {
    /// Runs `work` on a background thread; its result arrives through [`Self::finished`].
    pub fn spawn(&mut self, ctx: &egui::Context, work: impl FnOnce() -> JobDone + Send + 'static) {
        self.running += 1;
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        rayon::spawn(move || {
            let _ = sender.send(work());
            ctx.request_repaint();
        });
    }

    /// The jobs that finished since the last call.
    pub fn finished(&mut self) -> Vec<JobDone> {
        let done: Vec<JobDone> = self.receiver.try_iter().collect();
        self.running = self.running.saturating_sub(done.len());
        done
    }
}
