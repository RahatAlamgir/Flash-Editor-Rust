//! App state, start-up, settings, background jobs, autosave, window placement, keyboard and layout.

pub mod autosave;
pub mod clipboard;
pub mod eyedropper;
mod flash_app;
mod frame;
pub mod jobs;
mod keyboard;
mod layout;
pub mod logging;
pub mod paths;
pub mod settings;
pub mod settings_sync;
pub mod shortcuts;
mod startup;
pub mod thumbnails;
mod view_aids;
pub mod window;
mod window_placement;

pub use flash_app::FlashApp;
pub use startup::StartupOptions;
pub use view_aids::{GRID_SIZES, ViewAids};
pub use window_placement::WindowPlacement;
