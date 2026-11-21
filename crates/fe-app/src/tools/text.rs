//! The Text tool (C# `TextTool`): click on the canvas and type; clicking existing text edits its words instead.

use fe_core::imaging::PointD;

use super::{Pointer, ToolKind};
use crate::app::FlashApp;

/// The Text tool's settings (C# `TextToolOptions`) and the click waiting to be finished.
pub struct TextTool {
    pub family: String,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pending: Option<PointD>,
}

impl Default for TextTool {
    fn default() -> Self {
        Self { family: "Segoe UI".into(), size: 48.0, bold: false, italic: false, pending: None }
    }
}

pub fn pointer(app: &mut FlashApp, tool: ToolKind, p: &Pointer) {
    if tool != ToolKind::Text {
        return;
    }
    if p.pressed {
        app.tools.text.pending = Some(p.pos);
    }
    // The box opens when the button is released, so the canvas has let go of the mouse first.
    if p.released
        && let Some(at) = app.tools.text.pending.take()
    {
        let hit = super::moving::hit::layer_at(app.doc(), at, 3.0 / p.zoom, false)
            .filter(|id| app.doc().layer(*id).and_then(|l| l.object()).is_some_and(|o| o.kind() == "text"));
        match hit {
            Some(id) => {
                crate::commands::parts::select_part(app, None);
                crate::text_edit::begin(app, id, None, crate::text_edit::Start::At(at, false));
            }
            None => crate::text_edit::create_new(app, at),
        }
    }
}
