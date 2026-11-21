//! Page commands (page strip, Grid view, page menu, Insert > New Page, Ctrl+Alt+N). Each change is one undo
//! step. Copied pages can be pasted into any open project.

use fe_core::document::page_ops;

use crate::app::FlashApp;
use crate::dialogs::{Answer, Ask, Dialog};

/// The pages picked in the strip, or the shown page when none is picked.
pub fn picked(app: &FlashApp) -> Vec<usize> {
    let count = app.project().pages().len();
    let mut picked: Vec<usize> = app.pages_ui.selected.iter().copied().filter(|&i| i < count).collect();
    if picked.is_empty() {
        picked.push(app.project().active_index());
    }
    picked.sort_unstable();
    picked.dedup();
    picked
}

fn after_change(app: &mut FlashApp) {
    app.pages_ui.selected = vec![app.project().active_index()];
}

/// A blank page at `index` (0 = first), the size of the shown page.
pub fn add_at(app: &mut FlashApp, index: usize) {
    app.project_mut().edit_pages("Add Page", |pages, active| page_ops::add_page(pages, active, index));
    after_change(app);
}

/// Insert > New Page / Ctrl+Alt+N: a blank page after the shown one.
pub fn add_after_current(app: &mut FlashApp) {
    let index = app.project().active_index() + 1;
    add_at(app, index);
}

pub fn duplicate(app: &mut FlashApp, indices: &[usize]) {
    let name = if indices.len() > 1 { "Duplicate Pages" } else { "Duplicate Page" };
    app.project_mut().edit_pages(name, |pages, active| page_ops::duplicate_pages(pages, active, indices));
    after_change(app);
}

/// Asks once, then deletes (one undo step brings them all back).
pub fn ask_delete(app: &mut FlashApp, indices: Vec<usize>) {
    let total = app.project().pages().len();
    if total <= 1 {
        app.status = "A project always keeps at least one page.".into();
        return;
    }
    let n = indices.len().min(total - 1);
    let text = if n == 1 {
        format!("Delete the page \"{}\"? Ctrl+Z brings it back.", app.project().pages()[indices[0]].name)
    } else {
        format!("Delete {n} pages? Ctrl+Z brings them back.")
    };
    app.dialog = Some(Dialog::Ask(Ask::new(
        "Delete pages",
        text,
        &[("Delete", Answer::DeletePages(indices)), ("Cancel", Answer::Nothing)],
    )));
}

pub fn delete_now(app: &mut FlashApp, indices: &[usize]) {
    let name = if indices.len() > 1 { "Delete Pages" } else { "Delete Page" };
    let n = app.project_mut().edit_pages(name, |pages, active| page_ops::delete_pages(pages, active, indices));
    app.canvas.views.clear();
    after_change(app);
    app.status = format!(
        "Deleted {n} page{}. Ctrl+Z brings {} back.",
        if n == 1 { "" } else { "s" },
        if n == 1 { "it" } else { "them" }
    );
}

/// Drops pages into a gap of the strip (0 = before the first page).
pub fn move_to_gap(app: &mut FlashApp, indices: &[usize], gap: usize) {
    let mut pages = app.project().pages().to_vec();
    let mut active = app.project().active_index();
    if !page_ops::move_pages(&mut pages, &mut active, indices, gap) {
        return;
    }
    let name = if indices.len() > 1 { "Move Pages" } else { "Move Page" };
    app.project_mut().edit_pages(name, |pages, active| page_ops::move_pages(pages, active, indices, gap));
    app.canvas.views.clear();
    after_change(app);
}

/// Moves one page one place left (-1) or right (+1).
pub fn nudge(app: &mut FlashApp, index: usize, direction: i32) {
    let gap = if direction < 0 { index.saturating_sub(1) } else { index + 2 };
    move_to_gap(app, &[index], gap);
}

pub fn rename(app: &mut FlashApp, index: usize, name: &str) {
    let changed = app.project().pages().get(index).is_some_and(|p| p.name != name.trim() && !name.trim().is_empty());
    if changed {
        app.project_mut().edit_pages("Rename Page", |pages, _| page_ops::rename_page(pages, index, name));
    }
}

pub fn copy(app: &mut FlashApp, indices: &[usize]) {
    app.pages_ui.clipboard = indices.iter().filter_map(|&i| app.project().pages().get(i).cloned()).collect();
    let n = app.pages_ui.clipboard.len();
    app.status = format!(
        "Copied {n} page{}. Ctrl+V pastes after the shown page, also in another project.",
        if n == 1 { "" } else { "s" }
    );
}

pub fn cut(app: &mut FlashApp, indices: &[usize]) {
    copy(app, indices);
    if app.project().pages().len() > indices.len() {
        delete_now(app, indices);
    }
}

/// Pastes copied pages at `index`.
pub fn paste_at(app: &mut FlashApp, index: usize) {
    if app.pages_ui.clipboard.is_empty() {
        app.status = "No pages copied yet: right-click a page > Copy page(s), or Ctrl+C on the page row.".into();
        return;
    }
    let clipboard = app.pages_ui.clipboard.clone();
    let name = if clipboard.len() > 1 { "Paste Pages" } else { "Paste Page" };
    app.project_mut().edit_pages(name, |pages, active| page_ops::paste_pages(pages, active, index, &clipboard));
    after_change(app);
}

/// Shows a page.
pub fn show(app: &mut FlashApp, index: usize) {
    app.project_mut().set_active_index(index);
}
