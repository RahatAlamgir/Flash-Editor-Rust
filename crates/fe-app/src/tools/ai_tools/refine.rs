//! Refine brush (C# `RefineTool`), for fixing the result of Remove Background: Restore paints back what was removed by
//! mistake (from the picture as it was before), Remove paints away background that was missed. Alt uses the other mode
//! for one stroke; [ and ] change the size; X switches mode. One undo step per stroke.
//!
//! A cut-out is a picture object that remembers the original. The brush paints on the picture's own pixels, so it keeps
//! working after the picture was moved, resized, rotated or mirrored.

use std::time::{Duration, Instant};

use egui::Key;
use fe_core::document::{Document, LayerId, object_ops};
use fe_core::imaging::{PixelBuffer, PointD};
use fe_core::objects::{StoredObject, pictures};

use super::{Pointer, dab};
use crate::app::FlashApp;

/// Refine brush settings.
pub struct RefineOptions {
    pub size: f64,
    /// 0..100.
    pub hardness: f64,
    /// Restore (true) or Remove (false).
    pub restore: bool,
}

impl Default for RefineOptions {
    fn default() -> Self {
        Self { size: 30.0, hardness: 60.0, restore: true }
    }
}

/// A stroke in progress.
pub struct Stroke {
    layer: LayerId,
    before: Document,
    object: StoredObject,
    /// The picture as it was when the stroke began, what Restore paints from, and the picture being edited.
    base: PixelBuffer,
    original: PixelBuffer,
    work: PixelBuffer,
    /// How much the brush covered each pixel (0..1).
    coverage: Vec<f32>,
    restoring: bool,
    hardness: f64,
    /// The picture's own pixels covered by one canvas pixel.
    per_canvas: f64,
    last: Option<PointD>,
    dirty: bool,
    redrawn: Instant,
    redraw_cost: Duration,
}

/// Where a canvas point lies in the picture's own pixels.
fn to_source(object: &StoredObject, p: PointD, size: (u32, u32)) -> Option<PointD> {
    let b = object.box_()?;
    let (flip_x, flip_y) = object.flips();
    let local = b.to_local(p);
    let x = if flip_x { -local.x } else { local.x };
    let y = if flip_y { -local.y } else { local.y };
    Some(PointD::new(
        (x + b.width / 2.0) * f64::from(size.0) / b.width,
        (y + b.height / 2.0) * f64::from(size.1) / b.height,
    ))
}

fn begin(app: &mut FlashApp, p: &Pointer) -> Option<Stroke> {
    let layer = app.doc().active_layer()?;
    let (id, visible, locked) = (layer.id(), layer.visible, layer.locked);
    let Some(object) = layer.object().filter(|o| o.kind() == "image" && o.warp().is_none()).cloned() else {
        app.status =
            "Refine works on pictures, like the cut-out Remove Background leaves. On other layers use the Eraser (E)."
                .into();
        return None;
    };
    if !visible {
        app.status = "The active layer is hidden. Show it (eye icon) to edit it.".into();
        return None;
    }
    if locked {
        app.status = "This layer is locked. Unlock it with the lock icon in the Layers panel, or Ctrl+L.".into();
        return None;
    }
    let source = (*pictures::decoded(object.picture()?)?).clone();
    // What Restore paints back: the picture before its background was removed (or its current pixels).
    let original = match object.original_picture().and_then(pictures::decoded) {
        Some(o) => (*o).clone(),
        None => source.clone(),
    };
    if (original.width(), original.height()) != (source.width(), source.height()) {
        app.status = "Can't restore: the saved original does not fit this picture any more.".into();
        return None;
    }
    let b = object.box_()?;
    let per_canvas =
        (f64::from(source.width()) / b.width.max(1e-9) * f64::from(source.height()) / b.height.max(1e-9)).sqrt();
    let o = &app.tools.ai.refine;
    let restoring = o.restore != p.mods.alt; // Alt = the other mode, just for this stroke
    let coverage = vec![0.0; source.width() as usize * source.height() as usize];
    Some(Stroke {
        layer: id,
        before: app.doc().clone(),
        object,
        base: source.clone(),
        original,
        work: source,
        coverage,
        restoring,
        hardness: (o.hardness / 100.0).clamp(0.0, 1.0),
        per_canvas,
        last: None,
        dirty: false,
        redrawn: Instant::now(),
        redraw_cost: Duration::ZERO,
    })
}

/// One round dab on the picture's own pixels.
fn dab_at(s: &mut Stroke, cx: f64, cy: f64, radius: f64) {
    let (w, h) = (s.work.width() as usize, s.work.height() as usize);
    let x0 = (cx - radius).floor().max(0.0) as usize;
    let x1 = ((cx + radius).ceil() as i64).clamp(0, w as i64 - 1) as usize;
    let y0 = (cy - radius).floor().max(0.0) as usize;
    let y1 = ((cy + radius).ceil() as i64).clamp(0, h as i64 - 1) as usize;
    if cx + radius < 0.0 || cy + radius < 0.0 || cx - radius > w as f64 || cy - radius > h as f64 {
        return;
    }
    let solid = radius * s.hardness;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let d = ((x as f64 + 0.5 - cx).powi(2) + (y as f64 + 0.5 - cy).powi(2)).sqrt();
            let c = if d <= solid {
                1.0
            } else if d < radius {
                (radius - d) / (radius - solid).max(1e-9)
            } else {
                continue;
            } as f32;
            let i = y * w + x;
            if c <= s.coverage[i] {
                continue;
            }
            s.coverage[i] = c;
            let k = i * 4;
            let cov = f64::from(c);
            if s.restoring {
                let (b, o) = (&s.base.data()[k..k + 4], &s.original.data()[k..k + 4]);
                // Where nothing was left, the colors come from the original.
                let from: [f64; 3] = if b[3] == 0 {
                    [f64::from(o[0]), f64::from(o[1]), f64::from(o[2])]
                } else {
                    [f64::from(b[0]), f64::from(b[1]), f64::from(b[2])]
                };
                let out = [
                    from[0] + (f64::from(o[0]) - from[0]) * cov,
                    from[1] + (f64::from(o[1]) - from[1]) * cov,
                    from[2] + (f64::from(o[2]) - from[2]) * cov,
                    f64::from(b[3]) + (f64::from(o[3]) - f64::from(b[3])) * cov,
                ];
                for (d, v) in s.work.data_mut()[k..k + 4].iter_mut().zip(out) {
                    *d = v.round().clamp(0.0, 255.0) as u8;
                }
            } else {
                let a = f64::from(s.base.data()[k + 3]) * (1.0 - cov);
                s.work.data_mut()[k + 3] = a.round().clamp(0.0, 255.0) as u8;
            }
            s.dirty = true;
        }
    }
}

/// Shows the edited picture on the layer (at most as often as the redraws allow).
fn redraw(app: &mut FlashApp, s: &mut Stroke, force: bool) {
    if !s.dirty {
        return;
    }
    let wait = Duration::from_millis(35).max(s.redraw_cost * 3);
    if !force && s.redrawn.elapsed() < wait {
        return;
    }
    let started = Instant::now();
    let canvas = app.doc().bounds();
    let object = s.object.with_source(&s.work);
    if let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(s.layer) {
        object_ops::set_object(layer, object, canvas);
    }
    s.dirty = false;
    s.redraw_cost = started.elapsed();
    s.redrawn = Instant::now();
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    if p.pressed {
        app.tools.ai.refine_stroke = begin(app, p);
    }
    let Some(mut s) = app.tools.ai.refine_stroke.take() else { return };
    let size = app.tools.ai.refine.size;
    let (sw, sh) = (s.work.width(), s.work.height());
    let (object, last, per_canvas) = (s.object.clone(), s.last, s.per_canvas);
    let radius = (size * per_canvas).max(1.0) / 2.0;
    // The dabs are placed in canvas steps, then mapped onto the picture's pixels.
    let mut points = Vec::new();
    dab::along(last, p.pos, size, |x, y| points.push(PointD::new(x, y)));
    for point in points {
        if let Some(at) = to_source(&object, point, (sw, sh)) {
            dab_at(&mut s, at.x, at.y, radius);
        }
    }
    s.last = Some(p.pos);
    if p.released || !p.down {
        redraw(app, &mut s, true);
        let name = if s.restoring { "Refine: Restore" } else { "Refine: Remove" };
        if app.doc().layer(s.layer).is_some() && s.coverage.iter().any(|c| *c > 0.0) {
            app.project_mut().record(name, s.before);
        }
        return;
    }
    redraw(app, &mut s, false);
    app.tools.ai.refine_stroke = Some(s);
}

/// [ ] change the size, X switches between Restore and Remove.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    let o = &mut app.tools.ai.refine;
    match key {
        Key::OpenBracket => o.size = (o.size * 0.8).round().clamp(2.0, 500.0),
        Key::CloseBracket => o.size = (o.size * 1.25).round().clamp(2.0, 500.0),
        Key::X => o.restore = !o.restore,
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use fe_core::imaging::ColorRgba;
    use fe_core::objects::ObjectBox;

    #[test]
    fn canvas_points_map_onto_the_pictures_own_pixels_even_when_turned_and_scaled() {
        let pic = PixelBuffer::filled(100, 50, ColorRgba::rgb(1, 2, 3));
        // The picture is shown at 200 x 100 with its top-left corner at (10, 20).
        let object = StoredObject::new_cutout(ObjectBox::new(10.0, 20.0, 200.0, 100.0, 0.0), &pic, &pic, "p");
        let top_left = to_source(&object, PointD::new(10.0, 20.0), (100, 50)).expect("a point");
        assert!(top_left.x.abs() < 1e-9 && top_left.y.abs() < 1e-9);
        let middle = to_source(&object, PointD::new(110.0, 70.0), (100, 50)).expect("a point");
        assert!((middle.x - 50.0).abs() < 1e-9 && (middle.y - 25.0).abs() < 1e-9);
        let mut flipped = object.clone();
        flipped.set_flips(true, false);
        let mirrored = to_source(&flipped, PointD::new(10.0, 20.0), (100, 50)).expect("a point");
        assert!((mirrored.x - 100.0).abs() < 1e-9);
    }
}
