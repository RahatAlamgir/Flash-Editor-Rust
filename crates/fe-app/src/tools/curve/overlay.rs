//! What the canvas draws for the Curve tool: the line being drawn and the points / handles of the curve.

use fe_core::imaging::PointD;
use fe_core::objects::CurveNode;
use fe_core::objects::curve::flatten;
use fe_core::selection::bezier::flatten_cubic;

use super::editing;
use crate::app::FlashApp;
use crate::tools::ToolKind;

/// The outline and points to draw, in canvas pixels.
pub struct CurveOverlay {
    /// The line being drawn (empty when only the points of an existing curve are shown).
    pub line: Vec<PointD>,
    pub nodes: Vec<CurveNode>,
    pub selected: Option<usize>,
    /// While drawing, every point shows its handles and the first point is bigger (it closes the shape).
    pub drawing: bool,
}

pub fn overlay(app: &FlashApp) -> Option<CurveOverlay> {
    if app.tools.active() != ToolKind::Curve {
        return None;
    }
    let c = &app.tools.curve;
    if c.is_drawing() {
        let mut line = flatten(&c.drawing, false);
        if !c.dragging
            && let Some(last) = c.drawing.last()
        {
            flatten_cubic(last.point, last.handle_out, c.mouse, c.mouse, 0.5, &mut line);
        }
        return Some(CurveOverlay { line, nodes: c.drawing.clone(), selected: None, drawing: true });
    }
    let t = editing::target(app)?;
    let selected = c.selected.filter(|(id, _)| *id == t.id).map(|(_, i)| i);
    Some(CurveOverlay { line: Vec::new(), nodes: t.nodes, selected, drawing: false })
}
