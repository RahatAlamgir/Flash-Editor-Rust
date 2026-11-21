//! Drawing a barcode as vectors in its own box (C# `BarcodeRenderer`).

use tiny_skia::{PathBuilder, Pixmap, Rect, Transform};

use super::label::Label;
use super::qr_draw::crossed_box;
use super::vector::Canvas;
use crate::codes::barcode;
use crate::imaging::ColorRgba;
use crate::objects::{BarcodeObjData, ObjectBox};

/// The empty margin left and right, in modules.
const QUIET: f64 = 10.0;
const FONT: &str = "Consolas";

pub(super) fn draw_barcode(pm: &mut Pixmap, code: &BarcodeObjData, b: &ObjectBox, t: Transform) {
    let mut c = Canvas { pm, t };
    let (width, height) = (b.width, b.height);
    if !code.transparent_background {
        c.rect(0.0, 0.0, width, height, code.background);
    }
    let data = match barcode::encode(&code.content, code.kind) {
        Ok(d) => d,
        Err(message) => {
            // The text does not fit this kind of barcode: say why instead of drawing something wrong.
            crossed_box(&mut c, width, height);
            let label =
                Label::fit(&message, (height * 0.11).clamp(7.0, 16.0), false, "Segoe UI", (width - 12.0).max(10.0));
            c.label(&label, 6.0, ((height - label.height) / 2.0).max(4.0), ColorRgba::rgb(185, 28, 28));
            return;
        }
    };
    let n = data.width() as f64;
    let module = width / (n + QUIET * 2.0);
    let left = module * QUIET;
    let font = if code.show_text { (height * 0.2).min(module * 9.0).clamp(6.0, 40.0) } else { 0.0 };
    let text_zone = if code.show_text { font * 1.35 } else { 0.0 };
    let top = height * 0.05;
    let bar_bottom = (top + 2.0).max(height - text_zone - height * 0.04);

    // The bars: neighbouring dark modules are drawn as one rectangle.
    let mut pb = PathBuilder::new();
    let mut i = 0;
    while i < data.modules.len() {
        if !data.modules[i] {
            i += 1;
            continue;
        }
        let start = i;
        while i < data.modules.len() && data.modules[i] {
            i += 1;
        }
        let guard = data.guards.iter().any(|g| start >= g.0 && i <= g.1);
        let bottom =
            if guard && code.show_text { (height - height * 0.04).min(bar_bottom + font * 0.6) } else { bar_bottom };
        let (x0, x1) = (left + start as f64 * module, left + i as f64 * module);
        if let Some(r) = Rect::from_xywh(x0 as f32, top as f32, (x1 - x0) as f32, (bottom - top).max(0.1) as f32) {
            pb.push_rect(r);
        }
    }
    if let Some(path) = pb.finish() {
        c.fill(&path, code.foreground);
    }
    if code.show_text {
        for (text, center) in &data.labels {
            let label = Label::fit(text, font, true, FONT, n * module);
            let y = height - text_zone - height * 0.02 + (text_zone - label.height) / 2.0;
            c.label(&label, left + center * module - label.width / 2.0, y, code.foreground);
        }
    }
}
