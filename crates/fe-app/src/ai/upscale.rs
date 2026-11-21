//! AI Upscale / Clean Up (C# `UpscaleCommands`): makes a picture sharper and cleaner at the same size, or 2x / 3x / 4x
//! larger with real-looking detail. Size 1 changes only the selected layer; a larger size enlarges the whole picture
//! and improves every painted layer and placed picture (shapes and text are drawn from vectors: already sharp).

use fe_core::document::canvas_ops::{self, transform_all};
use fe_core::document::{LayerId, object_ops};
use fe_core::imaging::{PixelBuffer, TileMap};
use fe_core::objects::pictures;
use fe_core::transform::{ResampleMode, resize};

use super::job::{self, Progress};
use super::outcome::{self, AiDone};
use super::pending::{self, Pending};
use crate::app::FlashApp;
use crate::commands::checks;
use crate::dialogs::Dialog;

const TITLE: &str = "AI Upscale / Clean Up";

/// The settings of the window.
pub struct UpscaleForm {
    /// 1 (same size) to 4.
    pub scale: f64,
    /// 0..100: how much of the AI result is used.
    pub strength: f64,
    pub tiles: usize,
}

impl UpscaleForm {
    /// "about 3 s with a graphics card, up to 1 min without one" (roughly 0.1 s per tile on a graphics card, 2 s on the
    /// processor).
    pub fn time_text(&self) -> String {
        let fast = (self.tiles as f64 * 0.12).ceil().max(1.0);
        let slow = (self.tiles as f64 * 2.0 / 60.0).ceil().max(1.0);
        format!("about {fast} s with a graphics card, up to {slow} min without one")
    }
}

/// One layer the AI improved.
pub struct Improved {
    pub layer: LayerId,
    pub pixels: PixelBuffer,
    /// A placed picture (its own pixels were improved) rather than a painted layer.
    pub image: bool,
}

pub struct Upscaled {
    pub scale: f64,
    pub device: String,
    pub items: Vec<Improved>,
}

/// What to improve on one layer.
struct Piece {
    layer: LayerId,
    source: PixelBuffer,
    factor: f64,
    image: bool,
}

/// The menu command.
pub fn start(app: &mut FlashApp) {
    if app.doc().active_layer().is_none() {
        app.status = "Select a layer first.".into();
        return;
    }
    pending::run(app, Pending::UpscaleWindow);
}

/// The model is ready: opens the window.
pub fn open(app: &mut FlashApp) {
    let upscaler = app.ai.services.upscale(&app.ai.store);
    let tiles = upscaler.tile_count(app.doc().width() as usize, app.doc().height() as usize);
    app.dialog = Some(Dialog::Upscale(UpscaleForm { scale: 1.0, strength: 100.0, tiles }));
}

/// What can be improved on a layer: its pixels, or (for a placed picture) its own source pixels, enlarged enough to stay
/// sharp at the size it will be shown. Shapes, text and empty layers: nothing.
fn piece_of(app: &FlashApp, id: LayerId, scale: f64) -> Option<Piece> {
    let doc = app.doc();
    let layer = doc.layer(id)?;
    if layer.is_adjustment() {
        return None;
    }
    match layer.object() {
        Some(object) if object.kind() == "image" => {
            let source = (*pictures::decoded(object.picture()?)?).clone();
            let width = object.box_()?.width;
            let factor = (width * scale / f64::from(source.width())).clamp(1.0, 4.0);
            Some(Piece { layer: id, source, factor: (factor * 4.0).round() / 4.0, image: true })
        }
        Some(_) => None,
        None => {
            let source = layer.pixels().to_buffer(doc.bounds());
            if source.visible_bounds().is_empty() {
                return None;
            }
            Some(Piece { layer: id, source, factor: scale, image: false })
        }
    }
}

/// OK in the window: runs the AI.
pub fn run(app: &mut FlashApp, scale: f64, strength: f64) {
    let (w, h) = (app.doc().width(), app.doc().height());
    let (new_w, new_h) = ((f64::from(w) * scale).round() as u32, (f64::from(h) * scale).round() as u32);
    if new_w > canvas_ops::MAX_SIDE as u32 || new_h > canvas_ops::MAX_SIDE as u32 {
        outcome::tell(
            app,
            TITLE,
            format!(
                "The result would be {new_w} x {new_h} px, larger than the {} px limit. Choose a smaller size.",
                canvas_ops::MAX_SIDE
            ),
        );
        return;
    }
    app.tools.reset_active();
    let ids: Vec<LayerId> = if scale <= 1.0 {
        let Some(active) = checks::editable_layer(app) else { return };
        vec![active]
    } else {
        app.doc().layers().iter().filter(|l| l.visible).map(fe_core::document::Layer::id).collect()
    };
    let pieces: Vec<Piece> = ids.into_iter().filter_map(|id| piece_of(app, id, scale)).collect();
    if pieces.is_empty() {
        let text = if scale <= 1.0 {
            "The selected layer has nothing the AI can improve (it is empty, or a shape / text, which are always sharp). \
             Select the layer with your picture in the Layers panel."
        } else {
            "There is no picture to improve here (only shapes / text, which are always sharp)."
        };
        outcome::tell(app, TITLE, text);
        return;
    }
    let doc = app.doc().uid();
    let upscaler = app.ai.services.upscale(&app.ai.store);
    let mix = strength / 100.0;
    app.status = "The AI is working. You can cancel in the status bar; nothing changes until it finishes.".into();
    job::start(app, TITLE, move |progress: &Progress| {
        let count = pieces.len();
        let mut items = Vec::new();
        for (i, piece) in pieces.into_iter().enumerate() {
            let report = |f: f64| progress.report((i as f64 + f) / count as f64);
            let ai = match upscaler.upscale(&piece.source, piece.factor, &report, progress.flag()) {
                Ok(ai) => ai,
                Err(e) => return AiDone::Upscale { doc, result: Err(e) },
            };
            let pixels = if mix >= 0.999 { ai } else { mixed(&piece.source, &ai, mix) };
            items.push(Improved { layer: piece.layer, pixels, image: piece.image });
        }
        AiDone::Upscale { doc, result: Ok(Upscaled { scale, device: upscaler.device_name(), items }) }
    });
}

/// The plain enlargement blended with the AI result (`strength` 0..1).
fn mixed(source: &PixelBuffer, ai: &PixelBuffer, strength: f64) -> PixelBuffer {
    let plain = resize(source, ai.width(), ai.height(), ResampleMode::Lanczos);
    let mut out = ai.clone();
    for (i, v) in out.data_mut().iter_mut().enumerate() {
        if i % 4 != 3 {
            let p = f64::from(plain.data()[i]);
            *v = (p + (f64::from(*v) - p) * strength).round().clamp(0.0, 255.0) as u8;
        }
    }
    out
}

fn put_pixels(layer: &mut fe_core::document::Layer, pixels: &PixelBuffer) {
    let mut tiles = TileMap::default();
    tiles.write_buffer(pixels, 0, 0);
    layer.replace_pixels(tiles);
}

pub fn on_done(app: &mut FlashApp, doc: u64, result: Result<Upscaled, String>) {
    let done = match result {
        Ok(done) => done,
        Err(e) => return outcome::failed(app, TITLE, &e),
    };
    if app.doc().uid() != doc {
        app.status = "Another picture is shown now, so the result was dropped.".into();
        return;
    }
    let canvas = app.doc().bounds();
    if done.scale <= 1.0 {
        app.project_mut().edit("AI Clean Up", None, |doc| {
            for item in &done.items {
                let Some(layer) = doc.layer_mut(item.layer) else { continue };
                match (item.image, layer.object().cloned()) {
                    (true, Some(object)) => object_ops::set_object(layer, object.with_source(&item.pixels), canvas),
                    _ => {
                        let mut tiles = layer.pixels().clone();
                        tiles.write_buffer(&item.pixels, 0, 0);
                        layer.replace_pixels(tiles);
                    }
                }
            }
        });
        app.status = format!("AI clean-up done (on {}). Ctrl+Z to undo.", done.device);
        return;
    }
    let (ow, oh) = (app.doc().width(), app.doc().height());
    let (w, h) = ((f64::from(ow) * done.scale).round() as u32, (f64::from(oh) * done.scale).round() as u32);
    let (sx, sy) = (f64::from(w) / f64::from(ow), f64::from(h) / f64::from(oh));
    let name = format!("AI Upscale {:.0}x", done.scale);
    app.project_mut().edit(&name, None, |doc| {
        transform_all(
            doc,
            &|p: &PixelBuffer| resize(p, w, h, ResampleMode::Lanczos),
            Some(&|o, _, _| o.scaled(sx, sy)),
        );
        let new_canvas = doc.bounds();
        for item in &done.items {
            let Some(layer) = doc.layer_mut(item.layer) else { continue };
            match (item.image, layer.object().cloned()) {
                (true, Some(object)) => object_ops::set_object(layer, object.with_source(&item.pixels), new_canvas),
                _ => put_pixels(layer, &item.pixels),
            }
        }
    });
    app.tools.reset_active();
    app.current_view_mut().fit();
    app.status = format!(
        "AI upscale done on {}: now {} x {} px. Zoom in (100%) to see the detail. Ctrl+Z to undo.",
        done.device,
        app.doc().width(),
        app.doc().height()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_time_estimate_grows_with_the_tile_count() {
        let small = UpscaleForm { scale: 1.0, strength: 100.0, tiles: 10 }.time_text();
        let big = UpscaleForm { scale: 1.0, strength: 100.0, tiles: 600 }.time_text();
        assert_eq!(small, "about 2 s with a graphics card, up to 1 min without one");
        assert_eq!(big, "about 72 s with a graphics card, up to 20 min without one");
    }

    #[test]
    fn strength_mixes_the_plain_enlargement_and_the_ai_result() {
        let source = PixelBuffer::filled(2, 2, fe_core::imaging::ColorRgba::rgb(100, 100, 100));
        let ai = PixelBuffer::filled(2, 2, fe_core::imaging::ColorRgba::rgb(200, 200, 200));
        assert_eq!(mixed(&source, &ai, 0.5).pixel(0, 0).r, 150);
        assert_eq!(mixed(&source, &ai, 0.0).pixel(0, 0).r, 100);
    }
}
