//! The painting tools: Brush, Eraser, Mask Brush (`stroke.rs`), their settings (`options.rs`) and the pictures of the brush types for the options bar (`previews.rs`).

pub mod options;
pub mod previews;
mod stroke;

use egui::Key;
use fe_core::imaging::PointD;

pub use options::BrushOptions;

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// The painting tools' state and settings.
#[derive(Default)]
pub struct PaintTools {
    pub brush: BrushOptions,
    pub eraser: BrushOptions,
    pub mask: BrushOptions,
    stroke: Option<stroke::ActiveStroke>,
    /// Where the last stroke ended (Shift+click draws a straight line from there).
    last_end: Option<PointD>,
    pub previews: previews::BrushPreviews,
}

impl PaintTools {
    /// The settings of a brush tool.
    pub fn options_mut(&mut self, tool: ToolKind) -> Option<&mut BrushOptions> {
        match tool {
            ToolKind::Brush => Some(&mut self.brush),
            ToolKind::Eraser => Some(&mut self.eraser),
            ToolKind::MaskBrush => Some(&mut self.mask),
            _ => None,
        }
    }

    /// The brush size of a brush tool (for the brush circle on the canvas).
    pub fn brush_size(&self, tool: ToolKind) -> Option<f64> {
        match tool {
            ToolKind::Brush => Some(self.brush.size),
            ToolKind::Eraser => Some(self.eraser.size),
            ToolKind::MaskBrush => Some(self.mask.size),
            _ => None,
        }
    }

    pub fn is_painting(&self) -> bool {
        self.stroke.is_some()
    }
}

/// Handles the mouse for a painting tool. False when `tool` is not one.
pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) -> bool {
    match tool {
        ToolKind::Brush | ToolKind::Eraser | ToolKind::MaskBrush => stroke::pointer(app, tool, p),
        _ => return false,
    }
    true
}

/// [ and ] change the brush size. True when the key was used.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    let tool = app.tools.active();
    let Some(options) = app.tools.paint.options_mut(tool) else { return false };
    match key {
        Key::OpenBracket => options.shrink(),
        Key::CloseBracket => options.grow(),
        _ => return false,
    }
    true
}
