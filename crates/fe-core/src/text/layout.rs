//! Placing letters: lines, alignment, line / letter / word gaps (C# `SpacedText` and the WPF text layout).
//! The result is in the text's own "natural" space: (0, 0) = top-left of the natural size, with the 1 pixel margin
//! the C# renderer leaves. The object's box then stretches this natural drawing. The places between letters
//! (`CaretPos`) come with it, for editing the text on the canvas.

use rustybuzz::ttf_parser::GlyphId;
use rustybuzz::{Face, UnicodeBuffer};
use tiny_skia::{Path, PathBuilder};

use super::arc;
use super::data::{TextAlign, TextData};
use super::fonts;
use super::outline::glyph_path;

/// One shaped letter: where it starts on its line, how wide it is.
pub(super) struct Placed {
    pub glyph: GlyphId,
    /// Which character of the line the letter starts at (a ligature covers several).
    pub index: usize,
    pub x: f64,
    pub advance: f64,
}

/// One line of text, shaped.
pub(super) struct Line {
    pub letters: Vec<Placed>,
    pub width: f64,
    /// How many characters the line has.
    pub chars: usize,
}

/// A place between two characters where the text cursor can stand, in natural space: a short line from `a` (top)
/// to `b` (bottom). `index` counts characters of the whole text, a line break counting as one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretPos {
    pub index: usize,
    pub line: usize,
    /// How far along its line the place is (for moving up and down).
    pub along: f64,
    pub a: (f64, f64),
    pub b: (f64, f64),
}

/// A text drawing: all letters as one outline, the natural size, and the places for the text cursor.
pub struct Laid {
    pub path: Option<Path>,
    pub width: f64,
    pub height: f64,
    pub carets: Vec<CaretPos>,
}

/// The distance between two lines (C# `FontFamily.LineSpacing * size * LineSpacing`).
pub(super) fn line_height(face: &Face, t: &TextData) -> f64 {
    let upem = f64::from(face.units_per_em());
    let natural =
        f64::from(i32::from(face.ascender()) - i32::from(face.descender()) + i32::from(face.line_gap())) / upem;
    let factor = if (t.line_spacing - 1.0).abs() > 0.001 { t.line_spacing } else { 1.0 };
    (natural * t.size.max(1.0) * factor).max(1.0)
}

/// The font's own line height.
pub(super) fn natural_height(face: &Face, size: f64) -> f64 {
    f64::from(i32::from(face.ascender()) - i32::from(face.descender()) + i32::from(face.line_gap()))
        / f64::from(face.units_per_em())
        * size
}

pub(super) fn ascent(face: &Face, size: f64) -> f64 {
    f64::from(face.ascender()) / f64::from(face.units_per_em()) * size
}

pub(super) fn lines_of(t: &TextData) -> Vec<String> {
    let text = if t.text.is_empty() { " " } else { t.text.as_str() };
    text.replace('\r', "").split('\n').map(str::to_owned).collect()
}

/// Shapes a line with kerning; letter and word gaps push the letters apart.
pub(super) fn shape_line(face: &Face, line: &str, t: &TextData) -> Line {
    let size = t.size.max(1.0);
    let scale = size / f64::from(face.units_per_em());
    let (letter_gap, word_gap) = (t.letter_spacing * size, t.word_spacing * size);
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(line);
    // Ligatures join letters; with extra spacing every letter stands alone.
    let features = if letter_gap != 0.0 || word_gap != 0.0 {
        vec![rustybuzz::Feature::new(rustybuzz::ttf_parser::Tag::from_bytes(b"liga"), 0, ..)]
    } else {
        Vec::new()
    };
    let shaped = rustybuzz::shape(face, &features, buffer);
    let chars: Vec<(usize, char)> = line.char_indices().collect();
    let mut letters = Vec::new();
    let mut pen = 0.0;
    for (info, pos) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
        let index = chars.partition_point(|(byte, _)| *byte < info.cluster as usize);
        let spaces = chars[..index].iter().filter(|(_, c)| *c == ' ').count();
        let advance = f64::from(pos.x_advance) * scale;
        let x = pen + f64::from(pos.x_offset) * scale + index as f64 * letter_gap + spaces as f64 * word_gap;
        letters.push(Placed { glyph: GlyphId(info.glyph_id as u16), index, x, advance });
        pen += advance;
    }
    let width = letters.last().map_or(0.0, |l| (l.x + l.advance).max(1.0));
    Line { letters, width, chars: chars.len() }
}

/// Where each character boundary of a line lies along it: `chars + 1` numbers (a ligature is split evenly).
pub(super) fn boundaries(line: &Line) -> Vec<f64> {
    let mut xs = vec![0.0; line.chars + 1];
    if line.chars == 0 {
        return xs;
    }
    for (n, l) in line.letters.iter().enumerate() {
        let end = line.letters.get(n + 1).map_or(line.chars, |next| next.index).min(line.chars);
        let count = end.saturating_sub(l.index).max(1);
        for j in 0..count {
            if let Some(slot) = xs.get_mut(l.index + j) {
                *slot = l.x + l.advance * j as f64 / count as f64;
            }
        }
    }
    xs[line.chars] = line.letters.last().map_or(0.0, |l| l.x + l.advance);
    xs
}

/// Lays the text out. `None` when no font can be found at all.
pub fn lay_out(t: &TextData) -> Option<Laid> {
    fonts::with_face(&t.family, t.bold, t.italic, |face| {
        let mut lines: Vec<Line> = lines_of(t).iter().map(|l| shape_line(face, l, t)).collect();
        if t.text.is_empty() {
            lines[0].chars = 0; // an empty text is drawn as one space, but has no characters to stand between
        }
        if t.arc.abs() >= 0.5 {
            return arc::lay_out_arc(face, t, &lines);
        }
        let size = t.size.max(1.0);
        let (lh, asc) = (line_height(face, t), ascent(face, size));
        let natural = natural_height(face, size);
        let spaced = t.letter_spacing != 0.0 || t.word_spacing != 0.0;
        let max_width = lines.iter().map(|l| l.width).fold(1.0, f64::max);
        let mut builder = PathBuilder::new();
        let mut carets = Vec::new();
        let mut base = 0;
        for (n, line) in lines.iter().enumerate() {
            let offset = match t.align {
                TextAlign::Left => 0.0,
                TextAlign::Center => (max_width - line.width) / 2.0,
                TextAlign::Right => max_width - line.width,
            };
            // WPF puts the extra room of a bigger line spacing above the letters (unless every letter is placed by hand).
            let extra = if spaced { 0.0 } else { lh - natural };
            let baseline = 1.0 + n as f64 * lh + asc + extra;
            for l in &line.letters {
                if let Some(p) = glyph_path(face, l.glyph, size, 1.0 + offset + l.x, baseline) {
                    builder.push_path(&p);
                }
            }
            for (col, x) in boundaries(line).into_iter().enumerate() {
                let along = offset + x;
                carets.push(CaretPos {
                    index: base + col,
                    line: n,
                    along,
                    a: (1.0 + along, baseline - asc),
                    b: (1.0 + along, baseline - asc + natural.min(lh)),
                });
            }
            base += line.chars + 1;
        }
        Laid {
            path: builder.finish(),
            width: max_width.ceil() + 2.0,
            height: (lh * lines.len() as f64).ceil() + 2.0,
            carets,
        }
    })
}
