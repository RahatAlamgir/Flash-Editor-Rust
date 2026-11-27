//! What the slideshow needs of each page: pictures of the page before and after its animations, and (when the page has
//! animations) the page cut into parts: runs of ordinary layers drawn as one picture, and every animated layer on its own.

use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use fe_core::document::animation::{Animation, Timeline, look_of, timeline};
use fe_core::document::{Document, Layer, LayerId, Page};
use fe_core::imaging::PixelRect;
use fe_core::paint::compositor;

/// One piece of an animated page, from the bottom up.
pub enum Part {
    /// Layers without animation, flattened.
    Still(TextureHandle),
    /// An animated layer: its picture, where it is on the page (fractions of the page), and its opacity.
    Layer { id: LayerId, texture: TextureHandle, at: [f32; 4], opacity: f32 },
}

pub struct PageShow {
    /// The page before any animation has run (what a transition arrives at).
    pub initial: TextureHandle,
    /// The page after all animations (what a transition leaves). `None` when it is the same as `initial`.
    pub finished: Option<TextureHandle>,
    pub animations: Vec<Animation>,
    pub timeline: Timeline,
    pub parts: Vec<Part>,
}

fn texture(ctx: &Context, name: &str, image: ColorImage) -> TextureHandle {
    ctx.load_texture(format!("slideshow-{name}"), image, TextureOptions::LINEAR)
}

fn flat(ctx: &Context, name: &str, doc: &Document, keep: &dyn Fn(&Layer) -> bool) -> TextureHandle {
    let mut copy = Document::new(doc.width(), doc.height());
    let layers: Vec<Layer> = doc.layers().iter().filter(|l| keep(l)).cloned().collect();
    copy.set_layers(layers, None);
    let flat = compositor::flatten(&copy);
    texture(ctx, name, ColorImage::from_rgba_unmultiplied([flat.width() as usize, flat.height() as usize], flat.data()))
}

pub fn build(ctx: &Context, number: usize, page: &Page) -> PageShow {
    let animations = page.valid_animations();
    let timeline = timeline(&animations);
    if animations.is_empty() {
        return PageShow {
            initial: flat(ctx, &number.to_string(), &page.doc, &|_| true),
            finished: None,
            animations,
            timeline,
            parts: Vec::new(),
        };
    }
    let doc = &page.doc;
    let hidden_at =
        |layer: &Layer, step: usize, time: f32| !look_of(&animations, &timeline, layer.id(), step, time).visible;
    let initial = flat(ctx, &format!("{number}-start"), doc, &|l| !hidden_at(l, 0, 0.0));
    let finished = flat(ctx, &format!("{number}-end"), doc, &|l| !hidden_at(l, timeline.clicks + 1, 0.0));
    let animated: Vec<LayerId> = animations.iter().map(|a| a.layer).collect();
    let canvas = doc.bounds();
    let (w, h) = (doc.width() as f32, doc.height() as f32);
    let mut parts = Vec::new();
    let mut run: Vec<Layer> = Vec::new();
    let flush = |run: &mut Vec<Layer>, parts: &mut Vec<Part>| {
        if run.is_empty() {
            return;
        }
        let mut copy = Document::new(doc.width(), doc.height());
        copy.set_layers(std::mem::take(run), None);
        let flat = compositor::flatten(&copy);
        let image = ColorImage::from_rgba_unmultiplied([flat.width() as usize, flat.height() as usize], flat.data());
        parts.push(Part::Still(texture(ctx, &format!("{number}-run{}", parts.len()), image)));
    };
    for layer in doc.layers() {
        if !layer.visible || layer.opacity() <= 0.0 {
            continue;
        }
        if !animated.contains(&layer.id()) {
            run.push(layer.clone());
            continue;
        }
        flush(&mut run, &mut parts);
        let bounds: PixelRect = layer.pixels().visible_bounds().intersect(&canvas);
        if bounds.is_empty() {
            continue;
        }
        let rgba = layer.pixels().read_rect(bounds);
        let image = ColorImage::from_rgba_unmultiplied([bounds.width as usize, bounds.height as usize], &rgba);
        parts.push(Part::Layer {
            id: layer.id(),
            texture: texture(ctx, &format!("{number}-layer{}", parts.len()), image),
            at: [bounds.x as f32 / w, bounds.y as f32 / h, bounds.width as f32 / w, bounds.height as f32 / h],
            opacity: layer.opacity() as f32,
        });
    }
    flush(&mut run, &mut parts);
    PageShow { initial, finished: Some(finished), animations, timeline, parts }
}
