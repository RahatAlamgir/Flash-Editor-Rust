//! Click Select (AI) (C# `ClickSelectTool`): click an object to select it, or drag a box around it for a more exact
//! pick. Shift adds to the selection, Alt takes away from it.

use fe_core::imaging::PointD;

use super::Pointer;
use crate::app::FlashApp;

/// Where the mouse went down.
#[derive(Default)]
pub struct State {
    start: Option<PointD>,
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    if p.pressed {
        app.tools.ai.click.start = Some(p.pos);
    }
    if !p.released {
        return;
    }
    let Some(start) = app.tools.ai.click.start.take() else { return };
    let bounds = app.doc().bounds();
    let cx = |v: f64| v.clamp(0.0, f64::from(bounds.width - 1));
    let cy = |v: f64| v.clamp(0.0, f64::from(bounds.height - 1));
    // Less than 6 screen pixels of movement = a click.
    let moved = (p.pos.x - start.x).abs().max((p.pos.y - start.y).abs()) * p.zoom;
    if moved < 6.0 {
        crate::ai::click::start(app, vec![(cx(start.x), cy(start.y), true)], None, p.mods);
    } else {
        crate::ai::click::start(app, Vec::new(), Some((cx(start.x), cy(start.y), cx(p.pos.x), cy(p.pos.y))), p.mods);
    }
}
