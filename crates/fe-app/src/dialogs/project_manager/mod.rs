//! The Project Manager (C# `ProjectManagerWindow`): every project as a card with its picture, page count and
//! date; search; New project; Open; the projects folder. Right-click a card to rename it, show it in Explorer,
//! remove it from the list or delete it (to the Recycle Bin). Opens at start-up and from the Projects button.

mod card;
pub mod entry;
mod template_preview;
pub mod templates;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use egui::{RichText, TextureHandle};
use fe_ui_kit::theme::colors;

pub use entry::Entry;

use crate::app::FlashApp;
use crate::app::jobs::JobDone;
use crate::commands::file;
use crate::dialogs::{Answer, Ask, Dialog, NewProjectForm};
use card::CardAction;

/// What the Project Manager remembers.
#[derive(Default)]
pub struct ManagerState {
    pub open: bool,
    pub entries: Vec<Entry>,
    pub loading: bool,
    pub search: String,
    textures: HashMap<PathBuf, TextureHandle>,
    renaming: Option<(PathBuf, String)>,
    /// The online template browser.
    pub templates: templates::TemplatesState,
}

/// Opens the Project Manager and reads the project list in the background.
pub fn open(app: &mut FlashApp, ctx: &egui::Context) {
    app.manager.open = true;
    refresh(app, ctx);
}

/// Opens the Project Manager on the online templates (of one type, or all when `kind` is `None`).
pub fn open_templates(app: &mut FlashApp, ctx: &egui::Context, kind: Option<String>) {
    open(app, ctx);
    app.manager.templates.view = true;
    app.manager.templates.category = kind;
    app.manager.templates.wanted_category = true;
}

/// Reads the project list again (in the background).
pub fn refresh(app: &mut FlashApp, ctx: &egui::Context) {
    app.manager.loading = true;
    let recent = app.settings.recent.clone();
    let folder = app.settings.projects_folder();
    app.jobs.spawn(ctx, move || JobDone::ProjectList(entry::read_all(&recent, &folder)));
}

/// The list arrived.
pub fn on_list(app: &mut FlashApp, entries: Vec<Entry>) {
    app.manager.loading = false;
    app.manager.textures.clear();
    app.manager.entries = entries;
}

pub fn show(ctx: &egui::Context, app: &mut FlashApp) {
    if !app.manager.open {
        return;
    }
    let mut chosen: Option<(PathBuf, CardAction)> = None;
    let mut top_action = None;
    let launcher = app.launcher;
    let body = |ui: &mut egui::Ui| {
        let screen = ctx.content_rect();
        if launcher {
            // Start-up: the manager is the whole (small) window, like the C# version's Projects window.
            ui.set_width((screen.width() - 2.0 * super::frame::LAUNCHER_MARGIN.0).max(400.0));
            ui.set_height((screen.height() - 2.0 * super::frame::LAUNCHER_MARGIN.1).max(300.0));
        } else {
            ui.set_width((screen.width() - 160.0).clamp(500.0, 1180.0));
            ui.set_height((screen.height() - 160.0).clamp(400.0, 820.0));
        }
        ui.horizontal(|ui| {
            let in_templates = app.manager.templates.view;
            ui.label(RichText::new(if in_templates { "Templates" } else { "Projects" }).heading());
            ui.add_space(16.0);
            if in_templates {
                ui.label(RichText::new("Pick a ready-made design and make it your own").color(colors::SUBTLE_TEXT));
            } else {
                ui.add(egui::TextEdit::singleline(&mut app.manager.search).hint_text("Search").desired_width(220.0));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Close").clicked() {
                    top_action = Some("close");
                }
                let toggle = if app.manager.templates.view { "‹  Projects" } else { "Templates" };
                if ui.button(toggle).on_hover_text("Ready-made designs from the internet").clicked() {
                    top_action = Some("templates");
                }
                if ui.button("Open file...").clicked() {
                    top_action = Some("open");
                }
                if ui.add(egui::Button::new("New project").fill(colors::ACCENT)).clicked() {
                    top_action = Some("new");
                }
            });
        });
        if app.manager.templates.view {
            ui.separator();
            templates::show(ui, ctx, app);
            return;
        }
        ui.horizontal(|ui| {
            let folder = app.settings.projects_folder();
            ui.label(
                RichText::new(format!("New projects are saved in {}", folder.display()))
                    .size(11.5)
                    .color(colors::SUBTLE_TEXT),
            );
            if ui.small_button("Change folder...").clicked() {
                top_action = Some("folder");
            }
        });
        ui.separator();
        if app.manager.loading && app.manager.entries.is_empty() {
            ui.label("Reading the projects...");
        } else if app.manager.entries.is_empty() {
            ui.label(RichText::new("No projects yet. Press New project to start one.").color(colors::SUBTLE_TEXT));
        }
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                let search = app.manager.search.to_lowercase();
                let state = &mut app.manager;
                for entry in
                    state.entries.iter().filter(|e| search.is_empty() || e.name.to_lowercase().contains(&search))
                {
                    if !state.textures.contains_key(&entry.path)
                        && let Some(image) = &entry.preview
                    {
                        let texture =
                            ctx.load_texture(format!("pm-{}", entry.path.display()), image.clone(), Default::default());
                        state.textures.insert(entry.path.clone(), texture);
                    }
                    let renaming = match &mut state.renaming {
                        Some((p, name)) if *p == entry.path => Some(name),
                        _ => None,
                    };
                    if let Some(action) = card::show(ui, entry, state.textures.get(&entry.path), renaming) {
                        chosen = Some((entry.path.clone(), action));
                    }
                }
            });
        });
    };
    let modal = if launcher {
        super::frame::fullscreen(ctx, "project-manager", body)
    } else {
        super::frame::modal(ctx, "project-manager", body)
    };
    finish_rename(app, ctx);
    match top_action {
        Some("close") => close(app, ctx),
        Some("templates") => app.manager.templates.view = !app.manager.templates.view,
        Some("open") => {
            app.manager.open = false;
            file::open_dialog(app, ctx);
        }
        Some("new") => {
            app.manager.open = false;
            app.dialog = Some(Dialog::NewProject(NewProjectForm::new()));
        }
        Some("folder") => {
            if let Some(folder) = rfd::FileDialog::new().set_directory(app.settings.projects_folder()).pick_folder() {
                app.settings.projects_folder = folder.to_string_lossy().to_string();
                refresh(app, ctx);
            }
        }
        _ => {}
    }
    if let Some((path, action)) = chosen {
        run(app, ctx, &path, action);
    }
    if modal.should_close() && app.manager.renaming.is_none() {
        if app.manager.templates.view {
            app.manager.templates.view = false; // Esc goes back to the projects
        } else {
            close(app, ctx);
        }
    }
}

/// Close: from the editor it hides the manager; at start-up it ends the program (the manager is all there is).
fn close(app: &mut FlashApp, ctx: &egui::Context) {
    if app.launcher {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    } else {
        app.manager.open = false;
    }
}

fn run(app: &mut FlashApp, ctx: &egui::Context, path: &Path, action: CardAction) {
    match action {
        CardAction::Open => {
            app.manager.open = false;
            file::open_path(app, ctx, path, false);
        }
        CardAction::Rename => {
            let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            app.manager.renaming = Some((path.to_path_buf(), name));
        }
        CardAction::ShowInExplorer => {
            let _ = std::process::Command::new("explorer").arg(format!("/select,{}", path.display())).spawn();
        }
        CardAction::RemoveFromList => {
            app.settings.forget_recent(path);
            app.manager.entries.retain(|e| e.path != path);
        }
        CardAction::Delete => {
            let name = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            app.dialog = Some(Dialog::Ask(Ask::new(
                "Delete project",
                format!("Move \"{name}\" to the Recycle Bin? You can get it back from there."),
                &[("Delete", Answer::DeleteProjectFile(path.to_path_buf())), ("Cancel", Answer::Nothing)],
            )));
        }
    }
}

/// Enter (or a click elsewhere) renames the file; Esc cancels.
fn finish_rename(app: &mut FlashApp, ctx: &egui::Context) {
    let Some((path, name)) = app.manager.renaming.clone() else { return };
    let (enter, escape) = ctx.input(|i| (i.key_pressed(egui::Key::Enter), i.key_pressed(egui::Key::Escape)));
    let clicked = ctx.input(|i| i.pointer.any_pressed()) && !ctx.memory(|m| m.focused().is_some());
    if escape {
        app.manager.renaming = None;
    } else if enter || clicked {
        app.manager.renaming = None;
        let target = path.with_file_name(format!("{}.fe", crate::app::settings::clean_name(&name)));
        if target == path {
            return;
        }
        if target.exists() {
            app.status = "A project with that name already exists.".into();
            return;
        }
        match std::fs::rename(&path, &target) {
            Ok(()) => {
                for project in app.projects.iter_mut().filter(|p| p.path.as_deref() == Some(path.as_path())) {
                    project.path = Some(target.clone());
                    project.name = crate::app::settings::clean_name(&name);
                }
                app.settings.forget_recent(&path);
                app.settings.add_recent(&target);
                refresh(app, ctx);
            }
            Err(err) => app.status = format!("Could not rename: {err}"),
        }
    }
}

/// Moves a project file to the Recycle Bin (after the question).
pub fn delete_file(app: &mut FlashApp, path: &Path) {
    match trash::delete(path) {
        Ok(()) => {
            app.settings.forget_recent(path);
            app.manager.entries.retain(|e| e.path != path);
            app.status = "The project was moved to the Recycle Bin.".into();
        }
        Err(err) => app.status = format!("Could not delete the project: {err}"),
    }
}
