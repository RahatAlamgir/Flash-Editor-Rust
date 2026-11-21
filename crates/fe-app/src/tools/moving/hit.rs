//! Which layer is under the mouse (C# `MainViewModel.FindLayerAt`) and which layers a rubber band touches
//! (C# `MoveTool.LayersTouching`).

use fe_core::document::{Document, LayerId};
use fe_core::imaging::{PixelRect, PointD};
use fe_core::objects::handles;

/// The top-most layer with a visible pixel at `point` (within `margin` pixels for objects). Hidden, almost fully
/// see-through and adjustment layers are skipped, and locked ones unless `include_locked`.
pub fn layer_at(doc: &Document, point: PointD, margin: f64, include_locked: bool) -> Option<LayerId> {
    let (x, y) = (point.x.floor() as i32, point.y.floor() as i32);
    for layer in doc.layers().iter().rev() {
        if !layer.visible || layer.opacity() <= 0.02 || layer.is_adjustment() || (layer.locked && !include_locked) {
            continue;
        }
        let shown = |x: i32, y: i32| {
            let mask = layer.effective_mask().map_or(1.0, |m| m.coverage(x, y));
            doc.bounds().contains(x, y) && layer.pixels().pixel(x, y).a as f64 * mask > 16.0
        };
        let pixel_hit = || {
            let reach = margin.ceil().min(8.0) as i32;
            (-reach..=reach).any(|dy| (-reach..=reach).any(|dx| shown(x + dx, y + dy)))
        };
        let hit = match layer.object() {
            // A picture (or a reshaped object) counts where it has visible pixels, so the removed part of a cut-out
            // can be clicked through; other objects count anywhere inside their box.
            Some(obj) if obj.kind() == "image" || handles::has_custom_outline(obj) => pixel_hit(),
            Some(obj) => match obj.box_() {
                Some(b) => {
                    let extra = if handles::is_line_like(Some(obj)) { 4.0 } else { 0.0 };
                    let inside = b.contains(point, margin + extra);
                    // A layer with a mask (a picture in a frame) counts only where the mask lets it show.
                    let masked = layer
                        .effective_mask()
                        .is_some_and(|m| !doc.bounds().contains(x, y) || m.coverage(x, y) <= 0.05);
                    inside && !masked
                }
                None => pixel_hit(),
            },
            None => shown(x, y),
        };
        if hit {
            return Some(layer.id());
        }
    }
    None
}

/// The top-most part of a group under `point` (its index in the group), by the part's box.
pub fn part_at(group: &fe_core::objects::StoredObject, point: PointD, margin: f64) -> Option<usize> {
    let parts = group.current_members()?;
    parts.iter().enumerate().rev().find_map(|(i, m)| {
        let b = m.object.box_()?;
        let extra = if handles::is_line_like(Some(&m.object)) { 4.0 } else { 0.0 };
        b.contains(point, margin + extra).then_some(i)
    })
}

/// Visible, unlocked layers whose content touches `rect`. A painted layer that covers the whole canvas (a
/// background) is not picked, or every rubber band would select it.
pub fn layers_touching(doc: &Document, rect: PixelRect) -> Vec<LayerId> {
    let mut found = Vec::new();
    for layer in doc.layers() {
        if !layer.visible || layer.locked || layer.opacity() <= 0.02 || layer.is_adjustment() {
            continue;
        }
        if let Some(b) = layer.object().and_then(|o| o.box_()) {
            if b.intersects_rect(rect.x as f64, rect.y as f64, rect.right() as f64, rect.bottom() as f64) {
                found.push(layer.id());
            }
            continue;
        }
        let bounds = layer.visible_bounds();
        if bounds.is_empty() {
            continue;
        }
        let covers_canvas = bounds.width >= doc.width() as i32 && bounds.height >= doc.height() as i32;
        if layer.object().is_none() && covers_canvas {
            continue;
        }
        if bounds.x <= rect.right()
            && bounds.right() >= rect.x
            && bounds.y <= rect.bottom()
            && bounds.bottom() >= rect.y
        {
            found.push(layer.id());
        }
    }
    found
}
