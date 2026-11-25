//! Magic Resize (C# `MagicResize`): turns a design into another canvas size (a post into a story, a poster into a banner)
//! and keeps the layout. Everything keeps its proportions and moves to the same relative place; a background that covered
//! the whole canvas covers the new one.

use crate::imaging::{PixelBuffer, PixelRect, PointD};
use crate::objects::{ObjectBox, StoredObject};
use crate::transform::{ResampleMode, resize};

/// An object is a "background" when it covers (nearly) the whole old canvas.
pub fn is_background(object: &StoredObject, old_w: u32, old_h: u32) -> bool {
    object.box_().is_some_and(|b| {
        b.rotation.abs() < 0.5 && b.width >= f64::from(old_w) * 0.9 && b.height >= f64::from(old_h) * 0.9
    })
}

/// The object on the new canvas.
pub fn map_object(object: &StoredObject, old_w: u32, old_h: u32, new_w: u32, new_h: u32) -> StoredObject {
    let (ow, oh, nw, nh) = (f64::from(old_w), f64::from(old_h), f64::from(new_w), f64::from(new_h));
    let (sx, sy) = (nw / ow, nh / oh);
    let s = sx.min(sy);
    let Some(b) = object.box_() else { return object.clone() };
    if is_background(object, old_w, old_h) {
        // A flat shape stretches to the new canvas. Pictures and everything else cover it without being distorted.
        if object.kind() == "shape" {
            let mut stretched = object.clone();
            stretched.set_box(ObjectBox::new(0.0, 0.0, nw, nh, 0.0));
            return stretched;
        }
        let cover = (nw / b.width).max(nh / b.height);
        let (w, h) = (b.width * cover, b.height * cover);
        let mut centered = object.clone();
        centered.set_box(b.with_center(PointD::new(ow / 2.0, oh / 2.0)));
        let mut scaled = centered.scaled(cover, cover);
        scaled.set_box(ObjectBox::from_center(PointD::new(nw / 2.0, nh / 2.0), w, h, 0.0));
        return scaled;
    }
    // Size by the smaller change, place by the relative position of the middle.
    let center = b.center();
    object.scaled(s, s).offset(center.x * (sx - s), center.y * (sy - s))
}

/// A layer of plain pixels on the new canvas (what is painted keeps its proportions; a layer that is solid everywhere
/// covers the canvas).
pub fn map_pixels(pixels: &PixelBuffer, new_w: u32, new_h: u32) -> PixelBuffer {
    let (ow, oh) = (f64::from(pixels.width()), f64::from(pixels.height()));
    let (nw, nh) = (f64::from(new_w), f64::from(new_h));
    let (sx, sy) = (nw / ow, nh / oh);
    let s = sx.min(sy);
    let mut result = PixelBuffer::new(new_w, new_h);
    let bounds = pixels.visible_bounds();
    if bounds.is_empty() {
        return result;
    }
    let solid = pixels.data().iter().skip(3).step_by(4).filter(|a| **a > 250).count();
    if solid as f64 >= (pixels.data().len() / 4) as f64 * 0.95 {
        let cover = sx.max(sy);
        let w = new_w.max((ow * cover).round_ties_even() as u32);
        let h = new_h.max((oh * cover).round_ties_even() as u32);
        let big = resize(pixels, w, h, ResampleMode::Smooth);
        result.copy_from(
            &big,
            PixelRect::new(((w - new_w) / 2) as i32, ((h - new_h) / 2) as i32, new_w as i32, new_h as i32),
            0,
            0,
        );
        return result;
    }
    let content = pixels.crop(bounds);
    let cw = ((f64::from(content.width()) * s).round_ties_even() as u32).max(1);
    let ch = ((f64::from(content.height()) * s).round_ties_even() as u32).max(1);
    let scaled = resize(&content, cw, ch, ResampleMode::Smooth);
    let cx = (f64::from(bounds.x) + f64::from(bounds.width) / 2.0) * sx;
    let cy = (f64::from(bounds.y) + f64::from(bounds.height) / 2.0) * sy;
    result.copy_all_from(
        &scaled,
        (cx - f64::from(cw) / 2.0).round_ties_even() as i32,
        (cy - f64::from(ch) / 2.0).round_ties_even() as i32,
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;
    use crate::objects::{ShapeData, ShapeKind};

    fn shape_at(x: f64, y: f64, w: f64, h: f64) -> StoredObject {
        StoredObject::new_shape(
            ObjectBox::new(x, y, w, h, 0.0),
            &ShapeData::new(ShapeKind::Rectangle, Some(ColorRgba::rgb(10, 10, 10)), None, 0.0),
        )
    }

    #[test]
    fn a_background_shape_covers_the_new_canvas_and_a_small_one_keeps_its_shape() {
        let back = shape_at(0.0, 0.0, 1000.0, 1000.0);
        let mapped = map_object(&back, 1000, 1000, 1080, 1920);
        let b = mapped.box_().expect("box");
        assert_eq!((b.x, b.y, b.width, b.height), (0.0, 0.0, 1080.0, 1920.0));
        // A square in the middle stays a square, scaled by the smaller change, in the middle of the new canvas.
        let small = shape_at(400.0, 400.0, 200.0, 200.0);
        let b = map_object(&small, 1000, 1000, 1080, 1920).box_().expect("box");
        assert!((b.width - 216.0).abs() < 1e-6 && (b.height - 216.0).abs() < 1e-6);
        assert!((b.center().x - 540.0).abs() < 1e-6 && (b.center().y - 960.0).abs() < 1e-6);
    }

    #[test]
    fn painted_pixels_keep_their_shape_and_a_solid_layer_covers_the_canvas() {
        let mut dot = PixelBuffer::new(100, 100);
        for y in 40..60 {
            for x in 40..60 {
                dot.set_pixel(x, y, ColorRgba::rgb(255, 0, 0));
            }
        }
        let out = map_pixels(&dot, 200, 400);
        assert_eq!((out.width(), out.height()), (200, 400));
        assert_eq!(out.pixel(100, 200).r, 255, "the dot stays in the middle");
        assert_eq!(out.pixel(5, 5).a, 0);
        let solid = map_pixels(&PixelBuffer::filled(100, 100, ColorRgba::rgb(0, 0, 255)), 200, 400);
        assert_eq!(solid.pixel(3, 3).b, 255);
        assert_eq!(solid.pixel(199, 399).b, 255);
    }
}
