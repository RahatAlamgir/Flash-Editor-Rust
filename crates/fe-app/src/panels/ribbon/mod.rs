//! The ribbon at the top: File button, tabs, search box, collapse arrow and the buttons of the chosen tab.

mod animation;
pub mod model;
mod search;
mod tab_row;
mod view;

pub use animation::kind_of as animation_kind_of;
pub use view::show;

/// What the ribbon remembers between frames.
#[derive(Default)]
pub struct RibbonState {
    /// Index into [`model::TABS`].
    pub tab: usize,
    /// Buttons hidden (double-click a tab or the arrow at the right).
    pub collapsed: bool,
    /// Text in the "Search commands" box.
    pub search: String,
    /// Colorful icon style (Settings > Icon Style); plain gray when false.
    pub colorful: bool,
    /// The search results are showing, and the one chosen with the arrow keys.
    pub search_open: bool,
    pub search_pick: usize,
    /// The shown project is a presentation: the Animation tab is available.
    pub presentation: bool,
    /// The transition of the shown page (copied in each frame); the options boxes change it and set `transition_changed`.
    pub transition: fe_core::document::Transition,
    pub transition_changed: bool,
}

/// Something the user clicked in the ribbon that the app must handle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RibbonAction {
    /// The File button.
    File,
    /// A button, by tab / group / item index, and where it is on screen (to open its list under it).
    Button { tab: usize, group: usize, item: usize, rect: egui::Rect },
    /// An item of the File drawer, found by the search (position in `file_drawer::flat_items`).
    DrawerItem(usize),
}
