//! Liquify: drag to push the pixels along, or click and hold to swell (Bloat) or pinch (Pucker) what is under the brush.
//! Works on the pixels of the active layer.

use fe_core::document::{Document, LayerId};
use fe_core::imaging::{PixelRect, PointD};
use fe_core::retouch::{LiquifyMode, liquify};

use crate::app::FlashApp;
use crate::commands::checks;
use crate::tools::Pointer;

pub struct Active {
    layer: LayerId,
    before: Document,
    last: PointD,
    dirty: PixelRect,
}

fn step(app: &mut FlashApp, a: &mut Active, at: PointD, dx: f64, dy: f64) {
    let o = app.tools.retouch.liquify.clone();
    // Bloat and Pucker act a little at every step: a slower, softer effect than one big jump.
    let strength = if o.mode == LiquifyMode::Push { o.strength / 100.0 } else { o.strength / 100.0 * 0.35 };
    let canvas = app.doc().bounds();
    let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(a.layer) else { return };
    let area = liquify::apply(layer.pixels_mut(), canvas, at.x, at.y, o.size / 2.0, strength, o.mode, dx, dy);
    if !area.is_empty() {
        a.dirty = if a.dirty.is_empty() { area } else { a.dirty.union(&area) };
    }
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    if p.pressed {
        let Some(id) = checks::editable_layer(app) else { return };
        let mut a = Active { layer: id, before: app.doc().clone(), last: p.pos, dirty: PixelRect::EMPTY };
        if app.tools.retouch.liquify.mode != LiquifyMode::Push {
            step(app, &mut a, p.pos, 0.0, 0.0);
        }
        app.tools.retouch.liq = Some(a);
        return;
    }
    let Some(mut a) = app.tools.retouch.liq.take() else { return };
    let (dx, dy) = (p.pos.x - a.last.x, p.pos.y - a.last.y);
    if app.tools.retouch.liquify.mode == LiquifyMode::Push {
        if dx.abs() + dy.abs() >= 0.5 {
            step(app, &mut a, p.pos, dx, dy);
            a.last = p.pos;
        }
    } else if dx.abs() + dy.abs() >= 1.0 {
        step(app, &mut a, p.pos, 0.0, 0.0);
        a.last = p.pos;
    }
    if p.released || !p.down {
        if !a.dirty.is_empty() {
            app.project_mut().record("Liquify", a.before);
        }
    } else {
        app.tools.retouch.liq = Some(a);
    }
}
