//! What each command of the shortcut list does. The same commands are used by the File drawer, so a shortcut, a menu
//! entry and a ribbon button always do the same thing.

use crate::app::FlashApp;
use crate::app::jobs::AfterSave;
use crate::commands::{edit, file, layers, objects, pages, selection};
use crate::dialogs::{CanvasSizeForm, Dialog, ImageSizeForm, NewProjectForm};
use crate::tools::ToolKind;

/// Which clipboard command.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Copy,
    Cut,
    Paste,
}

/// Ctrl+C / X / V: pages while the page row has the keyboard, else the picture.
pub fn clipboard(app: &mut FlashApp, what: Clip) {
    if app.pages_ui.focused {
        let picked = pages::picked(app);
        match what {
            Clip::Copy => pages::copy(app, &picked),
            Clip::Cut => pages::cut(app, &picked),
            Clip::Paste => {
                let after = app.project().active_index() + 1;
                pages::paste_at(app, after);
            }
        }
        return;
    }
    match what {
        Clip::Copy => selection::copy(app, false),
        Clip::Cut => selection::copy(app, true),
        Clip::Paste => selection::paste(app),
    }
}

/// Runs the command with this title (a title of [`super::COMMANDS`]). False for a title nothing is wired to.
pub fn run(app: &mut FlashApp, ctx: &egui::Context, title: &str) -> bool {
    match title {
        "New" => app.dialog = Some(Dialog::NewProject(NewProjectForm::new(app.background))),
        "Open" => file::open_dialog(app, ctx),
        "Save project" => file::save(app, ctx),
        "Save project as" => {
            let index = app.active_project;
            file::save_as(app, ctx, index, AfterSave::Nothing);
        }
        "Export as picture" => crate::commands::export::open_export(app),
        "Add page" => pages::add_after_current(app),
        "Close project" => {
            let index = app.active_project;
            file::close_project(app, index);
        }
        "Next picture" => app.active_project = (app.active_project + 1) % app.projects.len(),
        "Previous picture" => app.active_project = (app.active_project + app.projects.len() - 1) % app.projects.len(),
        "Undo" => edit::undo(app),
        "Redo" => edit::redo(app),
        "Copy" => clipboard(app, Clip::Copy),
        "Cut" => clipboard(app, Clip::Cut),
        "Paste" => clipboard(app, Clip::Paste),
        "Select all" => {
            if app.pages_ui.focused {
                app.pages_ui.selected = (0..app.project().pages().len()).collect();
            } else if app.tools.active() == ToolKind::Move {
                layers::select_all(app);
            } else {
                selection::select_all(app);
            }
        }
        "Deselect" => selection::deselect(app),
        "Invert selection" => selection::invert(app),
        "Image size" => {
            let d = app.doc();
            app.dialog = Some(Dialog::ImageSize(ImageSizeForm::new(d.width(), d.height())));
        }
        "Canvas size" => {
            let d = app.doc();
            app.dialog = Some(Dialog::CanvasSize(CanvasSizeForm::new(d.width(), d.height())));
        }
        "Remove background (AI)" => crate::ai::remove_bg::start(app),
        "Remove / fill selection (AI)" => crate::ai::fill::start(app),
        "Recolor" => crate::commands::effects::run(app, &fe_core::effects::RECOLOR),
        "New layer" => layers::new_layer(app),
        "Duplicate" => {
            if app.pages_ui.focused {
                let picked = pages::picked(app);
                pages::duplicate(app, &picked);
            } else {
                layers::duplicate(app);
            }
        }
        "Merge down" => layers::merge(app),
        "Lock / unlock" => layers::toggle_lock(app),
        "Group" => objects::group(app),
        "Ungroup" => objects::ungroup(app),
        "Bring forward" => layers::move_layer(app, 1),
        "Send backward" => layers::move_layer(app, -1),
        "Bring to front" => layers::bring_to_end(app, true),
        "Send to back" => layers::bring_to_end(app, false),
        "Zoom in" => app.current_view_mut().zoom_in(),
        "Zoom out" => app.current_view_mut().zoom_out(),
        "Fit on screen" => app.current_view_mut().fit(),
        "Actual size (100%)" => app.current_view_mut().actual_size(),
        "Rulers" => app.aids.rulers = !app.aids.rulers,
        "Grid" => app.aids.grid = !app.aids.grid,
        "Guides" => app.aids.guides = !app.aids.guides,
        "Snapping" => app.aids.snapping = !app.aids.snapping,
        "Keyboard shortcuts" => app.dialog = Some(Dialog::Shortcuts(crate::dialogs::ShortcutsForm::default())),
        "Rename layer" => crate::panels::right::layers::start_rename_active(app),
        "Speed overlay" => app.canvas.show_debug = !app.canvas.show_debug,
        _ => return false,
    }
    true
}
