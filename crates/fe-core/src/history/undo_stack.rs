//! The list of undo steps and the position in it.

use super::size::{docs_bytes, step_bytes};
use crate::document::{Document, LayerId, Page};

/// Steps kept by default (Settings > Undo Steps).
pub const DEFAULT_STEPS: usize = 30;
/// Smallest and largest allowed number of steps.
pub const MIN_STEPS: usize = 1;
pub const MAX_STEPS: usize = 60;
/// Memory the history may use before the oldest steps are forgotten (about 1 GB).
pub const MAX_BYTES: usize = 1 << 30;

/// Two steps in a row with the same key become one (for example dragging one layer's opacity slider).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MergeKey {
    pub layer: LayerId,
    pub what: &'static str,
}

/// What a step changed.
#[derive(Clone, Debug)]
enum Change {
    /// One page's document.
    Page { before: Document, after: Document },
    /// The list of pages (added, removed, reordered, renamed pages) and which one was shown.
    Pages { before: (Vec<Page>, usize), after: (Vec<Page>, usize) },
}

/// One undoable change.
#[derive(Clone, Debug)]
pub struct Step {
    pub name: String,
    /// The page that is shown after undoing or redoing this step.
    pub page: usize,
    change: Change,
    merge: Option<MergeKey>,
    bytes: usize,
}

impl Step {
    /// Memory only this step holds.
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

/// The undo history of a project.
#[derive(Clone, Debug)]
pub struct History {
    steps: Vec<Step>,
    /// Steps `0..position` are done; the rest can be redone.
    position: usize,
    /// Position when the project was last saved (`None` = that state is gone).
    saved: Option<usize>,
    max_steps: usize,
    max_bytes: usize,
}

impl Default for History {
    fn default() -> Self {
        Self { steps: Vec::new(), position: 0, saved: Some(0), max_steps: DEFAULT_STEPS, max_bytes: MAX_BYTES }
    }
}

impl History {
    /// Records a change of page `page` from `before` to `after`. Forgets anything that could be redone.
    pub fn record(&mut self, name: &str, page: usize, before: Document, after: Document, merge: Option<MergeKey>) {
        self.steps.truncate(self.position);
        if self.saved.is_some_and(|s| s > self.position) {
            self.saved = None;
        }
        if let Some(key) = merge
            && let Some(last) = self.steps.last_mut()
            && last.merge == Some(key)
            && last.page == page
            && let Change::Page { before: last_before, after: last_after } = &mut last.change
        {
            *last_after = after;
            last.bytes = step_bytes(last_before, last_after);
            if self.saved == Some(self.position) {
                self.saved = None;
            }
            return;
        }
        let bytes = step_bytes(&before, &after);
        self.push(Step { name: name.to_owned(), page, change: Change::Page { before, after }, merge, bytes });
    }

    /// Records a change of the page list (with the shown page before and after).
    pub fn record_pages(&mut self, name: &str, before: (Vec<Page>, usize), after: (Vec<Page>, usize)) {
        self.steps.truncate(self.position);
        if self.saved.is_some_and(|s| s > self.position) {
            self.saved = None;
        }
        let bytes = docs_bytes(before.0.iter().map(|p| &p.doc), after.0.iter().map(|p| &p.doc));
        let page = after.1;
        self.push(Step { name: name.to_owned(), page, change: Change::Pages { before, after }, merge: None, bytes });
    }

    fn push(&mut self, step: Step) {
        self.steps.push(step);
        self.position = self.steps.len();
        self.trim();
    }

    /// Undoes the newest done step. Returns the page to show.
    pub fn undo(&mut self, pages: &mut Vec<Page>) -> Option<usize> {
        let step = self.steps.get(self.position.checked_sub(1)?)?;
        let shown = match &step.change {
            Change::Page { before, .. } => {
                pages.get_mut(step.page)?.doc = before.clone();
                step.page
            }
            Change::Pages { before, .. } => {
                *pages = before.0.clone();
                before.1
            }
        };
        self.position -= 1;
        Some(shown)
    }

    /// Redoes the next step. Returns the page to show.
    pub fn redo(&mut self, pages: &mut Vec<Page>) -> Option<usize> {
        let step = self.steps.get(self.position)?;
        let shown = match &step.change {
            Change::Page { after, .. } => {
                pages.get_mut(step.page)?.doc = after.clone();
                step.page
            }
            Change::Pages { after, .. } => {
                *pages = after.0.clone();
                after.1
            }
        };
        self.position += 1;
        Some(shown)
    }

    /// Goes to a position (0 = before every step), undoing or redoing as needed. Returns the last page changed.
    pub fn jump_to(&mut self, position: usize, pages: &mut Vec<Page>) -> Option<usize> {
        let position = position.min(self.steps.len());
        let mut last = None;
        while self.position > position {
            last = self.undo(pages).or(last);
        }
        while self.position < position {
            last = self.redo(pages).or(last);
        }
        last
    }

    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.position < self.steps.len()
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Name of the step Ctrl+Z would undo.
    pub fn undo_name(&self) -> Option<&str> {
        self.position.checked_sub(1).and_then(|i| self.steps.get(i)).map(|s| s.name.as_str())
    }

    /// Name of the step Ctrl+Y would redo.
    pub fn redo_name(&self) -> Option<&str> {
        self.steps.get(self.position).map(|s| s.name.as_str())
    }

    /// True when there are changes since the last save.
    pub fn is_dirty(&self) -> bool {
        self.saved != Some(self.position)
    }

    pub fn mark_saved(&mut self) {
        self.saved = Some(self.position);
    }

    /// Marks the state at `position` as saved (a background save that started at that position finished).
    pub fn mark_saved_at(&mut self, position: usize) {
        self.saved = (position <= self.steps.len()).then_some(position);
    }

    /// Marks the project as changed without an undo step (for example recovered work).
    pub fn mark_unsaved(&mut self) {
        self.saved = None;
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    /// Changes the number of steps kept (clamped to 1..60); extra old steps are forgotten.
    pub fn set_max_steps(&mut self, steps: usize) {
        self.max_steps = steps.clamp(MIN_STEPS, MAX_STEPS);
        self.trim();
    }

    /// Changes the memory limit (mainly for tests).
    pub fn set_max_bytes(&mut self, bytes: usize) {
        self.max_bytes = bytes;
        self.trim();
    }

    /// Memory held by all steps.
    pub fn total_bytes(&self) -> usize {
        self.steps.iter().map(Step::bytes).sum()
    }

    /// Forgets steps of a page that was removed, and renumbers the steps of the pages after it.
    /// (Page-list steps cannot be renumbered; when there are any, the whole history is forgotten.)
    pub fn forget_page(&mut self, page: usize) {
        if self.steps.iter().any(|s| matches!(s.change, Change::Pages { .. })) {
            self.steps.clear();
            self.position = 0;
            self.saved = None;
            return;
        }
        let mut index = 0;
        while index < self.steps.len() {
            if self.steps[index].page == page {
                self.steps.remove(index);
                if index < self.position {
                    self.position -= 1;
                }
                self.saved = None;
            } else {
                if self.steps[index].page > page {
                    self.steps[index].page -= 1;
                }
                index += 1;
            }
        }
    }

    /// Forgets the oldest done steps while there are too many or they use too much memory.
    fn trim(&mut self) {
        while self.position > 0
            && (self.steps.len() > self.max_steps || (self.total_bytes() > self.max_bytes && self.steps.len() > 1))
        {
            self.steps.remove(0);
            self.position -= 1;
            self.saved = self.saved.and_then(|s| s.checked_sub(1));
        }
    }
}
