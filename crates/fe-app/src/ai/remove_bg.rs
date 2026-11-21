//! Remove Background (AI) (C# `RemoveBackgroundWithAi`): a small window with three sliders and a live preview on the
//! picture. The AI looks at the picture once; the sliders only change how the edge is cut, so moving them is quick.
//! OK makes one undo step and turns the layer into a picture that remembers the original, so the Refine brush can
//! paint parts back.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use fe_ai::segmentation::SubjectSegmenter;
use fe_core::document::{Document, LayerId, object_ops};
use fe_core::imaging::{PixelBuffer, TileMap};
use fe_core::objects::{ObjectBox, StoredObject};
use fe_core::selection::Selection;

use super::outcome::{self, AiDone};
use super::pending::{self, Pending};
use crate::app::FlashApp;
use crate::app::jobs::JobDone;
use crate::commands::checks;
use crate::dialogs::Dialog;

const TITLE: &str = "Remove Background (AI)";

pub struct RemoveBgSession {
    layer: LayerId,
    before: Document,
    original: Arc<PixelBuffer>,
    all_pixels: TileMap,
    selection: Option<Selection>,
    segmenter: Arc<SubjectSegmenter>,
    /// Edge sharpness 0..100.
    pub sharpness: f64,
    /// Shrink (negative) / grow (positive) the edge, -50..50.
    pub shift: f64,
    /// Feather in pixels, 0..10.
    pub feather: f64,
    generation: u64,
    running: bool,
    dirty: bool,
    /// True until the first preview was asked for.
    pub fresh: bool,
    /// The newest finished preview (and the generation of settings it was made with).
    result: Option<(u64, PixelBuffer)>,
}

impl RemoveBgSession {
    /// True while a preview is being made.
    pub fn busy(&self) -> bool {
        self.running || self.dirty
    }

    /// OK can be pressed when the picture on screen is made with the current settings.
    pub fn ready(&self) -> bool {
        !self.busy() && self.result.as_ref().is_some_and(|(g, _)| *g == self.generation)
    }
}

/// The menu command: makes sure the model is there, then opens the window.
pub fn start(app: &mut FlashApp) {
    pending::run(app, Pending::RemoveBackground);
}

fn visible_count(pixels: &PixelBuffer) -> usize {
    pixels.data().iter().skip(3).step_by(4).filter(|a| **a > 127).count()
}

/// The model is ready: checks the layer and opens the window.
pub fn open(app: &mut FlashApp) {
    if matches!(app.dialog, Some(Dialog::RemoveBg(_))) {
        return;
    }
    let Some(id) = checks::editable_layer(app) else { return };
    let canvas = app.doc().bounds();
    let Some(layer) = app.doc().layer(id) else { return };
    let all_pixels = layer.pixels().clone();
    let original = Arc::new(all_pixels.to_buffer(canvas));
    if visible_count(&original) == 0 {
        let text = format!(
            "The selected layer \"{}\" is empty, so there is nothing to remove.\n\nClick the layer that contains your \
             picture in the Layers panel, then try again.",
            layer.name
        );
        outcome::tell(app, TITLE, text);
        return;
    }
    app.tools.reset_active();
    let session = RemoveBgSession {
        layer: id,
        before: app.doc().clone(),
        original,
        all_pixels,
        selection: app.doc().selected_area().cloned(),
        segmenter: app.ai.services.subject(&app.ai.store),
        sharpness: 30.0,
        shift: 0.0,
        feather: 0.0,
        generation: 0,
        running: false,
        dirty: false,
        fresh: true,
        result: None,
    };
    app.status = "The AI is finding the subject (a few seconds the first time)...".into();
    app.dialog = Some(Dialog::RemoveBg(Box::new(session)));
}

/// The cut-out of `original` with the settings; with a selection only the selected area changes.
fn process(
    segmenter: &SubjectSegmenter,
    original: &PixelBuffer,
    selection: Option<&Selection>,
    (sharpness, shift, feather): (f64, f64, f64),
) -> Result<PixelBuffer, String> {
    let never = AtomicBool::new(false);
    let cut = fe_ai::effects::remove_background(segmenter, original, sharpness / 100.0, shift / 50.0, feather, &never)?;
    let Some(sel) = selection else { return Ok(cut) };
    let bounds = sel.bounds().intersect(&original.bounds());
    let mut out = original.clone();
    let width = original.width() as usize;
    for y in bounds.y..bounds.y + bounds.height {
        for x in bounds.x..bounds.x + bounds.width {
            let coverage = sel.coverage(x, y);
            if coverage <= 0.0 {
                continue;
            }
            let i = (y as usize * width + x as usize) * 4;
            for c in 0..4 {
                let old = f64::from(out.data()[i + c]);
                let new = f64::from(cut.data()[i + c]);
                out.data_mut()[i + c] = (old + (new - old) * coverage).round() as u8;
            }
        }
    }
    Ok(out)
}

/// Starts (or queues) a new preview for the current sliders.
pub fn request_preview(app: &mut FlashApp, session: &mut RemoveBgSession) {
    session.generation += 1;
    if session.running {
        session.dirty = true; // started again when the running one is done
        return;
    }
    start_preview(app, session);
}

fn start_preview(app: &mut FlashApp, s: &mut RemoveBgSession) {
    let Some(ctx) = app.ai.ctx.clone() else { return };
    s.running = true;
    s.dirty = false;
    let (original, selection, segmenter) = (s.original.clone(), s.selection.clone(), s.segmenter.clone());
    let settings = (s.sharpness, s.shift, s.feather);
    let generation = s.generation;
    app.jobs.spawn(&ctx, move || {
        let result = process(&segmenter, &original, selection.as_ref(), settings);
        JobDone::Ai(AiDone::BackgroundPreview { generation, result })
    });
}

fn show_pixels(app: &mut FlashApp, layer: LayerId, buffer: &PixelBuffer, all: &TileMap) {
    let mut tiles = all.clone();
    tiles.write_buffer(buffer, 0, 0);
    if let Some(l) = app.project_mut().active_page_mut().doc.layer_mut(layer) {
        l.replace_pixels(tiles);
    }
}

/// A preview was made: show it (when it is the newest) and start the next one if the sliders moved on.
pub fn on_preview(app: &mut FlashApp, generation: u64, result: Result<PixelBuffer, String>) {
    let Some(Dialog::RemoveBg(mut session)) = app.dialog.take() else { return };
    session.running = false;
    match result {
        Ok(buffer) => {
            if generation == session.generation {
                show_pixels(app, session.layer, &buffer, &session.all_pixels);
            }
            session.result = Some((generation, buffer));
            if session.dirty {
                start_preview(app, &mut session);
            }
            app.dialog = Some(Dialog::RemoveBg(session));
        }
        Err(e) => {
            app.project_mut().active_page_mut().doc = session.before;
            outcome::failed(app, TITLE, &e);
        }
    }
}

/// OK: the cut-out with the final settings becomes one undo step, and the layer a picture that remembers the original.
pub fn commit(app: &mut FlashApp, session: RemoveBgSession) {
    let Some((_, result)) = session.result.as_ref() else { return cancel(app, session) };
    let before_count = visible_count(&session.original);
    let after_count = visible_count(result);
    show_pixels(app, session.layer, result, &session.all_pixels);
    make_cut_out(app, &session, result);
    app.project_mut().record(TITLE, session.before);
    report(app, before_count, after_count);
}

/// Turns the layer into a picture object that remembers the original (C# `MakeCutOutObject`).
fn make_cut_out(app: &mut FlashApp, session: &RemoveBgSession, result: &PixelBuffer) {
    let canvas = app.doc().bounds();
    let Some(layer) = app.project_mut().active_page_mut().doc.layer_mut(session.layer) else { return };
    if layer.has_hidden_pixels(canvas) {
        return;
    }
    let area = session.original.visible_bounds().union(&result.visible_bounds());
    if area.is_empty() {
        return;
    }
    let b = ObjectBox::new(f64::from(area.x), f64::from(area.y), f64::from(area.width), f64::from(area.height), 0.0);
    let name = layer.name.clone();
    let object = StoredObject::new_cutout(b, &result.crop(area), &session.original.crop(area), &name);
    object_ops::set_object(layer, object, canvas);
}

/// Cancel: the layer is as it was.
pub fn cancel(app: &mut FlashApp, session: RemoveBgSession) {
    app.project_mut().active_page_mut().doc = session.before;
    app.status = "Remove Background cancelled.".into();
}

fn report(app: &mut FlashApp, before: usize, after: usize) {
    let share = if before == 0 { 0.0 } else { (before - after.min(before)) as f64 / before as f64 };
    app.status = format!(
        "Remove Background: {:.0}% of the layer became transparent. Missed a spot? Use the Refine brush (R): Restore paints parts back, Remove takes more away.",
        share * 100.0
    );
    if share >= 0.01 {
        return;
    }
    let reason = if app.doc().selected_area().is_some() {
        "There is an active selection, so only the selected area was processed. Press Ctrl+Shift+D (or Esc) to deselect and try again.\n\n"
    } else {
        ""
    };
    outcome::tell(
        app,
        TITLE,
        format!(
            "Almost nothing was removed.\n\n{reason}The AI treated the whole picture as the subject. This happens when there is no clear main subject (e.g. a landscape) or the picture is already cut out. Try a negative \"Shrink / grow edge\" value."
        ),
    );
}
