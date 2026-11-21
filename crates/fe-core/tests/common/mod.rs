//! Helpers that build small images for tests (port of the C# `TestImages`).
#![allow(dead_code)]

use fe_core::document::{Document, Layer};
use fe_core::imaging::{ColorRgba, PixelBuffer};

pub fn solid(w: u32, h: u32, color: ColorRgba) -> PixelBuffer {
    PixelBuffer::filled(w, h, color)
}

/// Each pixel gets a unique color derived from its position (easy to track after transforms).
pub fn numbered(w: u32, h: u32) -> PixelBuffer {
    let mut buffer = PixelBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            buffer.set_pixel(x, y, ColorRgba::rgb(x as u8, y as u8, (x + y * w) as u8));
        }
    }
    buffer
}

/// A document with these layers (bottom to top); the top one is active.
pub fn doc_with(w: u32, h: u32, layers: Vec<Layer>) -> Document {
    let mut doc = Document::new(w, h);
    doc.set_layers(layers, None);
    doc
}

/// A layer filled with one color.
pub fn solid_layer(name: &str, w: u32, h: u32, color: ColorRgba) -> Layer {
    Layer::from_buffer(name, &solid(w, h, color))
}

pub fn in_range(v: u8, lo: u8, hi: u8) -> bool {
    (lo..=hi).contains(&v)
}
