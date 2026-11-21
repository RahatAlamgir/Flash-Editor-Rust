//! Text on a curve (C# `ArcText`): every letter sits on a circle and is turned to match it. A positive arc bends the
//! text like a rainbow, a negative one like a smile.

use rustybuzz::Face;
use tiny_skia::{PathBuilder, Transform};

use super::data::{TextAlign, TextData};
use super::layout::{CaretPos, Laid, Line, ascent, boundaries, line_height, natural_height};
use super::outline::{glyph_path, transformed};

pub(super) fn lay_out_arc(face: &Face, t: &TextData, lines: &[Line]) -> Laid {
    let size = t.size.max(1.0);
    let lh = line_height(face, t);
    let (asc, natural) = (ascent(face, size), natural_height(face, size).min(lh));
    let total = lines.iter().map(|l| l.width).fold(1.0, f64::max);
    let arc = t.arc.clamp(-360.0, 360.0).to_radians();
    let radius = total / arc.abs().max(1e-6); // the longest line takes the whole arc
    let up = arc > 0.0;

    let mut builder = PathBuilder::new();
    let mut carets = Vec::new();
    let mut base = 0;
    for (n, line) in lines.iter().enumerate() {
        // The next line is on a smaller circle for a rainbow (closer to the middle) and a bigger one for a smile.
        let line_radius = if up { (radius - n as f64 * lh).max(1.0) } else { radius + n as f64 * lh };
        let offset = match t.align {
            TextAlign::Left => 0.0,
            TextAlign::Right => total - line.width,
            TextAlign::Center => (total - line.width) / 2.0,
        };
        // Where a place `along` the line is on the circle, and which way is up there.
        let spot = |along: f64| {
            let phi = (along - total / 2.0) / line_radius; // angle from the middle of the arc
            let (x, y) = (
                line_radius * phi.sin(),
                if up { -line_radius * phi.cos() + radius } else { line_radius * phi.cos() - radius },
            );
            (x, y, if up { phi } else { -phi })
        };
        for l in &line.letters {
            let (x, y, turn) = spot(offset + l.x + l.advance / 2.0);
            // The letter with the middle of its baseline at the origin, its top pointing up.
            let Some(shape) = glyph_path(face, l.glyph, size, -l.advance / 2.0, 0.0) else { continue };
            let place = Transform::from_translate(x as f32, y as f32).pre_rotate(turn.to_degrees() as f32);
            if let Some(p) = transformed(&shape, place) {
                builder.push_path(&p);
            }
        }
        for (col, x) in boundaries(line).into_iter().enumerate() {
            let along = offset + x;
            let (px, py, turn) = spot(along);
            let (s, c) = turn.sin_cos();
            // Up from the baseline is (0, -1) turned by `turn`; down is the other way.
            carets.push(CaretPos {
                index: base + col,
                line: n,
                along,
                a: (px + asc * s, py - asc * c),
                b: (px - (natural - asc) * s, py + (natural - asc) * c),
            });
        }
        base += line.chars + 1;
    }
    // Natural size = the letters' bounds with a margin; the drawing is moved so its top-left lands on the margin.
    let margin = 1.0 + t.outline_width;
    let (min_x, min_y, width, height, path) = match builder.finish() {
        Some(path) => {
            let b = path.bounds();
            (
                f64::from(b.x()),
                f64::from(b.y()),
                f64::from(b.width()).ceil() + 2.0 + t.outline_width * 2.0,
                f64::from(b.height()).ceil() + 2.0 + t.outline_width * 2.0,
                Some(path),
            )
        }
        None => {
            let min_x = carets.iter().map(|c| c.a.0.min(c.b.0)).fold(f64::MAX, f64::min);
            let min_y = carets.iter().map(|c| c.a.1.min(c.b.1)).fold(f64::MAX, f64::min);
            (min_x, min_y, size.max(1.0), size.max(1.0), None)
        }
    };
    let (dx, dy) = (margin - min_x, margin - min_y);
    for c in &mut carets {
        c.a = (c.a.0 + dx, c.a.1 + dy);
        c.b = (c.b.0 + dx, c.b.1 + dy);
    }
    let moved = path.and_then(|p| transformed(&p, Transform::from_translate(dx as f32, dy as f32)));
    Laid { path: moved, width, height, carets }
}
