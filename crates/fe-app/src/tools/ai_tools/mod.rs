//! The AI tools and the Refine brush: Remove Brush (AI), Click Select (AI) and Refine (restore / remove). One file per
//! tool; this one holds their shared state. The AI work itself is in `crate::ai`.

mod click_select;
mod dab;
mod refine;
mod remove_brush;

use egui::Key;

pub use refine::RefineOptions;
pub use remove_brush::RemoveOptions;

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// The state and settings of the AI tools.
#[derive(Default)]
pub struct AiTools {
    pub remove: RemoveOptions,
    pub refine: RefineOptions,
    remove_stroke: Option<remove_brush::Stroke>,
    refine_stroke: Option<refine::Stroke>,
    click: click_select::State,
}

impl AiTools {
    /// Drops anything half done (when another tool is picked).
    pub fn cancel(&mut self) {
        self.remove_stroke = None;
        self.refine_stroke = None;
        self.click = click_select::State::default();
    }

    /// The brush size of a brush tool (for the brush circle on the canvas).
    pub fn brush_size(&self, tool: ToolKind) -> Option<f64> {
        match tool {
            ToolKind::RemoveBrush => Some(self.remove.size),
            ToolKind::Refine => Some(self.refine.size),
            _ => None,
        }
    }
}

/// Handles the mouse for an AI tool. False when `tool` is not one.
pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) -> bool {
    match tool {
        ToolKind::RemoveBrush => remove_brush::pointer(app, p),
        ToolKind::Refine => refine::pointer(app, p),
        ToolKind::ClickSelect => click_select::pointer(app, p),
        _ => return false,
    }
    true
}

/// Gives the active AI tool a key first. True when it used the key.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    match app.tools.active() {
        ToolKind::RemoveBrush => remove_brush::key(app, key),
        ToolKind::Refine => refine::key(app, key),
        _ => false,
    }
}
