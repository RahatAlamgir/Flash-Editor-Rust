//! One AI job on a background thread: a progress number the window shows in the status bar, and a Cancel flag the AI
//! checks between its steps.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use super::outcome::AiDone;
use crate::app::FlashApp;
use crate::app::jobs::JobDone;

/// The part of a job the window sees.
pub struct AiJob {
    pub title: String,
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
}

impl AiJob {
    /// 0..1.
    pub fn fraction(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / 1000.0
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// The part of a job the worker thread sees.
#[derive(Clone)]
pub struct Progress {
    progress: Arc<AtomicU32>,
    cancel: Arc<AtomicBool>,
}

impl Progress {
    /// Reports how far the work is (0..1).
    pub fn report(&self, fraction: f64) {
        self.progress.store((fraction.clamp(0.0, 1.0) * 1000.0) as u32, Ordering::Relaxed);
    }

    /// The flag the AI checks to stop early.
    pub fn flag(&self) -> &AtomicBool {
        &self.cancel
    }
}

/// Starts an AI job. False (and a message) when another one is still running.
pub fn start(app: &mut FlashApp, title: &str, work: impl FnOnce(&Progress) -> AiDone + Send + 'static) -> bool {
    if app.ai.job.is_some() {
        app.status = "The AI is still busy with another job. Wait for it, or cancel it in the status bar.".into();
        return false;
    }
    let Some(ctx) = app.ai.ctx.clone() else { return false };
    let progress = Arc::new(AtomicU32::new(0));
    let cancel = Arc::new(AtomicBool::new(false));
    app.ai.job = Some(AiJob { title: title.to_owned(), progress: progress.clone(), cancel: cancel.clone() });
    let worker = Progress { progress, cancel };
    app.jobs.spawn(&ctx, move || JobDone::Ai(work(&worker)));
    true
}
