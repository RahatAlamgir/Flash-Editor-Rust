//! Remove Brush (AI) (C# `RemoveBrushTool`): paint over what you want gone; when you let go the AI paints in what would
//! plausibly be behind it, on a new "AI Fill" layer. While you paint, the area shows as a selection.

use std::time::Instant;

use egui::Key;
use fe_core::imaging::PointD;

use super::{Pointer, dab};
use crate::ai::masks::selection_from_canvas_mask;
use crate::app::FlashApp;

/// Remove Brush settings.
pub struct RemoveOptions {
    pub size: f64,
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self { size: 30.0 }
    }
}

/// A stroke in progress.
pub struct Stroke {
    mask: Vec<u8>,
    last: Option<PointD>,
    shown: Instant,
}

fn add_dab(mask: &mut [u8], width: usize, height: usize, cx: f64, cy: f64, radius: f64) {
    let x0 = (cx - radius).floor().max(0.0) as usize;
    let x1 = ((cx + radius).ceil() as i64).clamp(0, width as i64 - 1) as usize;
    let y0 = (cy - radius).floor().max(0.0) as usize;
    let y1 = ((cy + radius).ceil() as i64).clamp(0, height as i64 - 1) as usize;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
            if dx * dx + dy * dy <= radius * radius {
                mask[y * width + x] = 255;
            }
        }
    }
}

fn show(app: &mut FlashApp) {
    let (w, h) = (app.doc().width(), app.doc().height());
    let Some(stroke) = app.tools.ai.remove_stroke.as_mut() else { return };
    stroke.shown = Instant::now();
    let selection = selection_from_canvas_mask(&stroke.mask, w, h);
    app.project_mut().active_page_mut().doc.set_selected_area(selection);
}

pub fn pointer(app: &mut FlashApp, p: &Pointer) {
    let size = app.tools.ai.remove.size;
    let (w, h) = (app.doc().width() as usize, app.doc().height() as usize);
    if p.pressed {
        app.tools.ai.remove_stroke = Some(Stroke { mask: vec![0; w * h], last: None, shown: Instant::now() });
    }
    let Some(stroke) = app.tools.ai.remove_stroke.as_mut() else { return };
    let (mask, last) = (&mut stroke.mask, stroke.last);
    dab::along(last, p.pos, size, |x, y| add_dab(mask, w, h, x, y, size / 2.0));
    stroke.last = Some(p.pos);
    if p.released || !p.down {
        show(app);
        app.tools.ai.remove_stroke = None;
        crate::ai::fill::start(app); // lets the AI fill what was painted
    } else if stroke.shown.elapsed().as_millis() > 80 {
        show(app);
    }
}

/// [ and ] change the brush size.
pub fn key(app: &mut FlashApp, key: Key) -> bool {
    let o = &mut app.tools.ai.remove;
    match key {
        Key::OpenBracket => o.size = (o.size * 0.8).round().clamp(2.0, 500.0),
        Key::CloseBracket => o.size = (o.size * 1.25).round().clamp(2.0, 500.0),
        _ => return false,
    }
    true
}
