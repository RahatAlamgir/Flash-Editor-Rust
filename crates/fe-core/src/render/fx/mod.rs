//! The looks of an object (C# `ObjectRenderer.RenderInto` / `DrawWithShadow`): blur, 3D, inner shadow, outline,
//! shadow and glow, made from the transparency of the drawn object. Everything here works in canvas directions
//! (a shadow keeps pointing the same way however the object is turned).

mod plane;
mod shadow;
mod surface;
mod three_d;

use tiny_skia::Pixmap;

pub use plane::Img;

use crate::objects::Looks;
use plane::{alpha_plane, blur, clamp_byte};

/// Premultiplied pixmap to straight colors.
pub fn from_pixmap(pm: &Pixmap) -> Img {
    let mut img = Img::new(pm.width() as usize, pm.height() as usize);
    for (o, p) in img.data.as_chunks_mut::<4>().0.iter_mut().zip(pm.data().as_chunks::<4>().0.iter()) {
        let a = u32::from(p[3]);
        if a == 0 {
            continue;
        }
        for c in 0..3 {
            o[c] = ((u32::from(p[c]) * 255 + a / 2) / a).min(255) as u8;
        }
        o[3] = p[3];
    }
    img
}

/// Straight colors back to a premultiplied pixmap.
pub fn to_pixmap(img: &Img) -> Option<Pixmap> {
    let mut pm = Pixmap::new(img.w as u32, img.h as u32)?;
    for (o, p) in pm.data_mut().as_chunks_mut::<4>().0.iter_mut().zip(img.data.as_chunks::<4>().0.iter()) {
        let a = u32::from(p[3]);
        for c in 0..3 {
            o[c] = ((u32::from(p[c]) * a + 127) / 255) as u8;
        }
        o[3] = p[3];
    }
    Some(pm)
}

/// Blurs the object itself (all four channels, premultiplied, so edges stay clean).
fn blur_object(img: &Img, sigma: f64) -> Img {
    let (w, h) = (img.w, img.h);
    let alpha = alpha_plane(img);
    let mut planes: Vec<Vec<f32>> = (0..3)
        .map(|c| img.data.as_chunks::<4>().0.iter().zip(&alpha).map(|(p, a)| f32::from(p[c]) / 255.0 * a).collect())
        .collect();
    for plane in &mut planes {
        *plane = blur(plane, w, h, sigma);
    }
    let alpha = blur(&alpha, w, h, sigma);
    let mut out = Img::new(w, h);
    for (i, a) in alpha.iter().enumerate() {
        if *a <= 0.0005 {
            continue;
        }
        for (c, plane) in planes.iter().enumerate() {
            out.data[i * 4 + c] = clamp_byte(plane[i] / a * 255.0);
        }
        out.data[i * 4 + 3] = clamp_byte(a * 255.0);
    }
    out
}

/// The drawn object with its looks. `box_size` is the size of the object's box on the canvas, `center` where its
/// middle is in the picture (a shadow can be bigger or smaller around it).
pub fn apply(img: Img, looks: &Looks, box_size: (f64, f64), center: (f64, f64)) -> Img {
    let mut pixels = img;
    // Blur of the object itself comes first, so the other looks are made from the blurred object.
    if looks.blur >= 0.3 {
        pixels = blur_object(&pixels, looks.blur);
    }
    // The 3D look comes next (the shadow is then cast by the object together with its side).
    if let Some(t) = looks.three_d {
        let (ox, oy) = t.offset();
        let bevel = (box_size.0.min(box_size.1) * 0.05).clamp(3.0, 14.0);
        pixels = three_d::apply(&pixels, ox, oy, t.color, t.bevel, bevel);
    }
    // Then the looks that follow the outline: a shadow inside the edges, and a line around the outside.
    if let Some(i) = looks.inner_shadow {
        let (ox, oy) = i.offset();
        pixels = surface::inner_shadow(&pixels, i.color, ox, oy, i.blur);
    }
    if let Some(o) = looks.outline {
        pixels = surface::outline(&pixels, o.color, o.width);
    }
    // What the glow is made from: the object without its shadow.
    let shape = looks.glow.is_some().then(|| pixels.clone());
    if let Some(s) = looks.shadow {
        let (ox, oy) = s.offset();
        pixels = shadow::apply(&pixels, s.color, ox, oy, s.blur, s.size, center.0, center.1);
    }
    if let (Some(g), Some(shape)) = (looks.glow, shape) {
        pixels = surface::under(&pixels, &surface::glow_layer(&shape, g.color, g.size, g.intensity));
    }
    pixels
}
