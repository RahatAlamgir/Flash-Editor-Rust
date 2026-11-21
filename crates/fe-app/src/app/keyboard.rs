//! Keyboard shortcuts handled for the whole window: the active tool's keys first, tool keys (V, M, L, B...),
//! clipboard (Ctrl+C / X / V), selection (Ctrl+A, Ctrl+Shift+D / I, Delete, Esc), undo / redo, layer shortcuts,
//! view shortcuts (Ctrl+0, Ctrl+1, Ctrl++, Ctrl+-, Ctrl+R, Ctrl+', Ctrl+;, Ctrl+Shift+;), F2 (rename layer)
//! and F12 (speed overlay). Editable shortcuts come in Phase 15.

use egui::{Event, Key, Modifiers, Ui};

use super::FlashApp;
use crate::commands::{edit, file, layers, pages, selection};
use crate::tools::ToolKind;

/// Looks at this frame's key presses (ignored while typing in a text box).
/// A key press, or a clipboard shortcut (egui turns Ctrl+C / X / V into their own events).
enum Press {
    Key(Key, Modifiers),
    Copy,
    Cut,
    Paste,
}

pub fn handle(ui: &Ui, app: &mut FlashApp) {
    // Text typed on the canvas takes every key (typing, arrows, Ctrl+C / V...).
    crate::text_edit::check(app);
    if crate::text_edit::active(app) {
        crate::text_edit::keys(ui, app);
        return;
    }
    // Only typing in a text box takes the keys (the canvas itself keeps the focus after a click).
    if ui.ctx().text_edit_focused() {
        return;
    }
    let presses: Vec<Press> = ui.input(|i| {
        i.events
            .iter()
            .filter_map(|e| match e {
                Event::Key { key, pressed: true, repeat: false, modifiers, .. } => Some(Press::Key(*key, *modifiers)),
                // Arrow keys repeat while held (nudging); other keys only count once.
                Event::Key { key, pressed: true, repeat: true, modifiers, .. } if is_arrow(*key) => {
                    Some(Press::Key(*key, *modifiers))
                }
                Event::Copy => Some(Press::Copy),
                Event::Cut => Some(Press::Cut),
                Event::Paste(_) | Event::PasteImage(_) => Some(Press::Paste),
                _ => None,
            })
            .collect()
    });
    let ctx = ui.ctx().clone();
    for press in presses {
        let (key, mods) = match press {
            Press::Key(key, mods) => (key, mods),
            Press::Copy | Press::Cut | Press::Paste => {
                clipboard_shortcut(app, &press);
                continue;
            }
        };
        if !mods.command && crate::tools::key(app, key, mods) {
            continue; // the active tool used it (Enter / Esc / Backspace while drawing, [ ], arrows)
        }
        if mods.command && app.pages_ui.focused && page_shortcut(app, key) {
            continue;
        }
        if mods.command && (file_shortcut(app, &ctx, key, mods) || edit_shortcut(app, key, mods.shift)) {
            continue;
        }
        if mods.command {
            view_shortcut(app, key, mods.shift);
        } else if key == Key::Delete && app.pages_ui.focused && mods.is_none() {
            let picked = pages::picked(app);
            pages::ask_delete(app, picked);
        } else if matches!(key, Key::Delete | Key::Backspace) && mods.is_none() {
            delete_key(app);
        } else if key == Key::Escape && mods.is_none() {
            selection::deselect(app);
        } else if mods.is_none() {
            if key == Key::F12 {
                app.canvas.show_debug = !app.canvas.show_debug;
            } else if key == Key::X {
                std::mem::swap(&mut app.foreground, &mut app.background);
            } else if key == Key::D {
                app.foreground = fe_core::imaging::ColorRgba::BLACK;
                app.background = fe_core::imaging::ColorRgba::WHITE;
            } else if key == Key::F2 {
                crate::panels::right::layers::start_rename_active(app);
            } else if let Some(letter) = letter_of(key) {
                app.tools.select_by_shortcut(letter);
            }
        }
    }
}

fn is_arrow(key: Key) -> bool {
    matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown)
}

/// Ctrl+C / X / V: pages while the page row has the keyboard, else the picture.
fn clipboard_shortcut(app: &mut FlashApp, press: &Press) {
    if app.pages_ui.focused {
        let picked = pages::picked(app);
        match press {
            Press::Copy => pages::copy(app, &picked),
            Press::Cut => pages::cut(app, &picked),
            _ => {
                let after = app.project().active_index() + 1;
                pages::paste_at(app, after);
            }
        }
        return;
    }
    match press {
        Press::Copy => selection::copy(app, false),
        Press::Cut => selection::copy(app, true),
        _ => selection::paste(app),
    }
}

/// Delete / Backspace: the selected pixels; without a selection, the selected object layer (C# `MainWindow`).
fn delete_key(app: &mut FlashApp) {
    if crate::commands::parts::delete_part(app) {
        return;
    }
    if selection::delete_pixels(app) {
        return;
    }
    if app.doc().active_layer().is_some_and(|l| l.object().is_some()) {
        layers::delete(app);
    }
}

/// File shortcuts: Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S, Ctrl+W, Ctrl+Tab, Ctrl+Alt+N. True when used.
fn file_shortcut(app: &mut FlashApp, ctx: &egui::Context, key: Key, mods: Modifiers) -> bool {
    match key {
        Key::N if mods.alt => pages::add_after_current(app),
        Key::N if !mods.shift => {
            app.dialog = Some(crate::dialogs::Dialog::NewProject(crate::dialogs::NewProjectForm::new(app.background)));
        }
        Key::O => file::open_dialog(app, ctx),
        Key::S if mods.shift => {
            let index = app.active_project;
            file::save_as(app, ctx, index, crate::app::jobs::AfterSave::Nothing);
        }
        Key::S => file::save(app, ctx),
        Key::W => {
            let index = app.active_project;
            file::close_project(app, index);
        }
        Key::Tab => app.active_project = (app.active_project + 1) % app.projects.len(),
        _ => return false,
    }
    true
}

/// While the page row has the keyboard: Ctrl+D / A act on pages (Ctrl+C / X / V: see `clipboard_shortcut`).
/// True when used.
fn page_shortcut(app: &mut FlashApp, key: Key) -> bool {
    let picked = pages::picked(app);
    match key {
        Key::D => pages::duplicate(app, &picked),
        Key::A => app.pages_ui.selected = (0..app.project().pages().len()).collect(),
        _ => return false,
    }
    true
}

/// Undo / redo and layer shortcuts. Returns true when the key was used.
fn edit_shortcut(app: &mut FlashApp, key: Key, shift: bool) -> bool {
    match key {
        Key::D if shift => selection::deselect(app),
        Key::G if shift => crate::commands::objects::ungroup(app),
        Key::G => crate::commands::objects::group(app),
        Key::I if shift => selection::invert(app),
        Key::Z if shift => edit::redo(app),
        Key::Z => edit::undo(app),
        Key::Y => edit::redo(app),
        Key::D => layers::duplicate(app),
        Key::N if shift => layers::new_layer(app),
        Key::E => layers::merge(app),
        Key::L => layers::toggle_lock(app),
        Key::CloseBracket => {
            if shift {
                layers::bring_to_end(app, true)
            } else {
                layers::move_layer(app, 1)
            }
        }
        Key::OpenBracket => {
            if shift {
                layers::bring_to_end(app, false)
            } else {
                layers::move_layer(app, -1)
            }
        }
        Key::A if app.tools.active() == ToolKind::Move => layers::select_all(app),
        Key::A => selection::select_all(app),
        _ => return false,
    }
    true
}

fn view_shortcut(app: &mut FlashApp, key: Key, shift: bool) {
    match key {
        Key::Num0 => app.current_view_mut().fit(),
        Key::Num1 => app.current_view_mut().actual_size(),
        Key::Plus | Key::Equals => app.current_view_mut().zoom_in(),
        Key::Minus => app.current_view_mut().zoom_out(),
        Key::R => app.aids.rulers = !app.aids.rulers,
        Key::Quote => app.aids.grid = !app.aids.grid,
        Key::Semicolon if shift => app.aids.snapping = !app.aids.snapping,
        Key::Semicolon => app.aids.guides = !app.aids.guides,
        _ => {}
    }
}

fn letter_of(key: Key) -> Option<char> {
    let name = key.name();
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Some(c.to_ascii_uppercase()),
        _ => None,
    }
}
