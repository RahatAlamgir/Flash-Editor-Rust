//! Patch: draw around a blemish, then drag the outlined area onto a clean part of the picture. The blemish is replaced
//! by that clean part, blended to the light and color around it. Esc cancels.

use fe_core::document::{Document, LayerId};
use fe_core::imaging::{PointD, TileMap};
use fe_core::retouch::{RetouchOp, apply_region};
use fe_core::selection::Selection;

use crate::app::FlashApp;
use crate::commands::checks;
use crate::tools::Pointer;

#[derive(Default)]
pub struct Patch {
    points: Vec<PointD>,
    drawing: bool,
    /// The outlined area (the blemish) and its outline.
    region: Option<(Selection, Vec<PointD>)>,
    drag: Option<Drag>,
}

struct Drag {
    layer: LayerId,
    original: TileMap,
    before: Document,
    start: PointD,
    offset: (i32, i32),
}

impl Patch {
    pub fn clear(&mut self) {
        self.points.clear();
        self.drawing = false;
        self.region = None;
        self.drag = None;
    }

    /// The outline to draw (moved to where it is dragged).
    pub fn outline(&self) -> Vec<PointD> {
        let (dx, dy) = self.drag.as_ref().map_or((0, 0), |d| d.offset);
        if self.drawing {
            return self.points.clone();
        }
        self.region
            .as_ref()
            .map(|(_, o)| o.iter().map(|p| PointD::new(p.x + f64::from(dx), p.y + f64::from(dy))).collect())
            .unwrap_or_default()
    }
}

/// The patch with the current drag offset, drawn into the layer from the pixels before the drag.
fn apply(app: &mut FlashApp) {
    let canvas = app.doc().bounds();
    let selection = app.doc().selected_area().cloned();
    let Some((region, _)) = app.tools.retouch.patch.region.as_ref() else { return };
    let Some(drag) = app.tools.retouch.patch.drag.as_ref() else { return };
    let b = region.bounds();
    let op = if drag.offset == (0, 0) {
        RetouchOp::Clone { dx: 0, dy: 0 }
    } else {
        RetouchOp::Heal { dx: drag.offset.0, dy: drag.offset.1, radius: (b.width.max(b.height) / 2 + 6).clamp(6, 120) }
    };
    let (region, original, layer) = (region.clone(), drag.original.clone(), drag.layer);
    if let Some(l) = app.project_mut().active_page_mut().doc.layer_mut(layer) {
        apply_region(l.pixels_mut(), &original, op, &region, selection.as_ref(), b, canvas);
    }
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    let (w, h) = (app.doc().width(), app.doc().height());
    if p.pressed {
        let inside = app
            .tools
            .retouch
            .patch
            .region
            .as_ref()
            .is_some_and(|(s, _)| s.is_selected(p.pos.x.floor() as i32, p.pos.y.floor() as i32));
        if inside {
            let Some(id) = checks::editable_layer(app) else { return };
            let Some(layer) = app.doc().layer(id) else { return };
            let original = layer.pixels().clone();
            let before = app.doc().clone();
            app.tools.retouch.patch.drag = Some(Drag { layer: id, original, before, start: p.pos, offset: (0, 0) });
            return;
        }
        let t = &mut app.tools.retouch.patch;
        t.region = None;
        t.points = vec![p.pos];
        t.drawing = true;
        return;
    }
    let t = &mut app.tools.retouch.patch;
    if t.drawing {
        let min = 1.0 / p.zoom.max(0.01);
        if p.down && t.points.last().is_none_or(|l| l.distance_to(&p.pos) >= min) {
            t.points.push(p.pos);
        }
        if p.released || !p.down {
            t.drawing = false;
            let pts = std::mem::take(&mut t.points);
            t.region = (pts.len() >= 3).then(|| Selection::from_polygon(&pts, w, h)).flatten().map(|s| (s, pts));
            if t.region.is_some() {
                app.status = "Now drag the outlined area onto a clean part of the picture.".into();
            }
        }
        return;
    }
    if t.drag.is_some() {
        if p.down {
            if let Some(d) = t.drag.as_mut() {
                d.offset =
                    ((p.pos.x - d.start.x).round_ties_even() as i32, (p.pos.y - d.start.y).round_ties_even() as i32);
            }
            apply(app);
        }
        if p.released || !p.down {
            finish(app);
        }
    }
}

fn finish(app: &mut FlashApp) {
    let Some(d) = app.tools.retouch.patch.drag.take() else { return };
    if d.offset != (0, 0) {
        app.project_mut().record("Patch", d.before);
    } else {
        app.project_mut().active_page_mut().doc = d.before;
    }
    app.tools.retouch.patch.region = None;
}

/// Esc: puts the picture back and forgets the outline. True when there was something to cancel.
pub fn cancel(app: &mut FlashApp) -> bool {
    let t = &mut app.tools.retouch.patch;
    if t.region.is_none() && !t.drawing && t.drag.is_none() {
        return false;
    }
    if let Some(d) = t.drag.take() {
        app.project_mut().active_page_mut().doc = d.before;
    }
    app.tools.retouch.patch.clear();
    true
}
