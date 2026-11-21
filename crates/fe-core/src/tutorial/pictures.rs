//! The pictures of the tutorial: the photos that ship with the program, and a few quick looks made from them.

use std::sync::{Arc, OnceLock};

use crate::imaging::{ColorRgba, PixelBuffer, PixelRect, color_math};
use crate::io::decode::load_bytes;
use crate::objects::{ObjectBox, StoredObject, picture_bytes};

macro_rules! asset {
    ($name:literal) => {
        include_bytes!(concat!("../../../../assets/tutorial/", $name)) as &[u8]
    };
}

fn decode(bytes: &[u8], ext: &str) -> Option<PixelBuffer> {
    load_bytes(bytes, Some(ext)).ok()
}

/// The photo of the tutorial (a man in a red T-shirt); a drawn landscape when it cannot be read.
pub fn photo() -> &'static PixelBuffer {
    static PHOTO: OnceLock<PixelBuffer> = OnceLock::new();
    PHOTO.get_or_init(|| decode(asset!("TutorialPhoto.jpg"), "jpg").unwrap_or_else(|| landscape(800, 800, 0)))
}

pub fn asset_picture(name: &str) -> Option<PixelBuffer> {
    match name {
        "tee" => decode(asset!("TutorialTee.jpg"), "jpg"),
        "recolor" => decode(asset!("TutorialTeeRecolor.jpg"), "jpg"),
        "cutout" => decode(asset!("TutorialTeeCutout.png"), "png"),
        "fill-before" => decode(asset!("TutorialFillBefore.jpg"), "jpg"),
        "fill-after" => decode(asset!("TutorialFillAfter.jpg"), "jpg"),
        _ => None,
    }
}

/// A small landscape: a sky, a sun and two hills.
pub fn landscape(width: u32, height: u32, variant: usize) -> PixelBuffer {
    let skies =
        [((251, 146, 60), (251, 207, 232)), ((14, 116, 144), (186, 230, 253)), ((30, 64, 175), (196, 181, 253))];
    let hills = [((124, 45, 18), (180, 83, 9)), ((6, 78, 59), (20, 184, 166)), ((49, 46, 129), (99, 102, 241))];
    let ((tr, tg, tb), (br, bg, bb)) = skies[variant % 3];
    let (far, near) = hills[variant % 3];
    let v = variant as f64;
    let mut b = PixelBuffer::new(width, height);
    let (w, h) = (f64::from(width), f64::from(height));
    let (sun_x, sun_y, sun_r) = (w * (0.3 + 0.2 * v), h * 0.38, h * 0.14);
    for y in 0..height {
        let t = f64::from(y) / h;
        for x in 0..width {
            let lerp = |a: i32, b2: i32| (f64::from(a) + f64::from(b2 - a) * t) as u8;
            let mut col = ColorRgba::rgb(lerp(tr, br), lerp(tg, bg), lerp(tb, bb));
            let (dx, dy) = (f64::from(x) - sun_x, f64::from(y) - sun_y);
            if dx * dx + dy * dy < sun_r * sun_r {
                col = ColorRgba::rgb(254, 249, 195);
            }
            let far_line = h * (0.62 + 0.07 * (f64::from(x) / w * 7.0 + v).sin());
            let near_line = h * (0.78 + 0.06 * (f64::from(x) / w * 5.0 + 2.0 + v).sin());
            if f64::from(y) > far_line {
                col = ColorRgba::rgb(far.0, far.1, far.2);
            }
            if f64::from(y) > near_line {
                col = ColorRgba::rgb(near.0, near.1, near.2);
            }
            b.set_pixel(x, y, col);
        }
    }
    b
}

/// The photo as a square from the middle, `size` pixels wide.
pub fn photo_square(size: u32) -> PixelBuffer {
    let p = photo();
    let side = p.width().min(p.height());
    let square = p.crop(PixelRect::new(
        ((p.width() - side) / 2) as i32,
        ((p.height() - side) / 2) as i32,
        side as i32,
        side as i32,
    ));
    resize(&square, size, size, true)
}

/// The picture resized (smooth, or hard pixel edges).
pub fn resize(src: &PixelBuffer, w: u32, h: u32, smooth: bool) -> PixelBuffer {
    crate::transform::resize(
        src,
        w,
        h,
        if smooth { crate::transform::ResampleMode::Smooth } else { crate::transform::ResampleMode::NearestNeighbor },
    )
}

/// Effects on the tutorial pictures.
pub mod fx {
    pub use crate::effects::*;
    use crate::imaging::PixelBuffer;

    /// An effect with some of its settings changed.
    pub fn run(effect: &Effect, src: &PixelBuffer, set: &[(&'static str, f64)]) -> PixelBuffer {
        let mut v = effect.defaults();
        for (k, x) in set {
            v.set(k, *x);
        }
        effect.apply(src, &v)
    }
}

/// Applies a color change to every pixel (the alpha stays).
pub fn map(src: &PixelBuffer, f: impl Fn(f64, f64, f64, f64, f64) -> (f64, f64, f64)) -> PixelBuffer {
    let mut out = src.clone();
    let (w, h) = (f64::from(src.width()), f64::from(src.height()));
    for y in 0..src.height() {
        for x in 0..src.width() {
            let p = src.pixel(x, y);
            let (r, g, b) = f(f64::from(p.r), f64::from(p.g), f64::from(p.b), f64::from(x) / w, f64::from(y) / h);
            out.set_pixel(
                x,
                y,
                ColorRgba::new(r.clamp(0.0, 255.0) as u8, g.clamp(0.0, 255.0) as u8, b.clamp(0.0, 255.0) as u8, p.a),
            );
        }
    }
    out
}

/// A hue turn of everything strongly red (the T-shirt); the rest stays.
pub fn recolor_red(src: &PixelBuffer, hue_shift: f64) -> PixelBuffer {
    map(src, |r, g, b, _, _| {
        let weight = ((r / (g.max(b) + 8.0) - 1.6) / 0.9).clamp(0.0, 1.0);
        if r < 60.0 || weight <= 0.0 {
            return (r, g, b);
        }
        let (h, s, v) = color_math::rgb_to_hsv(r / 255.0, g / 255.0, b / 255.0);
        let (r2, g2, b2) = color_math::hsv_to_rgb(h + hue_shift, s, v);
        (r + (r2 * 255.0 - r) * weight, g + (g2 * 255.0 - g) * weight, b + (b2 * 255.0 - b) * weight)
    })
}

/// A placed picture object.
pub fn pic(image: &PixelBuffer, x: f64, y: f64, w: f64, h: f64, name: &str) -> StoredObject {
    StoredObject::new_image(ObjectBox::new(x, y, w, h, 0.0), picture_bytes(image), name)
}

/// The photo as PNG bytes, made once (every frame that shows it shares the one picture in the file).
pub fn photo_png() -> Arc<Vec<u8>> {
    static PNG: OnceLock<Arc<Vec<u8>>> = OnceLock::new();
    PNG.get_or_init(|| picture_bytes(photo())).clone()
}
