//! Layer and page thumbnails as small textures. They are made on background threads and only redrawn when
//! the content really changed (fingerprints) and the mouse button is up — so dragging stays smooth, like the
//! C# version ("redrawn a moment after you let go of the mouse").

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender, channel};

use egui::{ColorImage, TextureHandle, TextureId, TextureOptions};
use fe_core::document::{Document, Layer, LayerId};
use fe_core::imaging::PixelBuffer;
use fe_core::paint::{signature, thumbnails};

/// Frames a thumbnail may stay unused before it is dropped.
const KEEP_FRAMES: u64 = 600;

/// What a thumbnail shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ThumbKey {
    Layer(u64, LayerId),
    Mask(u64, LayerId),
    Page(u64),
}

struct Entry {
    signature: u64,
    texture: TextureHandle,
    last_frame: u64,
}

/// The thumbnail cache.
pub struct Thumbnails {
    entries: HashMap<ThumbKey, Entry>,
    in_flight: HashSet<ThumbKey>,
    sender: Sender<(ThumbKey, u64, PixelBuffer)>,
    receiver: Receiver<(ThumbKey, u64, PixelBuffer)>,
    frame: u64,
}

impl Default for Thumbnails {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { entries: HashMap::new(), in_flight: HashSet::new(), sender, receiver, frame: 0 }
    }
}

impl Thumbnails {
    /// Call once per frame before using thumbnails: uploads finished ones and forgets unused ones.
    pub fn begin_frame(&mut self, ctx: &egui::Context) {
        self.frame += 1;
        while let Ok((key, sig, picture)) = self.receiver.try_recv() {
            self.in_flight.remove(&key);
            let size = [picture.width() as usize, picture.height() as usize];
            let image = ColorImage::from_rgba_unmultiplied(size, picture.data());
            match self.entries.get_mut(&key) {
                Some(entry) => {
                    entry.texture.set(image, TextureOptions::LINEAR);
                    entry.signature = sig;
                }
                None => {
                    let texture = ctx.load_texture(format!("fe-thumb-{key:?}"), image, TextureOptions::LINEAR);
                    self.entries.insert(key, Entry { signature: sig, texture, last_frame: self.frame });
                }
            }
        }
        let frame = self.frame;
        self.entries.retain(|_, e| e.last_frame + KEEP_FRAMES > frame);
    }

    /// The thumbnail of a layer (`None` until the first one is ready). `settled` = no mouse button held.
    pub fn layer(
        &mut self,
        ctx: &egui::Context,
        doc: &Document,
        layer: &Layer,
        max: [u32; 2],
        settled: bool,
    ) -> Option<(TextureId, [f32; 2])> {
        let key = ThumbKey::Layer(doc.uid(), layer.id());
        let sig = signature::layer_signature(layer, doc.bounds());
        if self.needs_update(key, sig, settled) {
            let canvas = doc.bounds();
            let layer = layer.clone(); // cheap: tiles are shared
            self.spawn(ctx, key, sig, move || thumbnails::layer_thumbnail(&layer, canvas, max[0], max[1]));
        }
        self.texture(key)
    }

    /// The thumbnail of a layer's mask (white = shown, black = hidden), if it has one.
    pub fn mask(
        &mut self,
        ctx: &egui::Context,
        doc: &Document,
        layer: &Layer,
        max: [u32; 2],
        settled: bool,
    ) -> Option<(TextureId, [f32; 2])> {
        let mask = layer.mask()?;
        let key = ThumbKey::Mask(doc.uid(), layer.id());
        let sig = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            mask.fill_value().hash(&mut h);
            let mut keys: Vec<_> = mask.tiles().keys().collect();
            keys.sort();
            for k in keys {
                (k, mask.tiles().tile_stamp(k)).hash(&mut h);
            }
            h.finish()
        };
        if self.needs_update(key, sig, settled) {
            let mask = mask.clone(); // cheap: tiles are shared
            self.spawn(ctx, key, sig, move || {
                fe_core::imaging::create_thumbnail(&mask.to_gray_picture(), max[0], max[1])
            });
        }
        self.texture(key)
    }

    /// The thumbnail of a whole page.
    pub fn page(
        &mut self,
        ctx: &egui::Context,
        doc: &Document,
        max: [u32; 2],
        settled: bool,
    ) -> Option<(TextureId, [f32; 2])> {
        let key = ThumbKey::Page(doc.uid());
        let sig = signature::page_signature(doc);
        if self.needs_update(key, sig, settled) {
            let doc = doc.clone(); // cheap: tiles are shared
            self.spawn(ctx, key, sig, move || thumbnails::page_thumbnail(&doc, max[0], max[1]));
        }
        self.texture(key)
    }

    fn needs_update(&self, key: ThumbKey, sig: u64, settled: bool) -> bool {
        settled && !self.in_flight.contains(&key) && self.entries.get(&key).is_none_or(|e| e.signature != sig)
    }

    fn spawn(
        &mut self,
        ctx: &egui::Context,
        key: ThumbKey,
        sig: u64,
        make: impl FnOnce() -> PixelBuffer + Send + 'static,
    ) {
        self.in_flight.insert(key);
        let (sender, ctx) = (self.sender.clone(), ctx.clone());
        rayon::spawn(move || {
            let _ = sender.send((key, sig, make()));
            ctx.request_repaint();
        });
    }

    fn texture(&mut self, key: ThumbKey) -> Option<(TextureId, [f32; 2])> {
        let frame = self.frame;
        self.entries.get_mut(&key).map(|e| {
            e.last_frame = frame;
            let size = e.texture.size_vec2();
            (e.texture.id(), [size.x, size.y])
        })
    }
}

/// Size of a picture fitted into a box, keeping its shape (for drawing a thumbnail centered in its frame).
pub fn fitted(size: [f32; 2], frame: egui::Rect) -> egui::Rect {
    let scale = (frame.width() / size[0]).min(frame.height() / size[1]);
    egui::Rect::from_center_size(frame.center(), egui::vec2(size[0], size[1]) * scale)
}
