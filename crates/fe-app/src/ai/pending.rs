//! "Ask before downloading" (C# `EnsureModel`): an AI feature first checks that its models are on this PC. When they
//! are not, the user is asked once; after the download the feature carries on by itself.

use fe_ai::models::{ISNET, LAMA, MOBILE_SAM_DECODER, MOBILE_SAM_ENCODER, ModelInfo, REAL_ESRGAN};

use super::job::{self, Progress};
use super::outcome::AiDone;
use crate::app::FlashApp;
use crate::dialogs::{Answer, Ask, Dialog};

/// An AI feature waiting for its models.
#[derive(Clone, Debug, PartialEq)]
pub enum Pending {
    RemoveBackground,
    SelectSubject,
    FillSelection,
    UpscaleWindow,
    ClickSelect { points: Vec<(f64, f64, bool)>, boxed: Option<(f64, f64, f64, f64)>, shift: bool, alt: bool },
}

impl Pending {
    pub fn title(&self) -> &'static str {
        match self {
            Self::RemoveBackground => "Remove Background (AI)",
            Self::SelectSubject => "Select Subject (AI)",
            Self::FillSelection => "Remove / Fill Selection (AI)",
            Self::UpscaleWindow => "AI Upscale / Clean Up",
            Self::ClickSelect { .. } => "Click Select (AI)",
        }
    }

    fn models(&self) -> Vec<&'static ModelInfo> {
        match self {
            Self::RemoveBackground | Self::SelectSubject => vec![&ISNET],
            Self::FillSelection => vec![&LAMA],
            Self::UpscaleWindow => vec![&REAL_ESRGAN],
            Self::ClickSelect { .. } => vec![&MOBILE_SAM_ENCODER, &MOBILE_SAM_DECODER],
        }
    }
}

/// Runs the feature now when its models are installed; otherwise asks to download them first.
pub fn run(app: &mut FlashApp, pending: Pending) {
    if app.ai.job.is_some() {
        app.status = "The AI is still busy with another job. Wait for it, or cancel it in the status bar.".into();
        return;
    }
    let missing: Vec<&ModelInfo> = pending.models().into_iter().filter(|m| !app.ai.store.is_installed(m)).collect();
    if missing.is_empty() {
        execute(app, pending);
        return;
    }
    let names: Vec<&str> = missing.iter().map(|m| m.display_name).collect();
    let total: u64 = missing.iter().map(|m| m.size_bytes).sum();
    let text = format!(
        "This AI feature needs a one-time download of the {} ({:.0} MB).\n\nIt is saved in:\n{}\n\nDownload it now?",
        names.join(" and the "),
        total as f64 / (1024.0 * 1024.0),
        app.ai.store.folder().display()
    );
    let ask = Ask::new(
        pending.title(),
        text,
        &[("Download", Answer::DownloadModels(pending.clone())), ("Not now", Answer::Nothing)],
    );
    app.dialog = Some(Dialog::Ask(ask));
}

/// The models are there: starts the feature.
pub fn execute(app: &mut FlashApp, pending: Pending) {
    match pending {
        Pending::RemoveBackground => super::remove_bg::open(app),
        Pending::SelectSubject => super::subject::run(app),
        Pending::FillSelection => super::fill::run(app),
        Pending::UpscaleWindow => super::upscale::open(app),
        Pending::ClickSelect { points, boxed, shift, alt } => super::click::run(app, points, boxed, shift, alt),
    }
}

/// The user said "Download": downloads what is missing, then carries on with the feature.
pub fn download(app: &mut FlashApp, pending: Pending) {
    let missing: Vec<&'static ModelInfo> =
        pending.models().into_iter().filter(|m| !app.ai.store.is_installed(m)).collect();
    let title = format!("Downloading for {}", pending.title());
    download_list(app, &title, missing, Some(pending));
}

/// Downloads models on a background thread (progress and Cancel in the status bar); `then` runs afterwards.
pub fn download_list(app: &mut FlashApp, title: &str, models: Vec<&'static ModelInfo>, then: Option<Pending>) {
    let store = app.ai.store.clone();
    app.status = format!("{title}...");
    job::start(app, title, move |progress: &Progress| {
        let count = models.len().max(1) as f64;
        for (i, model) in models.iter().enumerate() {
            let step = |f: f64| progress.report((i as f64 + f) / count);
            if let Err(error) = store.download(model, &step, progress.flag()) {
                return AiDone::Downloaded { result: Err(error), then };
            }
        }
        AiDone::Downloaded { result: Ok(()), then }
    });
}
