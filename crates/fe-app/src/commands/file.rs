//! File commands: open (pictures and `.fe` projects), save, save as, close, add pictures as layers.
//! Reading and writing run on background threads; `on_opened` / `on_saved` finish the job.

use std::path::{Path, PathBuf};

use fe_core::document::{Document, Layer, Page, Project};
use fe_core::imaging::PixelBuffer;
use fe_core::io::{decode, format, project};

use crate::app::FlashApp;
use crate::app::jobs::{AfterSave, JobDone, OpenedFile};
use crate::app::settings::{clean_name, new_project_path};
use crate::dialogs::{Answer, Ask, Dialog};

fn file_name(path: &Path) -> String {
    path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

fn stem(path: &Path) -> String {
    path.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "Picture".into())
}

/// File > Open (Ctrl+O): pictures and projects.
pub fn open_dialog(app: &mut FlashApp, ctx: &egui::Context) {
    let mut all: Vec<&str> = vec![project::EXTENSION, "psd", "svg"];
    all.extend(format::COMMON_EXTENSIONS);
    all.extend(format::WINDOWS_EXTENSIONS);
    let picked = rfd::FileDialog::new()
        .set_title("Open a picture or project")
        .add_filter("Pictures and projects", &all)
        .add_filter("Flash Editor project", &[project::EXTENSION])
        .add_filter("Photoshop file", &["psd"])
        .add_filter("SVG vector picture", &["svg"])
        .add_filter("All files", &["*"])
        .set_directory(app.settings.projects_folder())
        .pick_files();
    for path in picked.unwrap_or_default() {
        open_path(app, ctx, &path, false);
    }
}

/// File > Place Image as Layer: a picture file becomes a new layer on the shown page.
pub fn place_image(app: &mut FlashApp, ctx: &egui::Context) {
    let mut all: Vec<&str> = Vec::new();
    all.extend(format::COMMON_EXTENSIONS);
    all.extend(format::WINDOWS_EXTENSIONS);
    let picked = rfd::FileDialog::new()
        .set_title("Place a picture as a layer")
        .add_filter("Pictures", &all)
        .set_directory(app.settings.projects_folder())
        .pick_files();
    for path in picked.unwrap_or_default() {
        open_path(app, ctx, &path, true);
    }
}

/// Opens a file in the background. `as_layers`: pictures are added to the shown page (Ctrl+drop).
pub fn open_path(app: &mut FlashApp, ctx: &egui::Context, path: &Path, as_layers: bool) {
    let is_project = project::is_project(path);
    if is_project && let Some(index) = app.projects.iter().position(|p| p.path.as_deref() == Some(path)) {
        app.active_project = index; // already open
        return;
    }
    app.status = format!("Opening {}...", file_name(path));
    let path = path.to_path_buf();
    app.jobs.spawn(ctx, move || {
        let result = if is_project {
            project::open(&path).map(OpenedFile::Project)
        } else if fe_core::io::psd::is_psd(&path) {
            fe_core::io::psd::open(&path).map(OpenedFile::Document)
        } else if fe_core::io::svg::is_svg(&path) {
            fe_core::io::svg::open(&path).map(OpenedFile::Document)
        } else {
            decode::load(&path).map(OpenedFile::Picture)
        };
        JobDone::Opened { path, result: result.map_err(|e| e.to_string()), as_layers }
    });
}

/// A file finished loading.
pub fn on_opened(app: &mut FlashApp, path: PathBuf, result: Result<OpenedFile, String>, as_layers: bool) {
    match result {
        Err(err) => app.status = format!("{}: {err}", file_name(&path)),
        Ok(OpenedFile::Project(content)) => {
            let pages = content.pages.len();
            let mut project = Project::from_pages(stem(&path), content.pages, content.active_page);
            project.path = Some(path.clone());
            project.presentation = content.presentation;
            add_project(app, project);
            app.settings.add_recent(&path);
            app.status = format!("Opened {} ({pages} page{}).", stem(&path), if pages == 1 { "" } else { "s" });
        }
        Ok(OpenedFile::Document(doc)) => {
            let (w, h, layers) = (doc.width(), doc.height(), doc.layers().len());
            add_project(app, Project::new(stem(&path), Page::new("Page 1", doc)));
            app.status = format!(
                "Opened {} ({w} x {h} px, {layers} layer{}).",
                file_name(&path),
                if layers == 1 { "" } else { "s" }
            );
        }
        Ok(OpenedFile::Picture(image)) if as_layers => add_picture_layer(app, &image, &stem(&path)),
        Ok(OpenedFile::Picture(image)) => {
            let doc = Document::from_image(&image, "Background");
            add_project(app, Project::new(stem(&path), Page::new("Page 1", doc)));
            app.status = format!("Opened {} ({} x {} px).", file_name(&path), image.width(), image.height());
        }
    }
}

/// Adds a project as a new tab and shows it. The blank start-up project is replaced if it was never touched.
pub fn add_project(app: &mut FlashApp, mut project: Project) {
    project.history_mut().set_max_steps(app.settings.undo_steps);
    if app.projects.len() == 1 && app.start_project_untouched() {
        app.projects.clear();
        app.canvas.views.clear();
    }
    app.projects.push(project);
    app.active_project = app.projects.len() - 1;
}

/// A picture as a new layer on top of the shown page, centered (Ctrl+drop).
fn add_picture_layer(app: &mut FlashApp, image: &PixelBuffer, name: &str) {
    let name = name.to_owned();
    let image = image.clone();
    app.project_mut().edit("Add Layer", None, move |doc| {
        let x = (doc.width() as i32 - image.width() as i32) / 2;
        let y = (doc.height() as i32 - image.height() as i32) / 2;
        let mut layer = Layer::new(name);
        layer.pixels_mut().write_buffer(&image, x, y);
        doc.push_layer(layer);
    });
    app.status = "Added the picture as a new layer. Ctrl+Z takes it back.".into();
}

/// File > Save (Ctrl+S). A project without a file asks where to save it.
pub fn save(app: &mut FlashApp, ctx: &egui::Context) {
    match app.project().path.clone() {
        Some(path) => save_to(app, ctx, app.active_project, path, AfterSave::Nothing),
        None => {
            save_as(app, ctx, app.active_project, AfterSave::Nothing);
        }
    }
}

/// File > Save As (Ctrl+Shift+S). Returns false when the user cancelled.
pub fn save_as(app: &mut FlashApp, ctx: &egui::Context, index: usize, then: AfterSave) -> bool {
    let name = clean_name(&app.projects[index].name);
    let picked = rfd::FileDialog::new()
        .set_title("Save project")
        .add_filter("Flash Editor project", &[project::EXTENSION])
        .set_directory(app.settings.projects_folder())
        .set_file_name(format!("{name}.{}", project::EXTENSION))
        .save_file();
    match picked {
        Some(path) => {
            save_to(app, ctx, index, path, then);
            true
        }
        None => false,
    }
}

/// Saves project `index` to `path` in the background.
pub fn save_to(app: &mut FlashApp, ctx: &egui::Context, index: usize, path: PathBuf, then: AfterSave) {
    let project = &app.projects[index];
    let (pages, active) = (project.pages().to_vec(), project.active_index());
    let presentation = project.presentation;
    let (uid, position) = (project.uid(), project.history().position());
    app.status = format!("Saving {}...", stem(&path));
    app.jobs.spawn(ctx, move || {
        let result = project::save_as(&pages, active, presentation, &path).map_err(|e| e.to_string());
        JobDone::Saved { project: uid, path, position, result, then }
    });
}

/// A save finished.
pub fn on_saved(
    app: &mut FlashApp,
    ctx: &egui::Context,
    uid: u64,
    path: PathBuf,
    position: usize,
    result: Result<(), String>,
    then: AfterSave,
) {
    let Some(index) = app.projects.iter().position(|p| p.uid() == uid) else { return };
    if let Err(err) = result {
        app.status = format!("Could not save {}: {err}", file_name(&path));
        app.closing_window = false;
        return;
    }
    let project = &mut app.projects[index];
    project.history_mut().mark_saved_at(position);
    project.name = stem(&path);
    project.path = Some(path.clone());
    app.settings.add_recent(&path);
    app.status = format!("Saved {}.", stem(&path));
    match then {
        AfterSave::Nothing => {}
        AfterSave::CloseProject => remove_project(app, index),
        AfterSave::ContinueClosingWindow => continue_closing_window(app, ctx),
    }
}

/// The cross on a project tab (Ctrl+W): asks first when there are unsaved changes.
pub fn close_project(app: &mut FlashApp, index: usize) {
    let project = &app.projects[index];
    if !project.is_dirty() {
        remove_project(app, index);
        return;
    }
    let text = format!("Save changes to \"{}\" before closing it?", project.name);
    let uid = project.uid();
    app.dialog = Some(Dialog::Ask(Ask::new(
        "Flash Editor",
        text,
        &[
            ("Save", Answer::SaveAndCloseProject(uid)),
            ("Don't save", Answer::CloseProject(uid)),
            ("Cancel", Answer::Nothing),
        ],
    )));
}

/// Removes a project tab (no questions). The last tab is replaced by a blank project.
pub fn remove_project(app: &mut FlashApp, index: usize) {
    if index >= app.projects.len() {
        return;
    }
    app.projects.remove(index);
    app.canvas.views.clear(); // views are kept by tab position; they fit again
    if app.projects.is_empty() {
        app.projects.push(FlashApp::blank_project());
    }
    app.active_project = app.active_project.min(app.projects.len() - 1);
}

/// The window's close button: asks when there is unsaved work. Returns true when the window may close.
pub fn window_may_close(app: &mut FlashApp) -> bool {
    if app.quitting {
        return true;
    }
    let dirty = app.projects.iter().filter(|p| p.is_dirty()).count();
    if dirty == 0 {
        return true;
    }
    if app.dialog.is_none() {
        let text = if dirty == 1 {
            "There is 1 project with unsaved changes. Save it before closing?".to_owned()
        } else {
            format!("There are {dirty} projects with unsaved changes. Save them before closing?")
        };
        app.dialog = Some(Dialog::Ask(Ask::new(
            "Flash Editor",
            text,
            &[("Save all", Answer::SaveAllAndQuit), ("Don't save", Answer::Quit), ("Cancel", Answer::Nothing)],
        )));
    }
    false
}

/// Saves the next unsaved project; when none is left, closes the window.
pub fn continue_closing_window(app: &mut FlashApp, ctx: &egui::Context) {
    app.closing_window = true;
    match app.projects.iter().position(|p| p.is_dirty()) {
        Some(index) => match app.projects[index].path.clone() {
            Some(path) => save_to(app, ctx, index, path, AfterSave::ContinueClosingWindow),
            None => {
                if !save_as(app, ctx, index, AfterSave::ContinueClosingWindow) {
                    app.closing_window = false; // cancelled: stay open
                }
            }
        },
        None => quit(app, ctx),
    }
}

/// Closes the window for good.
pub fn quit(app: &mut FlashApp, ctx: &egui::Context) {
    app.quitting = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

/// File > New project: makes the project, saves it at once in the projects folder (like the C# version).
pub fn create_project(app: &mut FlashApp, ctx: &egui::Context, name: &str, pages: Vec<Page>, presentation: bool) {
    let folder = app.settings.projects_folder();
    if let Err(err) = std::fs::create_dir_all(&folder) {
        app.status = format!("Cannot create the projects folder {}: {err}", folder.display());
    }
    let path = new_project_path(&folder, name);
    let mut project = Project::from_pages(stem(&path), pages, 0);
    project.presentation = presentation;
    add_project(app, project);
    let index = app.active_project;
    save_to(app, ctx, index, path, AfterSave::Nothing);
}

/// File > Save as Template: keeps the project in `settings\templates` as a starting point for new projects.
pub fn save_as_template(app: &mut FlashApp, name: &str) {
    let folder = crate::app::paths::settings_dir().join("templates");
    let name = clean_name(name);
    let path = folder.join(format!("{name}.{}", project::EXTENSION));
    let project = app.project();
    let result = std::fs::create_dir_all(&folder).map_err(|e| e.to_string()).and_then(|()| {
        project::save_as(project.pages(), project.active_index(), project.presentation, &path)
            .map_err(|e| e.to_string())
    });
    app.status = match result {
        Ok(()) => format!("Saved as the template \"{name}\". It is in Project Manager > Templates > My templates."),
        Err(err) => format!("Could not save the template: {err}"),
    };
}

/// The names of the user's templates.
pub fn user_templates() -> Vec<String> {
    let folder = crate::app::paths::settings_dir().join("templates");
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .map(|dir| {
            dir.filter_map(Result::ok).map(|e| e.path()).filter(|p| project::is_project(p)).map(|p| stem(&p)).collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// The pages of a user template.
pub fn template_pages(name: &str) -> Result<Vec<Page>, String> {
    let path = crate::app::paths::settings_dir().join("templates").join(format!("{name}.{}", project::EXTENSION));
    project::open(path).map(|c| c.pages).map_err(|e| e.to_string())
}
