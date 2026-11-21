//! Keeps the tile textures of the shown document up to date and draws them.
//!
//! Small updates (a few full-size tiles, like a brush dab) are combined right away. Bigger ones (opening a
//! picture, zooming into another level, a layer setting that changes every tile) are combined on background
//! threads; until a tile arrives, its old picture — or a picture from a nearby level — stays on screen, so the
//! window never freezes.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use egui::{Color32, ColorImage, Painter, Pos2, Rect, TextureHandle, TextureOptions, pos2, vec2};
use fe_core::document::Document;
use fe_core::imaging::{PixelRect, TileKey};
use fe_core::paint::signature::region_signature;
use rayon::prelude::*;

use crate::levels::{level_for_zoom, level_span};
use crate::tile_image::{render_tile, tile_area};

/// Frames a texture may stay unused before it is dropped (about 5 s at 60 fps; no frames are drawn while idle).
const KEEP_FRAMES: u64 = 300;
/// Up to this many changed full-size tiles are combined at once, in the frame itself.
const IMMEDIATE_TILES: usize = 4;

/// Pixels look sharp (not blurred) when zoomed in, smooth when zoomed out.
const TILE_OPTIONS: TextureOptions = TextureOptions {
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Linear,
    wrap_mode: egui::TextureWrapMode::ClampToEdge,
    mipmap_mode: None,
};

/// Where the canvas is on screen: canvas pixel (x, y) is drawn at `origin + (x, y) * zoom`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewTransform {
    pub zoom: f32,
    pub origin: Pos2,
}

impl ViewTransform {
    pub fn to_screen(&self, x: f32, y: f32) -> Pos2 {
        self.origin + vec2(x, y) * self.zoom
    }

    pub fn to_canvas(&self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.zoom).to_pos2()
    }

    pub fn rect_to_screen(&self, r: PixelRect) -> Rect {
        Rect::from_min_max(self.to_screen(r.x as f32, r.y as f32), self.to_screen(r.right() as f32, r.bottom() as f32))
    }
}

/// What the last frame did (for the debug overlay and benchmarks).
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub level: u32,
    /// Tiles drawn this frame.
    pub drawn: usize,
    /// Tiles combined and uploaded this frame.
    pub rendered: usize,
    /// Tiles being combined on background threads.
    pub pending: usize,
    /// Time this frame spent combining and uploading, in milliseconds.
    pub render_ms: f32,
    /// Texture memory held by the cache.
    pub texture_bytes: usize,
}

struct Entry {
    texture: TextureHandle,
    signature: u64,
    area: PixelRect,
    last_frame: u64,
}

/// A finished background job: document, level, tile, fingerprint and the picture.
type Finished = (u64, u32, TileKey, u64, Option<(ColorImage, PixelRect)>);

/// The texture cache of one canvas view.
pub struct CanvasRenderer {
    entries: HashMap<(u32, TileKey), Entry>,
    /// Tiles being combined in the background.
    in_flight: HashMap<(u32, TileKey), u64>,
    /// The document the cache belongs to; another document clears it.
    doc_uid: Option<u64>,
    frame: u64,
    sender: Sender<Finished>,
    receiver: Receiver<Finished>,
}

impl Default for CanvasRenderer {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { entries: HashMap::new(), in_flight: HashMap::new(), doc_uid: None, frame: 0, sender, receiver }
    }
}

impl CanvasRenderer {
    /// Draws `doc` with `view` into `clip` (screen rect of the canvas area), updating changed tiles.
    pub fn paint(
        &mut self,
        ctx: &egui::Context,
        painter: &Painter,
        doc: &Document,
        view: ViewTransform,
        clip: Rect,
    ) -> RenderStats {
        self.frame += 1;
        if self.doc_uid != Some(doc.uid()) {
            self.clear();
            self.doc_uid = Some(doc.uid());
        }
        let start = Instant::now();
        let mut rendered = self.collect_finished(ctx);

        let level = level_for_zoom(view.zoom);
        let wanted: Vec<(TileKey, u64)> = visible_keys(doc, view, clip, level)
            .into_iter()
            .map(|k| (k, region_signature(doc, tile_area(level, k).intersect(&doc.bounds()))))
            .collect();
        let stale: Vec<(TileKey, u64)> = wanted
            .iter()
            .copied()
            .filter(|(k, sig)| self.entries.get(&(level, *k)).is_none_or(|e| e.signature != *sig))
            .filter(|(k, _)| !self.in_flight.contains_key(&(level, *k)))
            .collect();

        if level == 0 && stale.len() <= IMMEDIATE_TILES {
            let fresh: Vec<_> = stale.par_iter().map(|&(k, sig)| (k, sig, render_tile(doc, 0, k))).collect();
            rendered += fresh.len();
            for (key, sig, picture) in fresh {
                self.store(ctx, 0, key, sig, picture);
            }
        } else if !stale.is_empty() {
            self.start_jobs(ctx, doc, level, stale);
        }
        let render_ms = start.elapsed().as_secs_f32() * 1000.0;

        let drawn = self.draw(painter, view, level, &wanted);
        let frame = self.frame;
        self.entries.retain(|_, e| e.last_frame + KEEP_FRAMES > frame);
        let texture_bytes = self.entries.values().map(|e| e.texture.size()[0] * e.texture.size()[1] * 4).sum();
        RenderStats { level, drawn, rendered, pending: self.in_flight.len(), render_ms, texture_bytes }
    }

    /// Drops every texture and forgets running jobs (their results will be ignored).
    pub fn clear(&mut self) {
        self.entries.clear();
        self.in_flight.clear();
        self.doc_uid = None;
    }

    /// Uploads the tiles finished by background threads; returns how many.
    fn collect_finished(&mut self, ctx: &egui::Context) -> usize {
        let mut count = 0;
        while let Ok((uid, level, key, sig, picture)) = self.receiver.try_recv() {
            if Some(uid) != self.doc_uid {
                continue; // from a document that is no longer shown
            }
            self.in_flight.remove(&(level, key));
            self.store(ctx, level, key, sig, picture);
            count += 1;
        }
        count
    }

    /// Combines tiles on background threads. They work on a snapshot of the document (cheap: tiles are
    /// shared, not copied), and ask for a new frame when done.
    fn start_jobs(&mut self, ctx: &egui::Context, doc: &Document, level: u32, stale: Vec<(TileKey, u64)>) {
        let snapshot = Arc::new(doc.clone());
        for (key, sig) in stale {
            self.in_flight.insert((level, key), sig);
            let (doc, sender, ctx) = (snapshot.clone(), self.sender.clone(), ctx.clone());
            rayon::spawn(move || {
                let picture = render_tile(&doc, level, key);
                let _ = sender.send((doc.uid(), level, key, sig, picture));
                ctx.request_repaint();
            });
        }
    }

    fn store(
        &mut self,
        ctx: &egui::Context,
        level: u32,
        key: TileKey,
        sig: u64,
        picture: Option<(ColorImage, PixelRect)>,
    ) {
        let Some((image, area)) = picture else {
            self.entries.remove(&(level, key));
            return;
        };
        match self.entries.get_mut(&(level, key)) {
            Some(entry) => {
                entry.texture.set(image, TILE_OPTIONS);
                entry.signature = sig;
                entry.area = area;
            }
            None => {
                let name = format!("fe-tile-{level}-{}-{}", key.tx, key.ty);
                let texture = ctx.load_texture(name, image, TILE_OPTIONS);
                self.entries.insert((level, key), Entry { texture, signature: sig, area, last_frame: self.frame });
            }
        }
    }

    /// Draws the wanted tiles. A tile that is not ready yet is covered by pictures from other levels.
    fn draw(&mut self, painter: &Painter, view: ViewTransform, level: u32, wanted: &[(TileKey, u64)]) -> usize {
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let frame = self.frame;
        let missing: Vec<PixelRect> = wanted
            .iter()
            .filter(|(k, _)| !self.entries.contains_key(&(level, *k)))
            .map(|(k, _)| tile_area(level, *k))
            .collect();
        if !missing.is_empty() {
            // Coarser levels first, so finer stand-ins end up on top.
            let mut stand_ins: Vec<(&(u32, TileKey), &mut Entry)> = self
                .entries
                .iter_mut()
                .filter(|((l, _), e)| *l != level && missing.iter().any(|m| !m.intersect(&e.area).is_empty()))
                .collect();
            stand_ins.sort_by_key(|((l, _), _)| std::cmp::Reverse(*l));
            for (_, entry) in stand_ins {
                entry.last_frame = frame;
                painter.image(entry.texture.id(), view.rect_to_screen(entry.area), uv, Color32::WHITE);
            }
        }
        let mut drawn = 0;
        for (key, _) in wanted {
            if let Some(entry) = self.entries.get_mut(&(level, *key)) {
                entry.last_frame = frame;
                painter.image(entry.texture.id(), view.rect_to_screen(entry.area), uv, Color32::WHITE);
                drawn += 1;
            }
        }
        drawn
    }
}

/// The tiles of `level` that are (partly) on screen.
fn visible_keys(doc: &Document, view: ViewTransform, clip: Rect, level: u32) -> Vec<TileKey> {
    let a = view.to_canvas(clip.min);
    let b = view.to_canvas(clip.max);
    let visible = PixelRect::from_bounds(a.x as f64, a.y as f64, b.x as f64, b.y as f64).intersect(&doc.bounds());
    if visible.is_empty() {
        return Vec::new();
    }
    let span = level_span(level);
    let (tx0, ty0) = (visible.x.div_euclid(span), visible.y.div_euclid(span));
    let (tx1, ty1) = ((visible.right() - 1).div_euclid(span), (visible.bottom() - 1).div_euclid(span));
    (ty0..=ty1).flat_map(|ty| (tx0..=tx1).map(move |tx| TileKey::new(tx, ty))).collect()
}
