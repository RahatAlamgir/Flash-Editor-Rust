//! Page commands of the page strip (C# `PageCommands`): add, duplicate, delete, move, rename, paste. They work
//! on the page list of a project; the app wraps each in `Project::edit_pages` so it is one undo step.

use super::{Document, Page};
use crate::imaging::ColorRgba;

/// A blank page the size of `like`, named "Page N" (N = first free number).
pub fn blank_page_like(pages: &[Page], like: &Document) -> Page {
    let mut n = pages.len() + 1;
    while pages.iter().any(|p| p.name == format!("Page {n}")) {
        n += 1;
    }
    Page::new(format!("Page {n}"), Document::blank(like.width(), like.height(), ColorRgba::WHITE))
}

/// Adds a blank page at `index` (0 = first) and shows it.
pub fn add_page(pages: &mut Vec<Page>, active: &mut usize, index: usize) {
    let page = blank_page_like(pages, &pages[(*active).min(pages.len() - 1)].doc);
    let index = index.min(pages.len());
    pages.insert(index, page);
    *active = index;
}

/// Copies of the pages `indices` ("Name copy"), inserted after the last of them; the first copy is shown.
pub fn duplicate_pages(pages: &mut Vec<Page>, active: &mut usize, indices: &[usize]) {
    let copies: Vec<Page> =
        indices.iter().filter_map(|&i| pages.get(i)).map(|p| copy_of(p, &format!("{} copy", p.name))).collect();
    let after = indices.iter().max().map_or(pages.len(), |m| m + 1).min(pages.len());
    let count = copies.len();
    pages.splice(after..after, copies);
    if count > 0 {
        *active = after;
    }
}

/// Inserts copies of `clipboard` pages at `index` (paste); the first pasted page is shown.
pub fn paste_pages(pages: &mut Vec<Page>, active: &mut usize, index: usize, clipboard: &[Page]) {
    let index = index.min(pages.len());
    let copies: Vec<Page> = clipboard.iter().map(|p| copy_of(p, &p.name)).collect();
    if copies.is_empty() {
        return;
    }
    pages.splice(index..index, copies);
    *active = index;
}

/// Deletes the pages `indices`. The last page can never be deleted (returns how many were deleted).
pub fn delete_pages(pages: &mut Vec<Page>, active: &mut usize, indices: &[usize]) -> usize {
    let mut doomed: Vec<usize> = indices.iter().copied().filter(|&i| i < pages.len()).collect();
    doomed.sort_unstable();
    doomed.dedup();
    if doomed.len() >= pages.len() {
        doomed.remove(0);
    }
    for &i in doomed.iter().rev() {
        pages.remove(i);
    }
    let first = doomed.first().copied().unwrap_or(*active);
    *active = first.saturating_sub(1).min(pages.len() - 1);
    doomed.len()
}

/// Moves the pages `indices` so they start at gap `gap` (0 = before the first page); they keep their order
/// and the shown page follows. Returns false when nothing moves.
pub fn move_pages(pages: &mut Vec<Page>, active: &mut usize, indices: &[usize], gap: usize) -> bool {
    let mut moved: Vec<usize> = indices.iter().copied().filter(|&i| i < pages.len()).collect();
    moved.sort_unstable();
    moved.dedup();
    if moved.is_empty() {
        return false;
    }
    let gap = gap.min(pages.len());
    let insert_at = (0..gap).filter(|i| !moved.contains(i)).count();
    let shown_name_index = *active;
    let mut order: Vec<usize> = (0..pages.len()).filter(|i| !moved.contains(i)).collect();
    order.splice(insert_at..insert_at, moved.iter().copied());
    if order.iter().copied().eq(0..pages.len()) {
        return false;
    }
    let old = std::mem::take(pages);
    let mut slots: Vec<Option<Page>> = old.into_iter().map(Some).collect();
    *pages = order.iter().filter_map(|&i| slots[i].take()).collect();
    *active = order.iter().position(|&i| i == shown_name_index).unwrap_or(0);
    true
}

/// Renames a page (empty names are ignored). Returns false when nothing changed.
pub fn rename_page(pages: &mut [Page], index: usize, name: &str) -> bool {
    let name = name.trim();
    match pages.get_mut(index) {
        Some(page) if !name.is_empty() && page.name != name => {
            page.name = name.to_owned();
            true
        }
        _ => false,
    }
}

/// A copy of a page with its own document identity (so caches and thumbnails treat it as a new page).
pub fn copy_of(page: &Page, name: &str) -> Page {
    let mut doc = Document::new(page.doc.width(), page.doc.height());
    let layers: Vec<_> = page.doc.layers().iter().map(|l| l.duplicate(l.name.clone())).collect();
    let active = page.doc.active_index().and_then(|i| layers.get(i)).map(|l| l.id());
    doc.set_layers(layers, active);
    let mut copy = Page::new(name, doc);
    copy.vertical_guides = page.vertical_guides.clone();
    copy.horizontal_guides = page.horizontal_guides.clone();
    copy.transition = page.transition;
    // The copied layers have new ids: the animations follow their layers.
    copy.animations = page
        .animations
        .iter()
        .filter_map(|a| {
            let index = page.doc.index_of(a.layer)?;
            let layer = copy.doc.layers().get(index)?;
            Some(super::animation::Animation { layer: layer.id(), ..*a })
        })
        .collect();
    copy
}
