//! The pages of the shown project: the strip under the canvas and the Grid view (C# `PagesBar`,
//! `PageGridView`). Click a page to open it, Ctrl+click / Shift+click picks several, double-click renames, drag
//! to change the order, right-click (or "...") for the page menu, "+" between pages adds one there.

mod card;
pub mod grid;
mod interact;
mod menu;
mod strip;

use fe_core::document::Page;

pub use strip::show;

/// What the page strip and the Grid view remember.
#[derive(Default)]
pub struct PagesUi {
    /// Picked pages (indices); the shown page counts when nothing is picked.
    pub selected: Vec<usize>,
    /// The page row has the keyboard (Ctrl+C / X / V / D / A, Delete act on pages).
    pub focused: bool,
    /// The page being renamed and its new name so far.
    pub renaming: Option<(usize, String)>,
    /// Pages being dragged, and the gap they would drop into.
    pub drag: Option<(Vec<usize>, usize)>,
    /// The Grid view covers the canvas.
    pub grid: bool,
    /// Start of a Shift+click range.
    pub anchor: Option<usize>,
    /// Copied pages (Ctrl+C), pasted with Ctrl+V into any project.
    pub clipboard: Vec<Page>,
}
