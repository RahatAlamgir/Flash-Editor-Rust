//! Snapping for one drag (C# `SnapDrag`): the layers are looked at when the drag starts, then every mouse move asks
//! "does this snap?". The lines and gaps it snapped to are shown as pink smart guides. Alt turns snapping off for
//! a moment; the View bar's snap settings choose what to snap to.

use fe_core::document::LayerId;
use fe_core::imaging::{PixelRect, PointD};
use fe_core::layout::snapping::{SnapResult, SnapSession, SnapSettings};

use crate::app::FlashApp;

/// Snap distance on screen.
const THRESHOLD: f64 = 6.0;

/// A snapping session for a drag, or `None` when snapping is switched off.
pub fn session(app: &FlashApp, exclude: &[LayerId]) -> Option<SnapSession> {
    let a = &app.aids;
    if !a.snapping {
        return None;
    }
    let settings = SnapSettings {
        objects: a.snap_to_objects,
        grid: a.snap_to_grid && a.grid,
        canvas: a.snap_to_canvas,
        guides: a.snap_to_guides && a.guides,
        grid_size: a.grid_size,
        spacing: a.snap_to_spacing,
    };
    let page = app.project().active_page();
    Some(SnapSession::new(&page.doc, exclude, &page.vertical_guides, &page.horizontal_guides, settings))
}

/// A box moved by (dx, dy), after snapping its edges and center. Shows the guides.
pub fn snap_move(
    app: &mut FlashApp,
    s: Option<&SnapSession>,
    start: PixelRect,
    dx: f64,
    dy: f64,
    zoom: f64,
    alt: bool,
) -> (f64, f64) {
    let Some(s) = s.filter(|_| !alt && !start.is_empty()) else {
        app.canvas.snap = None;
        return (dx, dy);
    };
    let (l, t) = (start.x as f64 + dx, start.y as f64 + dy);
    let r = s.snap_box(l, t, l + start.width as f64, t + start.height as f64, THRESHOLD / zoom);
    let result = (dx + r.dx, dy + r.dy);
    app.canvas.snap = Some(r);
    result
}

/// A point (a handle being dragged) moved onto the nearest line. Shows the guides.
pub fn snap_point(app: &mut FlashApp, s: Option<&SnapSession>, p: PointD, zoom: f64, alt: bool) -> PointD {
    let Some(s) = s.filter(|_| !alt) else {
        app.canvas.snap = None;
        return p;
    };
    let r = s.snap_point(p, THRESHOLD / zoom);
    let moved = PointD::new(p.x + r.dx, p.y + r.dy);
    app.canvas.snap = Some(r);
    moved
}

/// Hides the guides (the drag ended).
pub fn end(app: &mut FlashApp) {
    app.canvas.snap = None;
}

/// Whether there is anything to draw.
pub fn shows(r: &SnapResult) -> bool {
    !r.lines_x.is_empty() || !r.lines_y.is_empty() || !r.gaps.is_empty()
}
