//! The tutorial project (C# `DemoProject.Create`): made the first time the program starts, and opened from
//! View > Tutorial. It lives in the projects folder, so it also shows in the Project Manager.

use std::path::PathBuf;

use crate::app::FlashApp;

/// Where the tutorial is kept.
pub fn path(app: &FlashApp) -> PathBuf {
    app.settings.projects_folder().join(fe_core::tutorial::FILE_NAME)
}

/// Makes the tutorial file when it is not there yet (or `overwrite`). Returns its path, or why it failed.
pub fn create(app: &FlashApp, overwrite: bool) -> Result<PathBuf, String> {
    let path = path(app);
    if path.exists() && !overwrite {
        return Ok(path);
    }
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    }
    let pages = fe_core::tutorial::build_pages();
    fe_core::io::project::save(&pages, 0, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

/// View > Tutorial: opens the tutorial project (making it first when it is missing).
pub fn open(app: &mut FlashApp, ctx: &egui::Context) {
    match create(app, false) {
        Ok(path) => super::file::open_path(app, ctx, &path, false),
        Err(err) => app.status = format!("The tutorial could not be made: {err}"),
    }
}

/// First start: makes the tutorial file once.
pub fn first_start(app: &mut FlashApp) {
    if app.settings.demo_created {
        return;
    }
    app.settings.demo_created = true;
    if let Err(err) = create(app, false) {
        log::warn!("Could not make the tutorial: {err}");
    }
}
