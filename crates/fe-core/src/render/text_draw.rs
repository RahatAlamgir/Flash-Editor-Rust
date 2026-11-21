//! Drawing a text object: the letters as outlines, an outline around them, a color or gradient inside.

use tiny_skia::{LineCap, LineJoin, Paint, Pixmap, Shader, Stroke, Transform};

use super::draw::{color, gradient_shader, solid};
use crate::objects::{ObjectBox, StoredObject};
use crate::text::{TextData, lay_out};

/// How big the text is by itself (C# `MeasureText`): the box of a new text object, and what the box follows when
/// the text changes.
pub fn measure_text(text: &TextData) -> (f64, f64) {
    match lay_out(text) {
        Some(laid) => (laid.width, laid.height),
        None => (text.size.max(1.0), text.size.max(1.0)),
    }
}

pub(super) fn draw_text(pm: &mut Pixmap, obj: &StoredObject, b: &ObjectBox, t: Transform) {
    let Some(text) = obj.text() else { return };
    let Some(laid) = lay_out(&text) else { return };
    let Some(path) = &laid.path else { return };
    // The box may have been resized: the natural drawing stretches to fill it.
    let t = t.pre_scale((b.width / laid.width) as f32, (b.height / laid.height) as f32);
    if text.outline_width > 0.0
        && let Some(c) = text.outline_color
    {
        // Drawn first, twice as wide, so only its outer half shows once the letters are on top.
        let stroke = Stroke {
            width: (text.outline_width * 2.0) as f32,
            line_join: LineJoin::Round,
            line_cap: LineCap::Round,
            ..Stroke::default()
        };
        pm.stroke_path(path, &solid(c), &stroke, t, None);
    }
    let mut paint: Paint = solid(text.color);
    if let Some(g) = obj.gradient()
        && let Some(shader) = gradient_shader(&g, laid.width, laid.height)
    {
        paint.shader = shader;
    } else {
        paint.shader = Shader::SolidColor(color(text.color));
    }
    pm.fill_path(path, &paint, tiny_skia::FillRule::Winding, t, None);
}
