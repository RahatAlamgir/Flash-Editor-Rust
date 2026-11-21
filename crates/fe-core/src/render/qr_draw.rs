//! Drawing a QR code as vectors in its own box (C# `QrRenderer`).

use tiny_skia::{PathBuilder, Pixmap, Rect, Transform};

use super::vector::{Canvas, arc, round_rect_path};
use crate::codes::encode_qr;
use crate::imaging::ColorRgba;
use crate::objects::{ObjectBox, QrData, QrStyle};

/// A crossed-out box: the text cannot be shown (too long for a QR code, or no room).
pub(super) fn crossed_box(c: &mut Canvas, width: f64, height: f64) {
    let red = ColorRgba::rgb(220, 38, 38);
    let line = (width.min(height) * 0.02).max(1.0);
    let (w, h) = ((width - 4.0).max(1.0), (height - 4.0).max(1.0));
    if let Some(r) = Rect::from_xywh(2.0, 2.0, w as f32, h as f32) {
        c.stroke(&PathBuilder::from_rect(r), red, line, false);
    }
    c.line((2.0, 2.0), (width - 2.0, height - 2.0), red, line, false);
    c.line((width - 2.0, 2.0), (2.0, height - 2.0), red, line, false);
}

pub(super) fn draw_qr(pm: &mut Pixmap, qr: &QrData, b: &ObjectBox, t: Transform) {
    let mut c = Canvas { pm, t };
    let (width, height) = (b.width, b.height);
    if !qr.transparent_background {
        c.rect(0.0, 0.0, width, height, qr.background);
    }
    let text = if qr.content.is_empty() { " " } else { qr.content.as_str() };
    let Ok(modules) = encode_qr(text, qr.level) else {
        crossed_box(&mut c, width, height);
        return;
    };
    let n = modules.len();
    let quiet = f64::from(qr.quiet.clamp(0, 12));
    let module = width.min(height) / (n as f64 + 2.0 * quiet);
    let (left, top) = ((width - module * n as f64) / 2.0, (height - module * n as f64) / 2.0);
    let mut pb = PathBuilder::new();
    for (y, row) in modules.iter().enumerate() {
        for (x, dark) in row.iter().enumerate() {
            if !dark {
                continue;
            }
            let (px, py) = (left + x as f64 * module, top + y as f64 * module);
            // The three big squares always stay square.
            let finder = (x < 7 && y < 7) || (x >= n - 7 && y < 7) || (x < 7 && y >= n - 7);
            if qr.style == QrStyle::Square || finder {
                // A hair larger: no gaps between neighbors.
                if let Some(r) = Rect::from_xywh(px as f32, py as f32, (module + 0.15) as f32, (module + 0.15) as f32) {
                    pb.push_rect(r);
                }
            } else if qr.style == QrStyle::Rounded {
                if let Some(p) = round_rect_path(px, py, module, module, module * 0.32) {
                    pb.push_path(&p);
                }
            } else {
                let (cx, cy, r) = (px + module / 2.0, py + module / 2.0, module * 0.46);
                pb.move_to((cx - r) as f32, cy as f32);
                arc(&mut pb, cx, cy, r, std::f64::consts::PI, std::f64::consts::TAU);
                pb.close();
            }
        }
    }
    if let Some(path) = pb.finish() {
        c.fill(&path, qr.foreground);
    }
}
