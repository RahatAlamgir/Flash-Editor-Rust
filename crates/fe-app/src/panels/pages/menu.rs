//! The page menu (right-click or "..."), like the C# page context menu.

use egui::Ui;

use crate::app::FlashApp;
use crate::commands::pages as cmd;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageItem {
    Rename,
    Copy,
    PasteAfter,
    PasteBefore,
    AddAfter,
    AddBefore,
    Duplicate,
    MoveLeft,
    MoveRight,
    Delete,
}

/// Draws the menu for `picked` pages; returns the choice.
pub fn show(ui: &mut Ui, app: &FlashApp, picked: &[usize]) -> Option<PageItem> {
    let several = picked.len() > 1;
    let has_clip = !app.pages_ui.clipboard.is_empty();
    let count = app.project().pages().len();
    let mut chosen = None;
    let mut item = |ui: &mut Ui, text: &str, keys: &str, value: PageItem, enabled: bool| {
        if ui.add_enabled(enabled, egui::Button::new(text).shortcut_text(keys)).clicked() {
            chosen = Some(value);
            ui.close();
        }
    };
    ui.set_min_width(230.0);
    item(ui, "Rename", "", PageItem::Rename, !several);
    ui.separator();
    item(ui, if several { "Copy pages" } else { "Copy page" }, "Ctrl+C", PageItem::Copy, true);
    item(ui, "Paste after this", "Ctrl+V", PageItem::PasteAfter, has_clip);
    item(ui, "Paste before this", "", PageItem::PasteBefore, has_clip);
    ui.separator();
    item(ui, "Add new page after", "Ctrl+Alt+N", PageItem::AddAfter, true);
    item(ui, "Add new page before", "", PageItem::AddBefore, true);
    item(ui, if several { "Duplicate pages" } else { "Duplicate page" }, "Ctrl+D", PageItem::Duplicate, true);
    ui.separator();
    item(ui, "Move left", "", PageItem::MoveLeft, !several && picked.first().is_some_and(|&i| i > 0));
    item(ui, "Move right", "", PageItem::MoveRight, !several && picked.first().is_some_and(|&i| i + 1 < count));
    ui.separator();
    item(ui, if several { "Delete pages" } else { "Delete page" }, "Delete", PageItem::Delete, count > 1);
    chosen
}

/// Runs a menu choice; `index` = the page the menu belongs to.
pub fn run(app: &mut FlashApp, index: usize, picked: &[usize], item: PageItem) {
    match item {
        PageItem::Rename => {
            let name = app.project().pages()[index].name.clone();
            app.pages_ui.renaming = Some((index, name));
        }
        PageItem::Copy => cmd::copy(app, picked),
        PageItem::PasteAfter => cmd::paste_at(app, index + 1),
        PageItem::PasteBefore => cmd::paste_at(app, index),
        PageItem::AddAfter => cmd::add_at(app, index + 1),
        PageItem::AddBefore => cmd::add_at(app, index),
        PageItem::Duplicate => cmd::duplicate(app, picked),
        PageItem::MoveLeft => cmd::nudge(app, index, -1),
        PageItem::MoveRight => cmd::nudge(app, index, 1),
        PageItem::Delete => cmd::ask_delete(app, picked.to_vec()),
    }
}
