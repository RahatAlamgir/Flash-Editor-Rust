//! A small picture of each brush type (a wavy stroke made with the real brush engine), shown in the brush type
//! list like the C# `BrushTypePreviews`.

use egui::{ColorImage, TextureHandle, TextureOptions};
use fe_core::imaging::{ColorRgba, PixelRect, PointD, TileMap};
use fe_core::paint::brush::{BrushSettings, BrushType};
use fe_core::paint::stroke::{Effect, PaintStroke};

const W: i32 = 96;
const H: i32 = 24;

/// The preview textures, made once when first needed.
#[derive(Default)]
pub struct BrushPreviews {
    textures: Vec<(BrushType, TextureHandle)>,
}

fn render(kind: BrushType) -> ColorImage {
    let canvas = PixelRect::new(0, 0, W, H);
    let mut pixels = TileMap::empty();
    let brush = BrushSettings { size: 10.0, hardness: 0.8, opacity: 1.0, kind };
    let mut stroke = PaintStroke::new(&pixels, brush, Effect::Paint(ColorRgba::rgb(0xE6, 0xE6, 0xE6)), None, canvas);
    for i in 0..=40 {
        let x = 8.0 + i as f64 * (W as f64 - 16.0) / 40.0;
        let y = H as f64 / 2.0 + (i as f64 / 40.0 * std::f64::consts::TAU).sin() * 5.0;
        stroke.add_point(&mut pixels, PointD::new(x, y));
    }
    let picture = pixels.to_buffer(canvas);
    ColorImage::from_rgba_unmultiplied([W as usize, H as usize], picture.data())
}

impl BrushPreviews {
    /// The preview of a brush type.
    pub fn texture(&mut self, ctx: &egui::Context, kind: BrushType) -> egui::TextureId {
        if self.textures.is_empty() {
            self.textures = BrushType::ALL
                .iter()
                .map(|k| (*k, ctx.load_texture(format!("brush-{}", k.name()), render(*k), TextureOptions::LINEAR)))
                .collect();
        }
        self.textures.iter().find(|(k, _)| *k == kind).map(|(_, t)| t.id()).unwrap_or_default()
    }

    pub fn size() -> egui::Vec2 {
        egui::vec2(W as f32, H as f32)
    }
}
