//! Small pictures of ready-made badges, graphics and diagrams for the pickers: drawn once with the real renderer and
//! kept as textures (the C# `IconPickerDialog.MakeBadgePreviews`).

use std::collections::HashMap;

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use fe_core::imaging::PixelRect;
use fe_core::library::{badges, graphics, smartart};
use fe_core::objects::{ChartKind, ObjectBox, StoredObject};

/// The textures made so far.
#[derive(Default)]
pub struct Previews {
    textures: HashMap<String, TextureHandle>,
}

/// The size of a preview, in pixels.
pub const SIZE: u32 = 150;

fn picture(ctx: &Context, key: &str, object: Option<StoredObject>) -> Option<TextureHandle> {
    let object = object?;
    let rect = PixelRect::new(0, 0, SIZE as i32, SIZE as i32);
    let tiles = fe_core::render::render_object(&object, rect)?;
    let buffer = tiles.to_buffer(rect);
    let image = ColorImage::from_rgba_unmultiplied([buffer.width() as usize, buffer.height() as usize], buffer.data());
    Some(ctx.load_texture(format!("preview-{key}"), image, TextureOptions::LINEAR))
}

impl Previews {
    fn get(
        &mut self,
        ctx: &Context,
        key: String,
        make: impl FnOnce() -> Option<StoredObject>,
    ) -> Option<&TextureHandle> {
        if !self.textures.contains_key(&key) {
            let texture = picture(ctx, &key, make())?;
            self.textures.insert(key.clone(), texture);
        }
        self.textures.get(&key)
    }

    pub fn badge(&mut self, ctx: &Context, id: &str) -> Option<&TextureHandle> {
        let size = f64::from(SIZE);
        self.get(ctx, format!("badge-{id}"), || badges::build(id, size, size, size * 0.86))
    }

    pub fn graphic(&mut self, ctx: &Context, id: &str) -> Option<&TextureHandle> {
        let size = f64::from(SIZE);
        self.get(ctx, format!("graphic-{id}"), || graphics::build(id, size, size, size * 0.8))
    }

    /// A chart with its sample numbers.
    pub fn chart(&mut self, ctx: &Context, kind: ChartKind) -> Option<&TextureHandle> {
        let size = f64::from(SIZE);
        self.get(ctx, format!("chart-{}", kind.name()), || {
            let chart = crate::commands::insert::sample_chart(kind, &[]);
            let b = ObjectBox::new(size * 0.04, size * 0.1, size * 0.92, size * 0.8, 0.0);
            Some(StoredObject::new_chart(b, &chart))
        })
    }

    /// A diagram with its sample words in a color scheme.
    pub fn smartart(&mut self, ctx: &Context, id: &str, scheme: &str) -> Option<&TextureHandle> {
        let size = f64::from(SIZE);
        self.get(ctx, format!("smartart-{id}-{scheme}"), || {
            let words: Vec<String> = smartart::layout(id)?.sample.iter().map(|w| (*w).to_owned()).collect();
            smartart::build(id, &words, scheme, size, size, size * 0.88)
        })
    }
}
