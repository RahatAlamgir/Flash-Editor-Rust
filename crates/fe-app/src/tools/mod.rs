//! The tools: the catalog (names, icons, keys, groups), the toolbox state, and the tools' behaviour (one folder
//! per kind of tool). More tools arrive with the phases that need them.

pub mod ai_tools;
pub mod catalog;
pub mod crop;
pub mod curve;
pub mod moving;
pub mod options_bar;
pub mod painting;
mod pointer;
pub mod retouch;
pub mod selecting;
pub mod snap;
pub mod state;
pub mod text;

pub use catalog::ToolKind;
pub use pointer::Pointer;
pub use state::ToolState;

use crate::app::FlashApp;

/// Gives the mouse to the active tool. A move that is still going on gets the mouse whatever the tool.
pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    if crate::text_edit::active(app) && crate::text_edit::pointer(app, p) {
        return; // the mouse is editing the text
    }
    let tool = app.tools.active();
    if tool == ToolKind::Move || app.tools.mover.is_moving() {
        moving::pointer(app, p);
        return;
    }
    if !painting::pointer(app, tool, p) && !selecting::pointer(app, tool, p) {
        if retouch::pointer(app, tool, p) || ai_tools::pointer(app, tool, p) {
            return;
        }
        crop::pointer(app, tool, p);
        curve::pointer(app, tool, p);
        text::pointer(app, tool, p);
    }
}

/// Gives a key to the active tool first (Enter / Esc / Backspace while drawing a path, [ ] brush size, arrow
/// nudges). True when the tool used it.
pub fn key(app: &mut FlashApp, key: egui::Key, mods: egui::Modifiers) -> bool {
    match app.tools.active() {
        ToolKind::Move => moving::key(app, key, mods),
        _ => {
            painting::key(app, key)
                || selecting::key(app, key)
                || curve::key(app, key)
                || crop::key(app, key)
                || retouch::key(app, key)
                || ai_tools::key(app, key)
        }
    }
}

/// True for the tools that show a cross-hair cursor on the canvas.
pub fn uses_crosshair(tool: ToolKind) -> bool {
    use ToolKind::*;
    is_selection_tool(tool)
        || matches!(
            tool,
            Brush
                | Eraser
                | MaskBrush
                | Bucket
                | Eyedropper
                | Curve
                | Crop
                | PerspectiveCrop
                | CloneStamp
                | SpotHealing
                | HealingBrush
                | Patch
                | DodgeBurn
                | BlurSharpen
                | Liquify
                | RemoveBrush
                | ClickSelect
                | Refine
        )
}

/// True for the tools that make selections.
pub fn is_selection_tool(tool: ToolKind) -> bool {
    use ToolKind::*;
    matches!(tool, RectSelect | EllipseSelect | Lasso | PolygonLasso | Pen | QuickSelect | MagicWand)
}
