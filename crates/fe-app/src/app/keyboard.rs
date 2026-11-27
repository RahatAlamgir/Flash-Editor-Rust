//! Keyboard shortcuts handled for the whole window: the active tool's keys first, tool keys (V, M, L, B...),
//! clipboard (Ctrl+C / X / V), selection (Ctrl+A, Ctrl+Shift+D / I, Delete, Esc), undo / redo, layer shortcuts,
//! view shortcuts (Ctrl+0, Ctrl+1, Ctrl++, Ctrl+-, Ctrl+R, Ctrl+', Ctrl+;, Ctrl+Shift+;), F2 (rename layer)
//! and F12 (speed overlay). The commands with a shortcut are in `shortcuts`, where the user can change them.

use egui::{Event, Key, Modifiers, Ui};

use super::FlashApp;
use super::shortcuts::{self, run::Clip, run::clipboard};
use crate::commands::{edit, file, layers, pages, selection};

/// Looks at this frame's key presses (ignored while typing in a text box).
/// A key press, or a clipboard shortcut (egui turns Ctrl+C / X / V into their own events).
enum Press {
    Key(Key, Modifiers),
    Copy,
    Cut,
    Paste,
}

pub fn handle(ui: &Ui, app: &mut FlashApp) {
    // The shortcut list is waiting for new keys: they are not commands now.
    if matches!(app.dialog, Some(crate::dialogs::Dialog::Shortcuts(_))) {
        return;
    }
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
            Press::Copy => {
                clipboard(app, Clip::Copy);
                continue;
            }
            Press::Cut => {
                clipboard(app, Clip::Cut);
                continue;
            }
            Press::Paste => {
                clipboard(app, Clip::Paste);
                continue;
            }
        };
        if !mods.command && crate::tools::key(app, key, mods) {
            continue; // the active tool used it (Enter / Esc / Backspace while drawing, [ ], arrows)
        }
        // The commands with a shortcut (the user can change them in the shortcut list).
        if let Some(title) = shortcuts::find(&app.settings, key, mods) {
            shortcuts::run(app, &ctx, title);
            continue;
        }
        // The usual extra ways to do the same.
        if mods.command && key == Key::Z && mods.shift {
            edit::redo(app);
            continue;
        }
        if mods.command && key == Key::F4 {
            let index = app.active_project;
            file::close_project(app, index);
            continue;
        }
        if mods.command {
            continue;
        }
        if key == Key::Delete && app.pages_ui.focused && mods.is_none() {
            let picked = pages::picked(app);
            pages::ask_delete(app, picked);
        } else if matches!(key, Key::Delete | Key::Backspace) && mods.is_none() {
            delete_key(app);
        } else if key == Key::Escape && mods.is_none() {
            selection::deselect(app);
        } else if mods.is_none() {
            if key == Key::X {
                std::mem::swap(&mut app.foreground, &mut app.background);
            } else if key == Key::D {
                app.foreground = fe_core::imaging::ColorRgba::BLACK;
                app.background = fe_core::imaging::ColorRgba::WHITE;
            } else if let Some(letter) = letter_of(key) {
                app.tools.select_by_shortcut(letter);
            }
        }
    }
}

fn is_arrow(key: Key) -> bool {
    matches!(key, Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown)
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

fn letter_of(key: Key) -> Option<char> {
    let name = key.name();
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphabetic() => Some(c.to_ascii_uppercase()),
        _ => None,
    }
}
