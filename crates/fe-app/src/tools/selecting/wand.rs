//! Magic Wand (C# `MagicWandTool`): click a color to select the area of similar color. Quick Select brush
//! (C# `QuickSelectTool`): drag over an object and the selection spreads into similar colors around the brush;
//! each stroke adds, Alt+drag removes, [ and ] change the brush size.

use std::sync::Arc;
use std::time::Instant;

use egui::Key;
use fe_core::imaging::{PixelBuffer, PixelRect, PointD};
use fe_core::paint::compositor;
use fe_core::selection::{CombineMode, Selection, magic_wand, quick_select};

use super::apply;
use crate::app::FlashApp;
use crate::tools::Pointer;

/// Magic Wand settings (C# `MagicWandOptions`).
pub struct WandOptions {
    /// 0..255.
    pub tolerance: f64,
    /// Only the touching area (off: every similar color in the picture).
    pub contiguous: bool,
    /// Look at what you see (off: only the selected layer).
    pub all_layers: bool,
}

impl Default for WandOptions {
    fn default() -> Self {
        Self { tolerance: 32.0, contiguous: true, all_layers: true }
    }
}

/// Quick Select settings (C# `QuickSelectOptions`).
pub struct QuickOptions {
    /// Brush size, 2..500 px.
    pub size: f64,
    /// 1..255.
    pub tolerance: f64,
    pub all_layers: bool,
}

impl Default for QuickOptions {
    fn default() -> Self {
        Self { size: 30.0, tolerance: 32.0, all_layers: true }
    }
}

/// The picture the tools look at: everything you see, or only the active layer (canvas-sized).
fn source(app: &FlashApp, all_layers: bool) -> PixelBuffer {
    let doc = app.doc();
    match doc.active_layer().filter(|_| !all_layers) {
        Some(layer) => layer.pixels().to_buffer(doc.bounds()),
        None => compositor::flatten(doc),
    }
}

pub fn magic_wand(app: &mut FlashApp, p: &Pointer) {
    if !p.pressed {
        return;
    }
    let (x, y) = p.pixel();
    if !app.doc().bounds().contains(x, y) {
        return;
    }
    let options = &app.tools.select.wand;
    let (tolerance, contiguous, all) = (options.tolerance as u8, options.contiguous, options.all_layers);
    let image = source(app, all);
    let mask = magic_wand::similar_area(&image, x, y, tolerance, contiguous);
    let (w, h) = (image.width(), image.height());
    let drawn = Selection::from_mask(mask, image.bounds(), w, h);
    apply::apply(app, drawn, p.mods);
}

/// A quick-select stroke in progress.
pub struct QuickStroke {
    image: PixelBuffer,
    /// Canvas-sized: 255 where this stroke selected.
    mask: Vec<u8>,
    /// The part of `mask` this stroke touched.
    touched: PixelRect,
    start: Option<Arc<Selection>>,
    subtract: bool,
    last: PointD,
    shown: Instant,
}

fn dab(stroke: &mut QuickStroke, at: PointD, size: f64, tolerance: u8) {
    let changed = quick_select::add_dab(&stroke.image, &mut stroke.mask, at.x, at.y, size / 2.0, tolerance);
    stroke.touched = stroke.touched.union(&changed);
}

/// Shows the selection with the stroke so far (tracing the outline takes time, so at most every 70 ms).
fn show(app: &mut FlashApp, force: bool) {
    let Some(stroke) = app.tools.select.quick_stroke.as_mut() else { return };
    if !force && stroke.shown.elapsed().as_millis() < 70 {
        return;
    }
    stroke.shown = Instant::now();
    let t = stroke.touched;
    let (w, h) = (stroke.image.width(), stroke.image.height());
    let part = if t.is_empty() {
        None
    } else {
        let mut piece = Vec::with_capacity(t.area() as usize);
        for y in t.y..t.bottom() {
            let start = (y * w as i32 + t.x) as usize;
            piece.extend_from_slice(&stroke.mask[start..start + t.width as usize]);
        }
        Selection::from_mask(piece, t, w, h)
    };
    let mode = if stroke.subtract { CombineMode::Subtract } else { CombineMode::Add };
    let start = stroke.start.clone();
    let combined = Selection::combine(start.as_deref(), part.as_ref(), mode);
    app.project_mut().active_page_mut().doc.set_selected_area(combined);
}

pub fn quick_select(app: &mut FlashApp, p: &Pointer) {
    let (size, tolerance) = (app.tools.select.quick.size, app.tools.select.quick.tolerance as u8);
    if p.pressed {
        let image = source(app, app.tools.select.quick.all_layers);
        let mask = vec![0; image.data().len() / 4];
        let start = app.doc().selected_area_arc().cloned();
        let mut stroke = QuickStroke {
            image,
            mask,
            touched: PixelRect::EMPTY,
            start,
            subtract: p.mods.alt,
            last: p.pos,
            shown: Instant::now(),
        };
        dab(&mut stroke, p.pos, size, tolerance);
        app.tools.select.quick_stroke = Some(stroke);
        show(app, true);
        return;
    }
    let Some(stroke) = app.tools.select.quick_stroke.as_mut() else { return };
    // Dabs every quarter brush along the drag, so fast strokes leave no gaps.
    let step = (size / 4.0).max(1.0);
    let distance = stroke.last.distance_to(&p.pos);
    let mut t = step;
    while t <= distance {
        let f = t / distance;
        let at =
            PointD::new(stroke.last.x + (p.pos.x - stroke.last.x) * f, stroke.last.y + (p.pos.y - stroke.last.y) * f);
        dab(stroke, at, size, tolerance);
        t += step;
    }
    if distance >= step {
        stroke.last = p.pos;
    }
    if p.released {
        show(app, true);
        app.tools.select.quick_stroke = None;
        app.status = match app.doc().selected_area() {
            None => "Nothing selected.".into(),
            Some(s) => format!(
                "Selected {} x {} px. Keep dragging to add; Alt+drag removes. [ ] = brush size.",
                s.bounds().width,
                s.bounds().height
            ),
        };
    } else {
        show(app, false);
    }
}

/// [ and ] change the brush size. True when the key was used.
pub fn quick_key(app: &mut FlashApp, key: Key) -> bool {
    let options = &mut app.tools.select.quick;
    match key {
        Key::OpenBracket => options.size = (options.size * 0.8).round().clamp(2.0, 500.0),
        Key::CloseBracket => options.size = (options.size * 1.25).round().clamp(2.0, 500.0),
        _ => return false,
    }
    true
}
