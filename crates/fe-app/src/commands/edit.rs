//! Undo and redo (Ctrl+Z, Ctrl+Y, the History panel buttons and the Edit tab).

use crate::app::FlashApp;

pub fn undo(app: &mut FlashApp) {
    let name = app.project().history().undo_name().map(str::to_owned);
    if app.project_mut().undo() {
        app.status = format!("Undo: {}", name.unwrap_or_default());
    }
}

pub fn redo(app: &mut FlashApp) {
    let name = app.project().history().redo_name().map(str::to_owned);
    if app.project_mut().redo() {
        app.status = format!("Redo: {}", name.unwrap_or_default());
    }
}
