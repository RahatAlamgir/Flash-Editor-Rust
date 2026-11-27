//! The canvas view: shows the active page with zoom / pan, the checkerboard, rulers, grid and guides.
//! The selection tools draw their outlines here (`selection_overlay`).

mod aids_overlay;
mod animation_badges;
pub mod bench;
mod context_menu;
mod debug_overlay;
mod input;
mod rulers;
pub mod selection_overlay;
mod view;
pub mod view_state;

use std::collections::HashMap;

use fe_gpu::{CanvasRenderer, RenderStats};
pub use view::show;
pub use view_state::ViewState;

/// A guide line being moved: vertical or not, and its index in the page's guide list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuideRef {
    pub vertical: bool,
    pub index: usize,
}

/// What the canvas remembers between frames.
#[derive(Default)]
pub struct CanvasState {
    /// The view (zoom, scroll) of each page, by (project index, page index).
    pub views: HashMap<(usize, usize), ViewState>,
    pub renderer: CanvasRenderer,
    /// A guide being dragged with the Select / Move tool.
    pub guide_drag: Option<GuideRef>,
    /// A new guide being pulled out of a ruler: `Some(true)` = vertical guide (from the left ruler).
    pub ruler_drag: Option<bool>,
    /// Where that new guide would land (canvas pixels).
    pub temp_guide: Option<f64>,
    /// The last frame's drawing statistics (F12 shows them).
    pub stats: RenderStats,
    pub show_debug: bool,
    /// Tint texture and layer bounds kept for the selection overlay.
    pub overlay: selection_overlay::OverlayCache,
    /// The selection is drawn moved by this much while its pixels are dragged.
    pub selection_offset: (i32, i32),
    /// The smart guides of the snap in progress.
    pub snap: Option<fe_core::layout::snapping::SnapResult>,
}
