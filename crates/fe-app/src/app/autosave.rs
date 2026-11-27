//! Crash protection (C# `AutoSaveService`): every minute, unsaved work is written in the background to
//! `settings\recovery\unsaved-work.fe`. A normal exit deletes it; if the program did not close properly, the
//! next start offers to bring the work back.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use fe_core::document::{Page, Project};
use fe_core::io::project;

use super::jobs::JobDone;
use super::{FlashApp, paths};
use crate::dialogs::{Answer, Ask, Dialog};

const INTERVAL: Duration = Duration::from_secs(60);

/// The backup timer.
pub struct AutoSave {
    last: Instant,
    /// What was written last time (so an unchanged project is not written again).
    last_stamp: Option<u64>,
    writing: bool,
}

impl Default for AutoSave {
    fn default() -> Self {
        Self { last: Instant::now(), last_stamp: None, writing: false }
    }
}

/// The backup file.
pub fn recovery_file() -> PathBuf {
    paths::settings_dir().join("recovery").join("unsaved-work.fe")
}

/// Deletes the backup (normal exit, or the user did not want it back).
pub fn delete_recovery() {
    let _ = std::fs::remove_file(recovery_file());
}

/// Call every frame: writes a backup when a minute has passed and there is unsaved work.
pub fn tick(app: &mut FlashApp, ctx: &egui::Context) {
    ctx.request_repaint_after(INTERVAL);
    let busy = ctx.input(|i| i.pointer.any_down()); // never compete with a drag or a brush stroke
    if app.autosave.writing || busy || app.autosave.last.elapsed() < INTERVAL {
        return;
    }
    app.autosave.last = Instant::now();
    let dirty: Vec<&Project> = app.projects.iter().filter(|p| p.is_dirty()).collect();
    if dirty.is_empty() {
        delete_recovery();
        app.autosave.last_stamp = None;
        return;
    }
    let stamp = dirty.iter().fold(dirty.len() as u64, |s, p| {
        s.wrapping_mul(1_000_003)
            .wrapping_add(p.uid() * 31 + p.history().position() as u64 * 7 + p.history().steps().len() as u64)
    });
    if app.autosave.last_stamp == Some(stamp) && recovery_file().exists() {
        return;
    }
    let pages: Vec<Page> = dirty
        .iter()
        .flat_map(|p| {
            let several = p.pages().len() > 1;
            p.pages().iter().map(move |page| {
                let mut copy = page.clone(); // cheap: tiles are shared
                copy.name = if several { format!("{} - {}", p.name, page.name) } else { p.name.clone() };
                copy
            })
        })
        .collect();
    app.autosave.writing = true;
    app.autosave.last_stamp = Some(stamp);
    let file = recovery_file();
    app.jobs.spawn(ctx, move || {
        let result = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .map_err(|e| e.to_string())
            .and_then(|()| project::save(&pages, 0, &file).map_err(|e| e.to_string()));
        JobDone::AutoSaved(result)
    });
}

/// A backup finished. A failed backup is only logged: it must never disturb the user.
pub fn on_saved(app: &mut FlashApp, result: Result<(), String>) {
    app.autosave.writing = false;
    if let Err(err) = result {
        log::warn!("Backup failed: {err}");
        app.autosave.last_stamp = None;
    }
}

/// At start-up: when a backup is there, asks whether to bring the work back.
pub fn offer_recovery(app: &mut FlashApp) {
    let Ok(content) = project::open(recovery_file()) else {
        delete_recovery();
        return;
    };
    let n = content.pages.len();
    let text = format!(
        "Flash Editor did not close properly last time.\n\nBring back your unsaved work ({n} picture{})?",
        if n == 1 { "" } else { "s" }
    );
    app.dialog = Some(Dialog::Ask(Ask::new(
        "Flash Editor",
        text,
        &[("Bring it back", Answer::RecoverWork), ("No, delete it", Answer::DiscardRecovery)],
    )));
}

/// Brings the backed-up pages back, one project per page, marked as unsaved.
pub fn recover(app: &mut FlashApp) {
    let Ok(content) = project::open(recovery_file()) else { return };
    let n = content.pages.len();
    for page in content.pages {
        let name = format!("{} (recovered)", page.name);
        let mut project = Project::from_pages(name, vec![page], 0);
        project.history_mut().mark_unsaved();
        crate::commands::file::add_project(app, project);
    }
    delete_recovery();
    app.autosave.last_stamp = None;
    app.status = format!("Brought back {n} unsaved picture{}. Save them to keep them.", if n == 1 { "" } else { "s" });
}
