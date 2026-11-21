//! The keyboard while text is edited: typing, Backspace / Delete, arrows (Ctrl = by word, Shift = select), Home /
//! End, Enter = new line, Ctrl+A / C / X / V, Esc = finish, Ctrl+Z = take the whole edit back.

use egui::{Event, Key, Modifiers, Ui};

use super::{TextEditor, apply, cancel, finish};
use crate::app::FlashApp;

/// What a key does to the text.
enum Change {
    None,
    Moved,
    Edited,
}

/// Handles this frame's keys and typed text. Everything else on the keyboard is ignored while editing.
pub fn keys(ui: &Ui, app: &mut FlashApp) {
    let (events, time) = ui.input(|i| (i.events.clone(), i.time));
    let ctx = ui.ctx().clone();
    if let Some(ed) = app.text_edit.as_mut().filter(|e| e.last_input < 0.0) {
        ed.last_input = time;
    }
    for event in events {
        let Some(mut ed) = app.text_edit.take() else { return };
        let change = match &event {
            Event::Text(s) => edited(ed.cursor.insert(s)),
            Event::Paste(s) => edited(ed.cursor.insert(s)),
            Event::Copy => {
                if ed.cursor.has_selection() {
                    ctx.copy_text(ed.cursor.selected_text());
                }
                Change::None
            }
            Event::Cut => {
                if ed.cursor.has_selection() {
                    ctx.copy_text(ed.cursor.selected_text());
                }
                edited(ed.cursor.backspace(false))
            }
            Event::Key { key, pressed: true, modifiers, .. } => {
                if *key == Key::Escape {
                    app.text_edit = Some(ed);
                    finish(app);
                    return;
                }
                if *key == Key::Z && modifiers.command {
                    app.text_edit = Some(ed);
                    cancel(app);
                    app.status = "Text edit undone.".into();
                    return;
                }
                key_press(&mut ed, *key, *modifiers)
            }
            _ => Change::None,
        };
        match change {
            Change::None => {}
            Change::Moved => ed.last_input = time,
            Change::Edited => {
                ed.last_input = time;
                apply(app, &mut ed);
            }
        }
        app.text_edit = Some(ed);
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(250)); // the cursor blinks
}

fn edited(changed: bool) -> Change {
    if changed { Change::Edited } else { Change::None }
}

fn key_press(ed: &mut TextEditor, key: Key, m: Modifiers) -> Change {
    let (select, word) = (m.shift, m.command);
    match key {
        Key::Enter => edited(ed.cursor.insert("\n")),
        Key::Backspace => edited(ed.cursor.backspace(word)),
        Key::Delete => edited(ed.cursor.delete(word)),
        Key::ArrowLeft => {
            ed.cursor.left(select, word);
            Change::Moved
        }
        Key::ArrowRight => {
            ed.cursor.right(select, word);
            Change::Moved
        }
        Key::ArrowUp => {
            ed.cursor.vertical(-1, select, &ed.carets);
            Change::Moved
        }
        Key::ArrowDown => {
            ed.cursor.vertical(1, select, &ed.carets);
            Change::Moved
        }
        Key::Home => {
            ed.cursor.home(select, m.command);
            Change::Moved
        }
        Key::End => {
            ed.cursor.end(select, m.command);
            Change::Moved
        }
        Key::A if m.command => {
            ed.cursor.select_all();
            Change::Moved
        }
        _ => Change::None,
    }
}
