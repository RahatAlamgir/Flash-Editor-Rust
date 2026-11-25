//! SVG (vector graphics) files (C# `SvgFile`). Saving writes shapes and text as real vector elements (they stay sharp at
//! any size) and everything else as embedded pictures. Opening understands the common simple elements: rectangles,
//! circles, ellipses, lines, polygons, paths (curves are flattened), text and embedded pictures, with fill, stroke and
//! opacity. Gradients, filters, clipping and fonts that are not installed are not supported.

use std::fmt::Write as _;
use std::path::Path;

use super::IoError;
use super::decode;
use super::pptx::{escape, write_file};
use crate::document::canvas_ops::bake_mask;
use crate::document::{Document, Layer, object_ops};
use crate::imaging::{BlendMode, ColorRgba, PointD};
use crate::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject, picture_bytes, pictures};
use crate::text::{TextAlign, TextData};

/// The extension, with the dot.
pub const EXTENSION: &str = ".svg";

/// True for a path that ends in .svg.
pub fn is_svg(path: impl AsRef<Path>) -> bool {
    path.as_ref().extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"))
}

fn f(v: f64) -> String {
    let text = format!("{v:.2}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed == "-0" || trimmed.is_empty() { "0".to_owned() } else { trimmed.to_owned() }
}

fn hex(c: ColorRgba) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

// ------------------------------------------------------------------------------------------------ base64

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let (mut bits, mut count) = (0u32, 0);
    for c in text.bytes().filter(|c| !c.is_ascii_whitespace() && *c != b'=') {
        let v = ALPHABET.iter().position(|a| *a == c)? as u32;
        bits = (bits << 6) | v;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
            bits &= (1 << count) - 1;
        }
    }
    Some(out)
}

// ------------------------------------------------------------------------------------------------- saving

fn blend_css(mode: BlendMode) -> &'static str {
    match mode {
        BlendMode::Normal => "normal",
        BlendMode::Multiply => "multiply",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::SoftLight => "soft-light",
        BlendMode::HardLight => "hard-light",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::Difference => "difference",
    }
}

fn common(layer: &Layer) -> String {
    let mut s = String::new();
    if layer.opacity() < 0.999 {
        let _ = write!(s, " opacity=\"{}\"", f(layer.opacity()));
    }
    if layer.blend != BlendMode::Normal {
        let _ = write!(s, " style=\"mix-blend-mode:{}\"", blend_css(layer.blend));
    }
    let _ = write!(s, " id=\"{}\"", escape(&layer.name));
    s
}

fn paint(attribute: &str, color: Option<ColorRgba>) -> String {
    match color {
        Some(c) if c.a > 0 => {
            let mut s = format!(" {attribute}=\"{}\"", hex(c));
            if c.a < 255 {
                let _ = write!(s, " {attribute}-opacity=\"{}\"", f(f64::from(c.a) / 255.0));
            }
            s
        }
        _ => format!(" {attribute}=\"none\""),
    }
}

fn transform(b: ObjectBox, flips: (bool, bool)) -> String {
    let c = b.center();
    let mut parts = Vec::new();
    if b.rotation.abs() > 0.001 {
        parts.push(format!("rotate({} {} {})", f(b.rotation), f(c.x), f(c.y)));
    }
    if flips.0 || flips.1 {
        parts.push(format!(
            "translate({} {}) scale({} {}) translate({} {})",
            f(c.x),
            f(c.y),
            if flips.0 { -1 } else { 1 },
            if flips.1 { -1 } else { 1 },
            f(-c.x),
            f(-c.y)
        ));
    }
    if parts.is_empty() { String::new() } else { format!(" transform=\"{}\"", parts.join(" ")) }
}

fn stroke_of(s: &ShapeData) -> String {
    match s.stroke {
        Some(c) if c.a > 0 && s.stroke_width > 0.0 => {
            format!(" stroke=\"{}\" stroke-width=\"{}\"", hex(c), f(s.stroke_width))
        }
        _ => String::new(),
    }
}

const LOOKS: [&str; 7] = ["Shadow", "ThreeD", "Glow", "InnerShadow", "Outline", "Warp", "Gradient"];

/// A vector element for a plain shape or text; `None` when the object has effects or a form that is easier to keep as a picture.
fn object_element(object: &StoredObject, common: &str) -> Option<String> {
    if object.json.as_object().is_some_and(|m| LOOKS.iter().any(|k| m.get(*k).is_some_and(|v| !v.is_null()))) {
        return None;
    }
    let b = object.box_()?;
    let flips = object.flips();
    match object.kind() {
        "shape" => {
            let s = object.shape()?;
            if s.vertices.is_none() && s.kind == ShapeKind::Rectangle {
                let corner = if s.corner_radius > 0.0 {
                    format!(" rx=\"{}\"", f(s.corner_radius.min(b.width.min(b.height) / 2.0)))
                } else {
                    String::new()
                };
                return Some(format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"{corner}{}{}{}{common}/>",
                    f(b.x),
                    f(b.y),
                    f(b.width),
                    f(b.height),
                    paint("fill", s.fill),
                    stroke_of(&s),
                    transform(b, flips)
                ));
            }
            if s.vertices.is_none() && s.kind == ShapeKind::Ellipse {
                let c = b.center();
                return Some(format!(
                    "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\"{}{}{}{common}/>",
                    f(c.x),
                    f(c.y),
                    f(b.width / 2.0),
                    f(b.height / 2.0),
                    paint("fill", s.fill),
                    stroke_of(&s),
                    transform(b, flips)
                ));
            }
            let vertices = s.vertices.clone().or_else(|| crate::render::geometry::vertices(&s, &b))?;
            if vertices.len() >= 3 && s.kind.is_polygonal() {
                let points: Vec<String> = vertices
                    .iter()
                    .map(|v| format!("{},{}", f(b.x + v.x * b.width), f(b.y + v.y * b.height)))
                    .collect();
                return Some(format!(
                    "<polygon points=\"{}\"{}{}{}{common}/>",
                    points.join(" "),
                    paint("fill", s.fill),
                    stroke_of(&s),
                    transform(b, flips)
                ));
            }
            None
        }
        "text" => {
            let t = object.text()?;
            if t.arc != 0.0 {
                return None; // curved text is kept as a picture
            }
            let text = t.text.replace('\r', "");
            let line_height = t.size * 1.25 * t.line_spacing;
            let (anchor, x) = match t.align {
                TextAlign::Center => ("middle", b.x + b.width / 2.0),
                TextAlign::Right => ("end", b.x + b.width),
                TextAlign::Left => ("start", b.x),
            };
            let mut s = format!(
                "<text font-family=\"{}\" font-size=\"{}\" text-anchor=\"{anchor}\"{}",
                escape(&t.family),
                f(t.size),
                paint("fill", Some(t.color))
            );
            if t.bold {
                s.push_str(" font-weight=\"bold\"");
            }
            if t.italic {
                s.push_str(" font-style=\"italic\"");
            }
            if let (true, Some(oc)) = (t.outline_width > 0.0, t.outline_color) {
                let _ = write!(
                    s,
                    " stroke=\"{}\" stroke-width=\"{}\" paint-order=\"stroke\" stroke-linejoin=\"round\"",
                    hex(oc),
                    f(t.outline_width * 2.0)
                );
            }
            let _ = write!(s, "{}{common}>", transform(b, flips));
            for (i, line) in text.split('\n').enumerate() {
                let _ = write!(
                    s,
                    "<tspan x=\"{}\" y=\"{}\">{}</tspan>",
                    f(x),
                    f(b.y + t.size * 0.95 + i as f64 * line_height),
                    escape(line)
                );
            }
            s.push_str("</text>");
            Some(s)
        }
        "image" => {
            let source = pictures::decoded(object.picture()?)?;
            let png = super::encode::encode_png(&source).ok()?;
            Some(format!(
                "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\" xlink:href=\"data:image/png;base64,{}\"{}{common}/>",
                f(b.x),
                f(b.y),
                f(b.width),
                f(b.height),
                base64_encode(&png),
                transform(b, flips)
            ))
        }
        _ => None,
    }
}

/// Any layer as a picture (cropped to what is visible, with its mask applied).
fn picture_element(layer: &Layer, canvas: crate::imaging::PixelRect, common: &str) -> String {
    let baked = match layer.mask() {
        Some(m) => bake_mask(layer.pixels(), m),
        None => layer.pixels().clone(),
    };
    let pixels = baked.to_buffer(canvas);
    let bounds = pixels.visible_bounds();
    if bounds.is_empty() {
        return format!("<g{common}/>");
    }
    let png = super::encode::encode_png(&pixels.crop(bounds)).unwrap_or_default();
    format!(
        "<image x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" xlink:href=\"data:image/png;base64,{}\"{common}/>",
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        base64_encode(&png)
    )
}

/// The document as SVG text.
pub fn export(document: &Document) -> String {
    let (w, h) = (document.width(), document.height());
    let canvas = document.bounds();
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n"
    );
    for layer in document.layers() {
        if !layer.visible || layer.is_adjustment() || layer.opacity() <= 0.0 {
            continue;
        }
        let common = common(layer);
        let element =
            if layer.mask().is_none() { layer.object().and_then(|o| object_element(o, &common)) } else { None };
        out.push_str(&element.unwrap_or_else(|| picture_element(layer, canvas, &common)));
        out.push('\n');
    }
    out.push_str("</svg>\n");
    out
}

/// Saves the document as an .svg file.
pub fn save(document: &Document, path: impl AsRef<Path>) -> Result<(), IoError> {
    write_file(path.as_ref(), export(document).into_bytes())
}

// ------------------------------------------------------------------------------------------------- opening

#[derive(Clone, Copy)]
struct Frame {
    scale: f64,
    dx: f64,
    dy: f64,
}

impl Frame {
    fn x(self, v: f64) -> f64 {
        v * self.scale + self.dx
    }
    fn y(self, v: f64) -> f64 {
        v * self.scale + self.dy
    }
}

#[derive(Clone, Copy)]
struct Style {
    fill: Option<ColorRgba>,
    stroke: Option<ColorRgba>,
    stroke_width: f64,
    opacity: f64,
}

fn number(s: &str) -> f64 {
    s.trim().parse().unwrap_or(0.0)
}

fn numbers(s: &str) -> Vec<f64> {
    s.split([' ', ',', '\t', '\n', '\r']).filter(|p| !p.is_empty()).map(number).collect()
}

/// A length such as "120", "120px" or "12.5pt" in pixels (percent and other units count as plain numbers).
fn length(value: Option<&str>, fallback: f64) -> f64 {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else { return fallback };
    let (text, factor) = if let Some(v) = value.strip_suffix("pt") {
        (v, 96.0 / 72.0)
    } else if let Some(v) = value.strip_suffix("px") {
        (v, 1.0)
    } else if let Some(v) = value.strip_suffix("mm") {
        (v, 96.0 / 25.4)
    } else if let Some(v) = value.strip_suffix("cm") {
        (v, 96.0 / 2.54)
    } else if let Some(v) = value.strip_suffix("in") {
        (v, 96.0)
    } else if value.ends_with('%') {
        return fallback;
    } else {
        (value, 1.0)
    };
    text.trim().parse::<f64>().map_or(fallback, |v| v * factor)
}

type Node<'a, 'b> = roxmltree::Node<'a, 'b>;

fn attr_length(e: Node, name: &str) -> f64 {
    length(e.attribute(name), 0.0)
}

fn style_attribute(e: Node, name: &str) -> Option<String> {
    let style = e.attribute("style")?;
    style.split(';').find_map(|d| d.split_once(':').filter(|(k, _)| k.trim() == name).map(|(_, v)| v.trim().to_owned()))
}

fn get(e: Node, name: &str) -> Option<String> {
    e.attribute(name).map(str::to_owned).or_else(|| style_attribute(e, name))
}

/// A color as written in SVG, or `None` for "none" (and for things that cannot be shown, like gradients).
fn color(value: &str) -> Option<ColorRgba> {
    let value = value.trim();
    if value == "none" || value == "transparent" || value.starts_with("url(") {
        return None;
    }
    if let Some(h) = value.strip_prefix('#') {
        let full: String = if h.len() == 3 { h.chars().flat_map(|c| [c, c]).collect() } else { h.to_owned() };
        if full.len() == 6
            && let Ok(rgb) = u32::from_str_radix(&full, 16)
        {
            return Some(ColorRgba::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8));
        }
        return None;
    }
    if value.len() > 3 && value[..3].eq_ignore_ascii_case("rgb") {
        let inner = value.split_once('(')?.1.trim_end_matches(')');
        let n: Vec<f64> =
            inner.split([',', ' ', '/']).filter(|s| !s.is_empty()).map(|s| number(s.trim_end_matches('%'))).collect();
        if n.len() >= 3 {
            return Some(ColorRgba::rgb(
                n[0].clamp(0.0, 255.0) as u8,
                n[1].clamp(0.0, 255.0) as u8,
                n[2].clamp(0.0, 255.0) as u8,
            ));
        }
        return None;
    }
    let named: [(&str, (u8, u8, u8)); 19] = [
        ("black", (0, 0, 0)),
        ("white", (255, 255, 255)),
        ("red", (255, 0, 0)),
        ("green", (0, 128, 0)),
        ("blue", (0, 0, 255)),
        ("yellow", (255, 255, 0)),
        ("orange", (255, 165, 0)),
        ("purple", (128, 0, 128)),
        ("gray", (128, 128, 128)),
        ("grey", (128, 128, 128)),
        ("pink", (255, 192, 203)),
        ("brown", (165, 42, 42)),
        ("cyan", (0, 255, 255)),
        ("magenta", (255, 0, 255)),
        ("lime", (0, 255, 0)),
        ("navy", (0, 0, 128)),
        ("teal", (0, 128, 128)),
        ("silver", (192, 192, 192)),
        ("gold", (255, 215, 0)),
    ];
    named.iter().find(|(n, _)| n.eq_ignore_ascii_case(value)).map(|(_, (r, g, b))| ColorRgba::rgb(*r, *g, *b))
}

fn style_of(e: Node, inherited: Style) -> Style {
    let mut fill = inherited.fill;
    let mut stroke = inherited.stroke;
    let mut opacity = inherited.opacity;
    if let Some(o) = get(e, "opacity") {
        opacity *= number(&o).clamp(0.0, 1.0);
    }
    let fill_opacity = get(e, "fill-opacity").map_or(1.0, |v| number(&v).clamp(0.0, 1.0));
    let stroke_opacity = get(e, "stroke-opacity").map_or(1.0, |v| number(&v).clamp(0.0, 1.0));
    if let Some(v) = get(e, "fill") {
        fill = color(&v);
    }
    if let Some(v) = get(e, "stroke") {
        stroke = color(&v);
    }
    let stroke_width = get(e, "stroke-width").map_or(inherited.stroke_width, |w| number(&w).max(0.0));
    if let Some(c) = &mut fill {
        c.a = (f64::from(c.a) * opacity * fill_opacity).round_ties_even() as u8;
    }
    if let Some(c) = &mut stroke {
        c.a = (f64::from(c.a) * opacity * stroke_opacity).round_ties_even() as u8;
    }
    Style { fill, stroke, stroke_width, opacity: inherited.opacity }
}

fn group_frame(g: Node, frame: Frame) -> Frame {
    let Some(t) = g.attribute("transform") else { return frame };
    let (mut dx, mut dy, mut scale) = (0.0, 0.0, 1.0);
    for part in t.split(')').filter(|p| !p.trim().is_empty()) {
        let Some((kind, args)) = part.split_once('(') else { continue };
        let args = numbers(args);
        match kind.trim() {
            "translate" if !args.is_empty() => {
                dx += args[0];
                dy += args.get(1).copied().unwrap_or(0.0);
            }
            "scale" if !args.is_empty() => scale *= args[0],
            _ => {}
        }
    }
    Frame { scale: frame.scale * scale, dx: frame.dx + dx * frame.scale, dy: frame.dy + dy * frame.scale }
}

fn bezier(from: (f64, f64), c1: (f64, f64), c2: (f64, f64), to: (f64, f64), add: &mut dyn FnMut(f64, f64)) {
    for s in 1..=16 {
        let t = f64::from(s) / 16.0;
        let u = 1.0 - t;
        add(
            u * u * u * from.0 + 3.0 * u * u * t * c1.0 + 3.0 * u * t * t * c2.0 + t * t * t * to.0,
            u * u * u * from.1 + 3.0 * u * u * t * c1.1 + 3.0 * u * t * t * c2.1 + t * t * t * to.1,
        );
    }
}

/// Lines and curves of a path, flattened into points.
fn flatten_path(d: &str, frame: Frame) -> Vec<(Vec<PointD>, bool)> {
    let mut tokens: Vec<String> = Vec::new();
    let mut num = String::new();
    let flush = |num: &mut String, tokens: &mut Vec<String>| {
        if !num.is_empty() {
            tokens.push(std::mem::take(num));
        }
    };
    for c in d.chars() {
        if c.is_alphabetic() && c != 'e' && c != 'E' {
            flush(&mut num, &mut tokens);
            tokens.push(c.to_string());
        } else if matches!(c, ' ' | ',' | '\t' | '\n' | '\r') {
            flush(&mut num, &mut tokens);
        } else if (c == '-' && !num.is_empty() && !num.ends_with(['e', 'E'])) || (c == '.' && num.contains('.')) {
            flush(&mut num, &mut tokens);
            num.push(c);
        } else {
            num.push(c);
        }
    }
    flush(&mut num, &mut tokens);
    let is_command = |t: &str| t.len() == 1 && t.chars().all(char::is_alphabetic);
    let mut current: Vec<PointD> = Vec::new();
    let (mut x, mut y, mut sx, mut sy, mut lcx, mut lcy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut command = 'M';
    let mut i = 0usize;
    let mut finished: Vec<(Vec<PointD>, bool)> = Vec::new();
    let next = |i: &mut usize| -> f64 {
        let v = tokens.get(*i).map_or(0.0, |t| number(t));
        *i += 1;
        v
    };
    while i < tokens.len() {
        if is_command(&tokens[i]) {
            command = tokens[i].chars().next().unwrap_or('M');
            i += 1;
        } else if command == 'M' {
            command = 'L';
        } else if command == 'm' {
            command = 'l';
        }
        let rel = command.is_lowercase();
        let (ox, oy) = if rel { (x, y) } else { (0.0, 0.0) };
        let add = |px: f64, py: f64, cur: &mut Vec<PointD>| cur.push(PointD::new(frame.x(px), frame.y(py)));
        match command.to_ascii_uppercase() {
            'M' => {
                if current.len() > 1 {
                    finished.push((std::mem::take(&mut current), false));
                }
                current = Vec::new();
                x = next(&mut i) + ox;
                y = next(&mut i) + oy;
                sx = x;
                sy = y;
                add(x, y, &mut current);
            }
            'L' => {
                x = next(&mut i) + ox;
                y = next(&mut i) + oy;
                add(x, y, &mut current);
            }
            'H' => {
                x = next(&mut i) + ox;
                add(x, y, &mut current);
            }
            'V' => {
                y = next(&mut i) + oy;
                add(x, y, &mut current);
            }
            'C' => {
                let c1 = (next(&mut i) + ox, next(&mut i) + oy);
                let c2 = (next(&mut i) + ox, next(&mut i) + oy);
                let to = (next(&mut i) + ox, next(&mut i) + oy);
                bezier((x, y), c1, c2, to, &mut |px, py| add(px, py, &mut current));
                (lcx, lcy) = c2;
                (x, y) = to;
            }
            'S' => {
                let c2 = (next(&mut i) + ox, next(&mut i) + oy);
                let to = (next(&mut i) + ox, next(&mut i) + oy);
                bezier((x, y), (2.0 * x - lcx, 2.0 * y - lcy), c2, to, &mut |px, py| add(px, py, &mut current));
                (lcx, lcy) = c2;
                (x, y) = to;
            }
            'Q' => {
                let c1 = (next(&mut i) + ox, next(&mut i) + oy);
                let to = (next(&mut i) + ox, next(&mut i) + oy);
                let a = (x + 2.0 / 3.0 * (c1.0 - x), y + 2.0 / 3.0 * (c1.1 - y));
                let b = (to.0 + 2.0 / 3.0 * (c1.0 - to.0), to.1 + 2.0 / 3.0 * (c1.1 - to.1));
                bezier((x, y), a, b, to, &mut |px, py| add(px, py, &mut current));
                (lcx, lcy) = c1;
                (x, y) = to;
            }
            'A' => {
                for _ in 0..5 {
                    next(&mut i); // radii, rotation, flags: arcs are drawn as straight lines to their end
                }
                x = next(&mut i) + ox;
                y = next(&mut i) + oy;
                add(x, y, &mut current);
            }
            'Z' => {
                if current.len() > 1 {
                    finished.push((std::mem::take(&mut current), true));
                }
                current = Vec::new();
                x = sx;
                y = sy;
                command = 'M';
            }
            _ => i += 1,
        }
    }
    if current.len() > 1 {
        finished.push((current, false));
    }
    finished
}

fn add_polygon(
    result: &mut Vec<(String, StoredObject)>,
    name: &str,
    points: &[PointD],
    style: Style,
    frame: Frame,
    closed: bool,
) {
    if points.len() < 2 {
        return;
    }
    let min_x = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let min_y = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let max_y = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let (w, h) = ((max_x - min_x).max(1.0), (max_y - min_y).max(1.0));
    let unit: Vec<PointD> = points.iter().map(|p| PointD::new((p.x - min_x) / w, (p.y - min_y) / h)).collect();
    let fill = if closed { style.fill } else { None };
    let stroke = style.stroke.or(if closed { None } else { Some(style.fill.unwrap_or(ColorRgba::BLACK)) });
    let mut data = ShapeData::new(ShapeKind::Polygon, fill, stroke, style.stroke_width * frame.scale);
    data.vertices = Some(unit);
    result.push((name.to_owned(), StoredObject::new_shape(ObjectBox::new(min_x, min_y, w, h, 0.0), &data)));
}

fn walk(parent: Node, frame: Frame, inherited: Style, result: &mut Vec<(String, StoredObject)>) {
    for e in parent.children().filter(Node::is_element) {
        let style = style_of(e, inherited);
        let name = e.attribute("id").unwrap_or("").to_owned();
        match e.tag_name().name() {
            "g" | "svg" => walk(e, group_frame(e, frame), style, result),
            "rect" => {
                let (x, y) = (frame.x(attr_length(e, "x")), frame.y(attr_length(e, "y")));
                let (w, h) = (attr_length(e, "width") * frame.scale, attr_length(e, "height") * frame.scale);
                if w <= 0.0 || h <= 0.0 {
                    continue;
                }
                let radius = attr_length(e, "rx").max(attr_length(e, "ry")) * frame.scale;
                let mut data =
                    ShapeData::new(ShapeKind::Rectangle, style.fill, style.stroke, style.stroke_width * frame.scale);
                data.corner_radius = radius;
                result.push((name, StoredObject::new_shape(ObjectBox::new(x, y, w, h, 0.0), &data)));
            }
            tag @ ("circle" | "ellipse") => {
                let (rx, ry) = if tag == "circle" {
                    (attr_length(e, "r") * frame.scale, attr_length(e, "r") * frame.scale)
                } else {
                    (attr_length(e, "rx") * frame.scale, attr_length(e, "ry") * frame.scale)
                };
                if rx <= 0.0 || ry <= 0.0 {
                    continue;
                }
                let (cx, cy) = (frame.x(attr_length(e, "cx")), frame.y(attr_length(e, "cy")));
                let data =
                    ShapeData::new(ShapeKind::Ellipse, style.fill, style.stroke, style.stroke_width * frame.scale);
                result.push((
                    name,
                    StoredObject::new_shape(ObjectBox::new(cx - rx, cy - ry, rx * 2.0, ry * 2.0, 0.0), &data),
                ));
            }
            "line" => {
                let points = [
                    PointD::new(frame.x(attr_length(e, "x1")), frame.y(attr_length(e, "y1"))),
                    PointD::new(frame.x(attr_length(e, "x2")), frame.y(attr_length(e, "y2"))),
                ];
                add_polygon(result, &name, &points, Style { fill: None, ..style }, frame, false);
            }
            tag @ ("polygon" | "polyline") => {
                let n = numbers(e.attribute("points").unwrap_or(""));
                let points: Vec<PointD> =
                    n.chunks_exact(2).map(|p| PointD::new(frame.x(p[0]), frame.y(p[1]))).collect();
                let closed = tag == "polygon";
                add_polygon(
                    result,
                    &name,
                    &points,
                    if closed { style } else { Style { fill: None, ..style } },
                    frame,
                    closed,
                );
            }
            "path" => {
                for (points, closed) in flatten_path(e.attribute("d").unwrap_or(""), frame) {
                    add_polygon(
                        result,
                        &name,
                        &points,
                        if closed { style } else { Style { fill: None, ..style } },
                        frame,
                        closed,
                    );
                }
            }
            "text" => {
                let text: String = e
                    .children()
                    .map(|n| {
                        if n.is_text() {
                            n.text().unwrap_or("").to_owned()
                        } else {
                            n.text().map_or_else(|| n.descendants().filter_map(|d| d.text()).collect(), str::to_owned)
                        }
                    })
                    .collect::<String>()
                    .trim()
                    .to_owned();
                if text.is_empty() {
                    continue;
                }
                let size = (length(get(e, "font-size").as_deref(), 16.0) * frame.scale).max(1.0);
                let family = get(e, "font-family").unwrap_or_else(|| "Segoe UI".into());
                let family = family.split(',').next().unwrap_or("Segoe UI").trim().trim_matches(['\'', '"']).to_owned();
                let anchor = e.attribute("text-anchor").unwrap_or("start");
                let estimated = size.max(text.chars().count() as f64 * size * 0.55);
                let height = size * 1.35;
                let (x, y) = (frame.x(attr_length(e, "x")), frame.y(attr_length(e, "y")) - size * 0.95);
                let left = match anchor {
                    "middle" => x - estimated / 2.0,
                    "end" => x - estimated,
                    _ => x,
                };
                let mut data = TextData::new(&text, &family, size, style.fill.unwrap_or(ColorRgba::BLACK));
                data.align = match anchor {
                    "middle" => TextAlign::Center,
                    "end" => TextAlign::Right,
                    _ => TextAlign::Left,
                };
                data.bold = matches!(get(e, "font-weight").as_deref(), Some("bold" | "700" | "800" | "900"));
                data.italic = get(e, "font-style").as_deref() == Some("italic");
                result.push((name, StoredObject::new_text(ObjectBox::new(left, y, estimated, height, 0.0), &data)));
            }
            "image" => {
                let href = e.attributes().find(|a| a.name() == "href").map_or("", |a| a.value());
                let Some((head, data)) = href.split_once(',') else { continue };
                if !href.to_ascii_lowercase().starts_with("data:image") || !head.contains("base64") {
                    continue;
                }
                let Some(bytes) = base64_decode(data) else { continue };
                let extension = if head.contains("jpeg") || head.contains("jpg") { "jpg" } else { "png" };
                let Ok(picture) = decode::load_bytes(&bytes, Some(extension)) else { continue }; // a picture that cannot be read is skipped
                let mut w = attr_length(e, "width") * frame.scale;
                let mut h = attr_length(e, "height") * frame.scale;
                if w <= 0.0 {
                    w = f64::from(picture.width()) * frame.scale;
                }
                if h <= 0.0 {
                    h = f64::from(picture.height()) * frame.scale;
                }
                let label = if name.is_empty() { "Image".to_owned() } else { name.clone() };
                let object = StoredObject::new_image(
                    ObjectBox::new(frame.x(attr_length(e, "x")), frame.y(attr_length(e, "y")), w, h, 0.0),
                    picture_bytes(&picture),
                    &label,
                );
                result.push((name, object));
            }
            _ => {}
        }
    }
}

/// The elements of a picture as named editable objects (bottom to top).
type Objects = Vec<(String, StoredObject)>;

/// The size of the picture and its elements as editable objects (bottom to top), without drawing them.
pub fn parse(svg: &str) -> Result<(u32, u32, Objects), IoError> {
    let not_svg = || IoError::CannotOpen("This is not an SVG picture.".into());
    let options = roxmltree::ParsingOptions { allow_dtd: true, ..roxmltree::ParsingOptions::default() };
    let doc = roxmltree::Document::parse_with_options(svg, options).map_err(|_| not_svg())?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err(not_svg());
    }
    let (mut vb_x, mut vb_y, mut vb_w, mut vb_h) = (0.0, 0.0, 0.0, 0.0);
    if let Some(vb) = root.attribute("viewBox") {
        let parts = numbers(vb);
        if parts.len() == 4 {
            (vb_x, vb_y, vb_w, vb_h) = (parts[0], parts[1], parts[2], parts[3]);
        }
    }
    let mut w = length(root.attribute("width"), 0.0);
    let mut h = length(root.attribute("height"), 0.0);
    if w <= 0.0 {
        w = if vb_w > 0.0 { vb_w } else { 800.0 };
    }
    if h <= 0.0 {
        h = if vb_h > 0.0 { vb_h } else { 600.0 };
    }
    let scale = if vb_w > 0.0 { w / vb_w } else { 1.0 };
    let width = w.round_ties_even().clamp(1.0, 16000.0) as u32;
    let height = h.round_ties_even().clamp(1.0, 16000.0) as u32;
    let mut result = Vec::new();
    let inherit = Style { fill: Some(ColorRgba::BLACK), stroke: None, stroke_width: 1.0, opacity: 1.0 };
    walk(root, Frame { scale, dx: -vb_x * scale, dy: -vb_y * scale }, inherit, &mut result);
    Ok((width, height, result))
}

/// Opens SVG text as a document: every element becomes a layer holding an editable object.
pub fn import(svg: &str) -> Result<Document, IoError> {
    let (width, height, objects) = parse(svg)?;
    let mut document = Document::new(width, height);
    let canvas = document.bounds();
    let mut layers = vec![Layer::new("Background")];
    let mut number = 0;
    for (name, object) in objects {
        let name = if name.is_empty() {
            number += 1;
            let kind = match object.kind() {
                "shape" => "Shape",
                "text" => "Text",
                "image" => "Image",
                other => other,
            };
            format!("{kind} {number}")
        } else {
            name
        };
        layers.push(object_ops::object_layer(name, object, canvas));
    }
    let last = layers.last().map(Layer::id);
    document.set_layers(layers, last);
    Ok(document)
}

/// Opens an .svg file.
pub fn open(path: impl AsRef<Path>) -> Result<Document, IoError> {
    import(&std::fs::read_to_string(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips() {
        for text in ["", "a", "ab", "abc", "abcd", "Flash Editor \u{e9}"] {
            assert_eq!(base64_decode(&base64_encode(text.as_bytes())).as_deref(), Some(text.as_bytes()));
        }
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn shapes_and_text_open_as_editable_objects() {
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="100" viewBox="0 0 100 50">
            <rect id="Box" x="10" y="5" width="30" height="20" rx="4" fill="#ff0000" stroke="blue" stroke-width="2"/>
            <circle cx="70" cy="25" r="10" fill="rgb(0,128,255)" opacity="0.5"/>
            <polygon points="0,0 10,0 5,10" fill="green"/>
            <path d="M 0 40 L 20 40 L 20 48 Z" fill="black"/>
            <text x="50" y="45" font-size="8" font-family="Arial, sans-serif" text-anchor="middle" font-weight="bold">Hi</text></svg>"##;
        let (w, h, objects) = parse(svg).expect("parsed");
        assert_eq!((w, h), (200, 100), "the viewBox is scaled to the width and height");
        assert_eq!(objects.len(), 5);
        let (name, rect) = &objects[0];
        assert_eq!(name, "Box");
        let b = rect.box_().expect("box");
        assert_eq!((b.x, b.y, b.width, b.height), (20.0, 10.0, 60.0, 40.0));
        let shape = rect.shape().expect("shape");
        assert_eq!(shape.kind, ShapeKind::Rectangle);
        assert_eq!(shape.corner_radius, 8.0);
        assert_eq!(shape.fill, Some(ColorRgba::rgb(255, 0, 0)));
        assert_eq!(objects[1].1.shape().expect("circle").fill.map(|c| c.a), Some(128));
        assert_eq!(objects[2].1.shape().expect("polygon").vertices.expect("corners").len(), 3);
        let text = objects[4].1.text().expect("text");
        assert_eq!(
            (text.text.as_str(), text.family.as_str(), text.bold, text.align),
            ("Hi", "Arial", true, TextAlign::Center)
        );
    }

    #[test]
    fn a_document_survives_an_export_and_an_import() {
        let mut doc = Document::blank(120, 80, ColorRgba::WHITE);
        let canvas = doc.bounds();
        let data =
            ShapeData::new(ShapeKind::Ellipse, Some(ColorRgba::rgb(10, 200, 30)), Some(ColorRgba::rgb(0, 0, 0)), 3.0);
        let mut layer = object_ops::object_layer(
            "Dot",
            StoredObject::new_shape(ObjectBox::new(20.0, 10.0, 40.0, 30.0, 0.0), &data),
            canvas,
        );
        layer.set_opacity(0.8);
        doc.push_layer(layer);
        let text = TextData::new("A & B", "Segoe UI", 20.0, ColorRgba::rgb(255, 0, 0));
        doc.push_layer(object_ops::object_layer(
            "Words",
            StoredObject::new_text(ObjectBox::new(10.0, 50.0, 100.0, 25.0, 0.0), &text),
            canvas,
        ));
        let svg = export(&doc);
        assert!(svg.contains("<ellipse") && svg.contains("opacity=\"0.8\"") && svg.contains("A &amp; B"));
        let back = import(&svg).expect("opens again");
        assert_eq!((back.width(), back.height()), (120, 80));
        let kinds: Vec<&str> = back.layers().iter().filter_map(|l| l.object()).map(StoredObject::kind).collect();
        assert!(kinds.contains(&"shape") && kinds.contains(&"text"));
    }

    #[test]
    fn something_that_is_not_svg_is_refused() {
        assert!(parse("<html/>").is_err());
        assert!(parse("not xml at all").is_err());
        assert!(is_svg("a.SVG") && !is_svg("a.png"));
    }
}
