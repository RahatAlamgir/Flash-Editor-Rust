//! A project: one or more pages, each with its own document and guide lines, one undo history for all of
//! them, and the `.flash` file it is saved in (if any).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::Document;
use crate::history::{History, MergeKey};

/// One page of a project.
#[derive(Clone, Debug)]
pub struct Page {
    pub name: String,
    pub doc: Document,
    /// x positions of vertical guide lines (canvas pixels).
    pub vertical_guides: Vec<f64>,
    /// y positions of horizontal guide lines (canvas pixels).
    pub horizontal_guides: Vec<f64>,
    /// How the slideshow arrives at this page (presentations).
    pub transition: super::Transition,
    /// Effects on the layers during the slideshow, in the order they play.
    pub animations: Vec<super::animation::Animation>,
}

impl Page {
    pub fn new(name: impl Into<String>, doc: Document) -> Self {
        Self {
            name: name.into(),
            doc,
            vertical_guides: Vec::new(),
            horizontal_guides: Vec::new(),
            transition: super::Transition::default(),
            animations: Vec::new(),
        }
    }

    /// The animations whose layer still exists (a deleted layer leaves its animations behind until they are cleaned).
    pub fn valid_animations(&self) -> Vec<super::animation::Animation> {
        self.animations.iter().filter(|a| self.doc.index_of(a.layer).is_some()).copied().collect()
    }
}

/// A project with its pages. Always has at least one page.
#[derive(Clone, Debug)]
pub struct Project {
    /// Identifies the project for background work (saving), even when tabs move.
    uid: u64,
    pub name: String,
    /// The `.flash` file, once saved or opened.
    pub path: Option<PathBuf>,
    pages: Vec<Page>,
    active_page: usize,
    history: History,
    /// A presentation: the Animation tab (slide transitions, slideshow) is available.
    pub presentation: bool,
}

impl Project {
    /// A project with one page.
    pub fn new(name: impl Into<String>, first_page: Page) -> Self {
        Self::from_pages(name, vec![first_page], 0)
    }

    /// A project with these pages (at least one; an empty list gets a small blank page).
    pub fn from_pages(name: impl Into<String>, mut pages: Vec<Page>, active_page: usize) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        if pages.is_empty() {
            pages.push(Page::new("Page 1", Document::blank(800, 600, crate::imaging::ColorRgba::WHITE)));
        }
        let active_page = active_page.min(pages.len() - 1);
        Self {
            uid: NEXT.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            path: None,
            pages,
            active_page,
            history: History::default(),
            presentation: false,
        }
    }

    pub fn uid(&self) -> u64 {
        self.uid
    }

    /// True when there are changes that are not saved.
    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
    }

    /// Changes the list of pages as one undo step (add, delete, reorder, rename pages). `change` gets the
    /// pages and the index of the shown page.
    pub fn edit_pages<R>(&mut self, name: &str, change: impl FnOnce(&mut Vec<Page>, &mut usize) -> R) -> R {
        let before = (self.pages.clone(), self.active_page);
        let result = change(&mut self.pages, &mut self.active_page);
        if self.pages.is_empty() {
            self.pages = before.0.clone(); // a project always keeps a page
        }
        self.active_page = self.active_page.min(self.pages.len() - 1);
        self.history.record_pages(name, before, (self.pages.clone(), self.active_page));
        result
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }

    /// Changes the active page's document as one undo step called `name`. `merge` joins it with the previous
    /// step when both have the same key (slider drags).
    pub fn edit<R>(&mut self, name: &str, merge: Option<MergeKey>, change: impl FnOnce(&mut Document) -> R) -> R {
        let page = self.active_page;
        let before = self.pages[page].doc.clone();
        let result = change(&mut self.pages[page].doc);
        let after = self.pages[page].doc.clone();
        self.history.record(name, page, before, after, merge);
        result
    }

    /// Records a change that was already made to the active page (for example while dragging) as one undo step,
    /// from `before` to the page's document now.
    pub fn record(&mut self, name: &str, before: Document) {
        let page = self.active_page;
        let after = self.pages[page].doc.clone();
        self.history.record(name, page, before, after, None);
    }

    /// Undoes the newest step (and shows its page). False when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.history.undo(&mut self.pages) {
            Some(page) => {
                self.active_page = page;
                true
            }
            None => false,
        }
    }

    /// Redoes the next step (and shows its page). False when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        match self.history.redo(&mut self.pages) {
            Some(page) => {
                self.active_page = page;
                true
            }
            None => false,
        }
    }

    /// Goes to a point in the history (History panel click).
    pub fn jump_to(&mut self, position: usize) {
        if let Some(page) = self.history.jump_to(position, &mut self.pages) {
            self.active_page = page;
        }
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn pages_mut(&mut self) -> &mut [Page] {
        &mut self.pages
    }

    pub fn active_index(&self) -> usize {
        self.active_page
    }

    pub fn active_page(&self) -> &Page {
        &self.pages[self.active_page]
    }

    pub fn active_page_mut(&mut self) -> &mut Page {
        &mut self.pages[self.active_page]
    }

    /// Opens another page (index is clamped).
    pub fn set_active_index(&mut self, index: usize) {
        self.active_page = index.min(self.pages.len() - 1);
    }

    /// Inserts a page at `index` and opens it.
    pub fn insert_page(&mut self, index: usize, page: Page) {
        let index = index.min(self.pages.len());
        self.pages.insert(index, page);
        self.active_page = index;
    }

    /// Removes a page; the last page can never be removed (returns `None`).
    pub fn remove_page(&mut self, index: usize) -> Option<Page> {
        if self.pages.len() <= 1 || index >= self.pages.len() {
            return None;
        }
        let page = self.pages.remove(index);
        self.history.forget_page(index);
        if self.active_page >= self.pages.len() || self.active_page > index {
            self.active_page = self.active_page.saturating_sub(1).min(self.pages.len() - 1);
        }
        Some(page)
    }
}
