//! The selection tools: Rectangle / Ellipse Select, Lasso, Polygon Lasso, Pen, Magic Wand and Quick Select
//! (C# `Tools/Selecting`). Each file holds one tool; this one holds their shared state and the outline previews
//! the canvas draws while a selection is being made.

pub mod apply;
mod lasso;
mod marquee;
mod pen;
mod wand;

use egui::Key;
use fe_core::imaging::{PixelRect, PointD};
use fe_core::selection::bezier::Anchor;

pub use wand::{QuickOptions, WandOptions};

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// An outline shown while a selection is being drawn (canvas pixels).
#[derive(Clone, Debug)]
pub enum Preview {
    Rect(PixelRect),
    Ellipse(PixelRect),
    Path { points: Vec<PointD>, closed: bool },
}

/// The state of every selection tool.
#[derive(Default)]
pub struct SelectTools {
    marquee: marquee::Marquee,
    lasso: lasso::Lasso,
    pen: pen::Pen,
    quick_stroke: Option<wand::QuickStroke>,
    pub wand: WandOptions,
    pub quick: QuickOptions,
}

impl SelectTools {
    /// Drops anything half drawn (when another tool is picked, like C# `Deactivate`).
    pub fn cancel(&mut self) {
        self.marquee.cancel();
        self.lasso.cancel();
        self.pen.cancel();
        self.quick_stroke = None;
    }

    /// The outline to show for the active tool, if it is drawing one.
    pub fn preview(&self, tool: ToolKind, canvas: PixelRect) -> Option<Preview> {
        match tool {
            ToolKind::RectSelect => self.marquee.preview(false, canvas),
            ToolKind::EllipseSelect => self.marquee.preview(true, canvas),
            ToolKind::Lasso => self.lasso.preview(false),
            ToolKind::PolygonLasso => self.lasso.preview(true),
            ToolKind::Pen => self.pen.preview(),
            _ => None,
        }
    }

    /// The points of the Pen path being drawn (for its handles).
    pub fn pen_anchors(&self) -> &[Anchor] {
        self.pen.anchors()
    }
}

/// Handles the mouse for a selection tool. Returns false when `tool` is not one.
pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) -> bool {
    match tool {
        ToolKind::RectSelect => marquee::pointer(app, p, false),
        ToolKind::EllipseSelect => marquee::pointer(app, p, true),
        ToolKind::Lasso => lasso::freehand(app, p),
        ToolKind::PolygonLasso => lasso::polygon(app, p),
        ToolKind::Pen => pen::pointer(app, p),
        ToolKind::MagicWand => wand::magic_wand(app, p),
        ToolKind::QuickSelect => wand::quick_select(app, p),
        _ => return false,
    }
    true
}

/// Gives the active selection tool a key first (Enter / Esc / Backspace / [ ]). True when it used the key.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    match app.tools.active() {
        ToolKind::PolygonLasso => lasso::polygon_key(app, key),
        ToolKind::Pen => pen::key(app, key),
        ToolKind::QuickSelect => wand::quick_key(app, key),
        _ => false,
    }
}
