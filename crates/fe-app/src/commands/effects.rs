//! Adjustments and filters (C# `EffectCommands`, `EffectPreviewSession`): an effect without settings runs at once;
//! one with settings opens a small window with a live preview. The preview is made on a background thread (a newer
//! change of a slider replaces an older one still running) and shown on the layer itself; OK makes it one undo step,
//! Cancel puts the layer back.

use std::sync::Arc;

use fe_core::document::{Document, LayerId};
use fe_core::effects::{self, Effect, Values};
use fe_core::imaging::{PixelBuffer, TileMap};
use fe_core::selection::Selection;

use super::checks;
use crate::app::FlashApp;
use crate::app::jobs::JobDone;

/// The effect being previewed.
pub struct EffectSession {
    pub effect: &'static Effect,
    pub values: Values,
    pub defaults: Values,
    layer: LayerId,
    /// The whole document as it was before (the undo step goes from there).
    before: Document,
    /// The layer's pixels on the canvas, and all its pixels (some may lie outside the canvas).
    original: Arc<PixelBuffer>,
    all_pixels: TileMap,
    selection: Option<Selection>,
    generation: u64,
    running: bool,
    dirty: bool,
    /// A note under the sliders.
    pub note: &'static str,
    /// True until the first preview was asked for.
    pub fresh: bool,
}

impl EffectSession {
    /// True while a preview is being made.
    pub fn busy(&self) -> bool {
        self.running || self.dirty
    }
}

/// The pixels of the layer with the effect applied (nothing outside the canvas or the selection changes).
fn processed(
    session_original: &PixelBuffer,
    all: &TileMap,
    effect: &Effect,
    values: &Values,
    sel: Option<&Selection>,
) -> TileMap {
    let out = effects::apply_to_selection(session_original, effect, values, sel);
    let mut tiles = all.clone();
    tiles.write_buffer(&out, 0, 0);
    tiles
}

fn show_pixels(app: &mut FlashApp, layer: LayerId, tiles: TileMap) {
    if let Some(l) = app.project_mut().active_page_mut().doc.layer_mut(layer) {
        l.replace_pixels(tiles);
    }
}

/// Runs an adjustment or filter on the active layer.
pub fn run(app: &mut FlashApp, effect: &'static Effect) {
    if matches!(app.dialog, Some(crate::dialogs::Dialog::Effect(_))) {
        return;
    }
    let Some(id) = checks::editable_layer(app) else { return };
    let canvas = app.doc().bounds();
    let Some(layer) = app.doc().layer(id) else { return };
    let all_pixels = layer.pixels().clone();
    let original = Arc::new(all_pixels.to_buffer(canvas));
    let selection = app.doc().selected_area().cloned();
    if std::ptr::eq(effect, &effects::REMOVE_BY_COLOR) && crate::ai::remove_bg::visible_count(&original) == 0 {
        let name = app.doc().layer(id).map(|l| l.name.clone()).unwrap_or_default();
        crate::ai::outcome::tell(
            app,
            "Remove Background",
            format!(
                "The selected layer \"{name}\" is empty, so there is nothing to remove.

Click the layer that contains your picture in the Layers panel, then try again."
            ),
        );
        return;
    }
    if effect.params.is_empty() {
        let tiles = processed(&original, &all_pixels, effect, &effect.defaults(), selection.as_ref());
        app.project_mut().edit(effect.name, None, |doc| {
            if let Some(l) = doc.layer_mut(id) {
                l.replace_pixels(tiles);
            }
        });
        app.status = format!("{} applied.", effect.name);
        return;
    }
    let mut values = effect.defaults();
    let mut note = "";
    if std::ptr::eq(effect, &effects::RECOLOR) {
        let (_, l, s) = effects::measure(&original, selection.as_ref());
        values = effects::recolor_values(app.foreground, l, s);
        note =
            "Changes the color of the picked area (the selection, or the whole layer) but keeps its light and shade.";
    }
    let session = EffectSession {
        effect,
        defaults: values.clone(),
        values,
        layer: id,
        before: app.doc().clone(),
        original,
        all_pixels,
        selection,
        generation: 0,
        running: false,
        dirty: false,
        note,
        fresh: true,
    };
    app.dialog = Some(crate::dialogs::Dialog::Effect(Box::new(session)));
}

/// Starts (or queues) a new preview for the current values.
pub fn request_preview(app: &mut FlashApp, ctx: &egui::Context, session: &mut EffectSession) {
    session.generation += 1;
    if session.running {
        session.dirty = true; // started again when the running one is done
        return;
    }
    start(app, ctx, session);
}

fn start(app: &mut FlashApp, ctx: &egui::Context, s: &mut EffectSession) {
    s.running = true;
    s.dirty = false;
    let (original, all, effect, values, sel, generation) =
        (s.original.clone(), s.all_pixels.clone(), s.effect, s.values.clone(), s.selection.clone(), s.generation);
    app.jobs.spawn(ctx, move || {
        let tiles = processed(&original, &all, effect, &values, sel.as_ref());
        JobDone::EffectPreview { generation, tiles }
    });
}

/// A preview was made: show it (when it is still the newest) and start the next one if the sliders moved on.
pub fn on_preview(app: &mut FlashApp, ctx: &egui::Context, generation: u64, tiles: TileMap) {
    let Some(crate::dialogs::Dialog::Effect(mut session)) = app.dialog.take() else { return };
    session.running = false;
    if generation == session.generation {
        show_pixels(app, session.layer, tiles);
    }
    if session.dirty {
        start(app, ctx, &mut session);
    }
    app.dialog = Some(crate::dialogs::Dialog::Effect(session));
}

/// OK: the effect with the final values becomes one undo step.
pub fn commit(app: &mut FlashApp, session: EffectSession) {
    let tiles =
        processed(&session.original, &session.all_pixels, session.effect, &session.values, session.selection.as_ref());
    show_pixels(app, session.layer, tiles);
    if std::ptr::eq(session.effect, &effects::REMOVE_BY_COLOR) {
        // Like the AI version: the layer becomes a picture that remembers the original, so the Refine brush can restore.
        let after = app.doc().layer(session.layer).map(|l| l.pixels().to_buffer(app.doc().bounds()));
        if let Some(after) = after {
            let (before_count, after_count) =
                (crate::ai::remove_bg::visible_count(&session.original), crate::ai::remove_bg::visible_count(&after));
            crate::ai::remove_bg::make_cut_out(app, session.layer, &session.original, &after);
            app.project_mut().record(session.effect.name, session.before);
            crate::ai::remove_bg::report(app, before_count, after_count, false);
            return;
        }
    }
    app.project_mut().record(session.effect.name, session.before);
    app.status = format!("{} applied.", session.effect.name);
}

/// Cancel: the layer is as it was.
pub fn cancel(app: &mut FlashApp, session: EffectSession) {
    app.project_mut().active_page_mut().doc = session.before;
}
