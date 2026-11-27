//! Opening PowerPoint files (.pptx): every slide becomes a page. Text boxes become text objects, PowerPoint shapes
//! become shapes, lines and arrows become lines, pictures become placed pictures and tables become cells of shapes and
//! text, all at the place they have on the slide. Backgrounds, theme colors and the text styles of the slide master
//! are followed. Charts, SmartArt, videos and fancy effects cannot be brought in and are counted so the user is told.
//! Text that is formatted in several ways inside one box is split into one text object for every run of equal looks.

use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;

use roxmltree::Node;

use super::IoError;
use crate::document::{Document, Page, object_ops};
use crate::imaging::{ColorRgba, PixelRect, PointD};
use crate::io::decode;
use crate::objects::{ObjectBox, ShapeData, ShapeKind, StoredObject, StrokeStyle, picture_bytes};
use crate::text::{TextAlign, TextData, lay_out};

const EMU_PER_PIXEL: f64 = 9525.0;
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
/// The longest side a page may get, in pixels.
const MAX_SIDE: f64 = 6000.0;

/// What was read from a .pptx file.
pub struct Imported {
    pub pages: Vec<Page>,
    /// How many things (charts, SmartArt, videos, odd pictures...) could not be brought in.
    pub skipped: usize,
}

/// True for a path that ends in .pptx.
pub fn is_pptx(path: impl AsRef<Path>) -> bool {
    path.as_ref().extension().is_some_and(|e| e.eq_ignore_ascii_case("pptx"))
}

/// Opens a .pptx file.
pub fn open_pptx(path: impl AsRef<Path>) -> Result<Imported, IoError> {
    read_pptx(std::fs::read(path)?)
}

fn not_pptx() -> IoError {
    IoError::CannotOpen("This is not a PowerPoint (.pptx) file, or it is damaged.".into())
}

// ---------------------------------------------------------------- the zip

struct Package {
    zip: zip::ZipArchive<Cursor<Vec<u8>>>,
}

type Rels = HashMap<String, String>;

impl Package {
    fn bytes(&mut self, name: &str) -> Option<Vec<u8>> {
        let name = name.trim_start_matches('/');
        let mut file = self.zip.by_name(name).ok()?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok()?;
        Some(data)
    }

    fn text(&mut self, name: &str) -> Option<String> {
        String::from_utf8(self.bytes(name)?).ok()
    }

    /// The relationships of a part: id -> the full name of the part it points to.
    fn rels(&mut self, part: &str) -> Rels {
        let (dir, file) = part.rsplit_once('/').unwrap_or(("", part));
        let Some(xml) = self.text(&format!("{dir}/_rels/{file}.rels")) else { return Rels::new() };
        let Ok(doc) = roxmltree::Document::parse(&xml) else { return Rels::new() };
        doc.descendants()
            .filter(|n| n.tag_name().name() == "Relationship")
            .filter(|n| n.attribute("TargetMode") != Some("External"))
            .filter_map(|n| Some((n.attribute("Id")?.to_owned(), resolve(dir, n.attribute("Target")?))))
            .collect()
    }
}

/// `target` (as written in a .rels file) relative to the folder `dir`.
fn resolve(dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = if target.starts_with('/') { Vec::new() } else { dir.split('/').collect() };
    for piece in target.split('/') {
        match piece {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.retain(|p| !p.is_empty());
    parts.join("/")
}

// ---------------------------------------------------------------- xml helpers

type N<'a> = Node<'a, 'a>;

fn child<'a>(n: N<'a>, name: &str) -> Option<N<'a>> {
    n.children().find(|c| c.is_element() && c.tag_name().name() == name)
}

fn kids<'a>(n: N<'a>, name: &'a str) -> impl Iterator<Item = N<'a>> {
    n.children().filter(move |c| c.is_element() && c.tag_name().name() == name)
}

fn num(n: N<'_>, name: &str) -> Option<f64> {
    n.attribute(name)?.trim().parse().ok()
}

fn rid<'a>(n: N<'a>, name: &str) -> Option<&'a str> {
    n.attribute((NS_R, name))
}

/// `val` of a child such as `<a:alpha val="50000"/>`, as a fraction.
fn percent_of(n: N<'_>, name: &str) -> Option<f64> {
    num(child(n, name)?, "val").map(|v| v / 100_000.0)
}

// ---------------------------------------------------------------- the theme and colors

#[derive(Default)]
struct Theme {
    colors: HashMap<String, ColorRgba>,
    map: HashMap<String, String>,
    major: String,
    minor: String,
}

impl Theme {
    fn read(xml: Option<&str>, master: Option<N<'_>>) -> Self {
        let mut theme = Self { major: "Calibri Light".into(), minor: "Calibri".into(), ..Self::default() };
        for (key, value) in [
            ("dk1", (0, 0, 0)),
            ("lt1", (255, 255, 255)),
            ("dk2", (68, 84, 106)),
            ("lt2", (231, 230, 230)),
            ("accent1", (68, 114, 196)),
            ("accent2", (237, 125, 49)),
            ("accent3", (165, 165, 165)),
            ("accent4", (255, 192, 0)),
            ("accent5", (91, 155, 213)),
            ("accent6", (112, 173, 71)),
            ("hlink", (5, 99, 193)),
            ("folHlink", (149, 79, 114)),
        ] {
            theme.colors.insert(key.into(), ColorRgba::rgb(value.0, value.1, value.2));
        }
        if let Some(doc) = xml.and_then(|x| roxmltree::Document::parse(x).ok()) {
            for n in doc.descendants().filter(|n| n.is_element()) {
                match n.tag_name().name() {
                    "clrScheme" => {
                        for slot in n.children().filter(Node::is_element) {
                            let c = slot.children().find(Node::is_element).and_then(|c| match c.tag_name().name() {
                                "srgbClr" => c.attribute("val").and_then(hex),
                                "sysClr" => c.attribute("lastClr").and_then(hex),
                                _ => None,
                            });
                            if let Some(c) = c {
                                theme.colors.insert(slot.tag_name().name().to_owned(), c);
                            }
                        }
                    }
                    "majorFont" => {
                        if let Some(f) =
                            child(n, "latin").and_then(|l| l.attribute("typeface")).filter(|f| !f.is_empty())
                        {
                            theme.major = f.to_owned();
                        }
                    }
                    "minorFont" => {
                        if let Some(f) =
                            child(n, "latin").and_then(|l| l.attribute("typeface")).filter(|f| !f.is_empty())
                        {
                            theme.minor = f.to_owned();
                        }
                    }
                    _ => {}
                }
            }
        }
        for (from, to) in [("bg1", "lt1"), ("tx1", "dk1"), ("bg2", "lt2"), ("tx2", "dk2")] {
            theme.map.insert(from.into(), to.into());
        }
        if let Some(m) = master.and_then(|m| m.descendants().find(|n| n.tag_name().name() == "clrMap")) {
            for a in m.attributes() {
                theme.map.insert(a.name().to_owned(), a.value().to_owned());
            }
        }
        theme
    }

    fn scheme(&self, name: &str) -> Option<ColorRgba> {
        let name = self.map.get(name).map_or(name, String::as_str);
        self.colors.get(name).copied()
    }

    fn font(&self, name: &str) -> String {
        match name {
            "+mj-lt" => self.major.clone(),
            "+mn-lt" | "" => self.minor.clone(),
            other => other.to_owned(),
        }
    }
}

fn hex(v: &str) -> Option<ColorRgba> {
    if v.len() != 6 {
        return None;
    }
    let p = |i: usize| u8::from_str_radix(&v[i..i + 2], 16).ok();
    Some(ColorRgba::rgb(p(0)?, p(2)?, p(4)?))
}

fn rgb_to_hsl(c: ColorRgba) -> (f64, f64, f64) {
    let (r, g, b) = (f64::from(c.r) / 255.0, f64::from(c.g) / 255.0, f64::from(c.b) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-9 {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - r).abs() < 1e-9 {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < 1e-9 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h / 6.0, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64, a: u8) -> ColorRgba {
    let channel = |t: f64| {
        let t = t.rem_euclid(1.0);
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let v = if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        };
        (v.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    if s <= 0.0 {
        let v = (l.clamp(0.0, 1.0) * 255.0).round() as u8;
        return ColorRgba::new(v, v, v, a);
    }
    ColorRgba::new(channel(h + 1.0 / 3.0), channel(h), channel(h - 1.0 / 3.0), a)
}

/// The color written inside `holder` (`solidFill`, `fillRef`, `ln`...): the first color element, with its changes.
fn color_in(holder: N<'_>, theme: &Theme, ph: Option<ColorRgba>) -> Option<ColorRgba> {
    let c = holder
        .children()
        .find(|c| c.is_element() && matches!(c.tag_name().name(), "srgbClr" | "schemeClr" | "sysClr" | "prstClr"))?;
    let mut color = match c.tag_name().name() {
        "srgbClr" => c.attribute("val").and_then(hex)?,
        "sysClr" => c.attribute("lastClr").and_then(hex).unwrap_or(ColorRgba::BLACK),
        "prstClr" => match c.attribute("val").unwrap_or("black") {
            "white" => ColorRgba::WHITE,
            "red" => ColorRgba::rgb(255, 0, 0),
            "green" => ColorRgba::rgb(0, 128, 0),
            "blue" => ColorRgba::rgb(0, 0, 255),
            "yellow" => ColorRgba::rgb(255, 255, 0),
            "gray" | "grey" => ColorRgba::rgb(128, 128, 128),
            _ => ColorRgba::BLACK,
        },
        _ => {
            let name = c.attribute("val")?;
            if name == "phClr" { ph? } else { theme.scheme(name)? }
        }
    };
    let alpha = percent_of(c, "alpha");
    if let Some(m) = percent_of(c, "lumMod") {
        let (h, s, l) = rgb_to_hsl(color);
        color = hsl_to_rgb(h, s, (l * m).clamp(0.0, 1.0), color.a);
    }
    if let Some(o) = percent_of(c, "lumOff") {
        let (h, s, l) = rgb_to_hsl(color);
        color = hsl_to_rgb(h, s, (l + o).clamp(0.0, 1.0), color.a);
    }
    if let Some(t) = percent_of(c, "tint") {
        let mix = |v: u8| (f64::from(v) * t + 255.0 * (1.0 - t)).round().clamp(0.0, 255.0) as u8;
        color = ColorRgba::new(mix(color.r), mix(color.g), mix(color.b), color.a);
    }
    if let Some(s) = percent_of(c, "shade") {
        let mix = |v: u8| (f64::from(v) * s).round().clamp(0.0, 255.0) as u8;
        color = ColorRgba::new(mix(color.r), mix(color.g), mix(color.b), color.a);
    }
    if let Some(a) = alpha {
        color.a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
    }
    Some(color)
}

/// What a shape or background is filled with.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Fill {
    /// Nothing was said: ask the next place (layout, master, style).
    Inherit,
    None,
    Solid(ColorRgba),
    /// A picture or pattern: not drawn as a fill.
    Other,
}

fn fill_in(holder: N<'_>, theme: &Theme, ph: Option<ColorRgba>) -> Fill {
    for c in holder.children().filter(Node::is_element) {
        match c.tag_name().name() {
            "noFill" => return Fill::None,
            "solidFill" => return color_in(c, theme, ph).map_or(Fill::Other, Fill::Solid),
            "gradFill" => {
                let stops: Vec<ColorRgba> = child(c, "gsLst")
                    .map(|l| kids(l, "gs").filter_map(|g| color_in(g, theme, ph)).collect())
                    .unwrap_or_default();
                return stops.get(stops.len() / 2).copied().map_or(Fill::Other, Fill::Solid);
            }
            "pattFill" => {
                return child(c, "fgClr").and_then(|f| color_in(f, theme, ph)).map_or(Fill::Other, Fill::Solid);
            }
            "blipFill" => return Fill::Other,
            _ => {}
        }
    }
    Fill::Inherit
}

// ---------------------------------------------------------------- places

/// Maps the coordinates inside a group (EMU) to page pixels: `px = t + emu * s`.
#[derive(Clone, Copy)]
struct Frame {
    tx: f64,
    ty: f64,
    sx: f64,
    sy: f64,
}

struct Xfrm {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    rot: f64,
    flip_h: bool,
    flip_v: bool,
}

fn xfrm_in(holder: N<'_>) -> Option<Xfrm> {
    let x = child(holder, "xfrm")?;
    let off = child(x, "off")?;
    let ext = child(x, "ext")?;
    Some(Xfrm {
        x: num(off, "x")?,
        y: num(off, "y")?,
        w: num(ext, "cx")?,
        h: num(ext, "cy")?,
        rot: num(x, "rot").unwrap_or(0.0) / 60_000.0,
        flip_h: x.attribute("flipH") == Some("1"),
        flip_v: x.attribute("flipV") == Some("1"),
    })
}

/// A placed rectangle on the page, in pixels.
#[derive(Clone, Copy)]
struct Placed {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    rot: f64,
    flip_h: bool,
    flip_v: bool,
}

impl Placed {
    fn of(x: &Xfrm, f: Frame) -> Self {
        Self {
            x: f.tx + x.x * f.sx,
            y: f.ty + x.y * f.sy,
            w: x.w * f.sx,
            h: x.h * f.sy,
            rot: x.rot,
            flip_h: x.flip_h,
            flip_v: x.flip_v,
        }
    }

    fn center(&self) -> (f64, f64) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    fn object_box(&self) -> ObjectBox {
        ObjectBox::new(self.x, self.y, self.w.max(1.0), self.h.max(1.0), self.rot)
    }
}

// ---------------------------------------------------------------- shapes

fn shape_kind(prst: &str, adjust: Option<f64>, min_side: f64) -> Option<(ShapeKind, ShapeData)> {
    let blank = |kind| ShapeData::new(kind, None, None, 0.0);
    let simple = |kind| Some((kind, blank(kind)));
    let star = |points: i32| {
        let mut d = blank(ShapeKind::Star);
        d.points = points;
        Some((ShapeKind::Star, d))
    };
    let polygon = |points: i32| {
        let mut d = blank(ShapeKind::Polygon);
        d.points = points;
        Some((ShapeKind::Polygon, d))
    };
    match prst {
        "rect"
        | "flowChartProcess"
        | "flowChartAlternateProcess"
        | "snip1Rect"
        | "snip2SameRect"
        | "round1Rect"
        | "round2SameRect"
        | "textBox" => simple(ShapeKind::Rectangle),
        "roundRect" => {
            let mut d = blank(ShapeKind::Rectangle);
            d.corner_radius = adjust.unwrap_or(0.16667).clamp(0.0, 0.5) * min_side;
            Some((ShapeKind::Rectangle, d))
        }
        "ellipse" | "flowChartConnector" => simple(ShapeKind::Ellipse),
        "triangle" | "flowChartExtract" => simple(ShapeKind::Triangle),
        "rtTriangle" => simple(ShapeKind::RightTriangle),
        "diamond" | "flowChartDecision" => simple(ShapeKind::Diamond),
        "trapezoid" => simple(ShapeKind::Trapezoid),
        "parallelogram" | "flowChartData" => simple(ShapeKind::Parallelogram),
        "heart" => simple(ShapeKind::Heart),
        "rightArrow" | "homePlate" => simple(ShapeKind::Arrow),
        "leftRightArrow" => simple(ShapeKind::DoubleArrow),
        "chevron" => simple(ShapeKind::Chevron),
        "lightningBolt" => simple(ShapeKind::Lightning),
        "cloud" => simple(ShapeKind::Cloud),
        "teardrop" => simple(ShapeKind::Teardrop),
        "moon" => simple(ShapeKind::Crescent),
        "pentagon" => polygon(5),
        "hexagon" => polygon(6),
        "heptagon" => polygon(7),
        "octagon" => polygon(8),
        "decagon" => polygon(10),
        "dodecagon" => polygon(12),
        "mathMultiply" => simple(ShapeKind::XMark),
        "plus" => {
            let mut d = blank(ShapeKind::Cross);
            d.inner = (1.0 - 2.0 * adjust.unwrap_or(0.25)).clamp(0.05, 0.95);
            Some((ShapeKind::Cross, d))
        }
        "frame" => {
            let mut d = blank(ShapeKind::Frame);
            d.inner = (1.0 - 2.0 * adjust.unwrap_or(0.125)).clamp(0.05, 0.95);
            Some((ShapeKind::Frame, d))
        }
        "donut" => {
            let mut d = blank(ShapeKind::Ring);
            d.inner = (1.0 - 2.0 * adjust.unwrap_or(0.25)).clamp(0.05, 0.95);
            Some((ShapeKind::Ring, d))
        }
        "wedgeRectCallout" => simple(ShapeKind::SpeechBubble),
        "wedgeRoundRectCallout" => {
            let mut d = blank(ShapeKind::SpeechBubble);
            d.corner_radius = 0.16667 * min_side;
            Some((ShapeKind::SpeechBubble, d))
        }
        "wedgeEllipseCallout" | "cloudCallout" => simple(ShapeKind::OvalBubble),
        "star4" => star(4),
        "star5" => star(5),
        "star6" => star(6),
        "star7" => star(7),
        "star8" => star(8),
        "star10" => star(10),
        "star12" => star(12),
        "star16" => star(16),
        "star24" => star(24),
        "star32" => star(32),
        _ => None,
    }
}

fn is_line(prst: &str) -> bool {
    matches!(prst, "line" | "straightConnector1" | "bentConnector2" | "bentConnector3" | "curvedConnector3")
}

/// The first `adj` / `adj1` value of a preset shape, as a fraction.
fn adjust_of(geom: N<'_>) -> Option<f64> {
    let list = child(geom, "avLst")?;
    let gd = list.children().find(|g| g.is_element() && g.tag_name().name() == "gd")?;
    let v = gd.attribute("fmla")?.strip_prefix("val ")?.trim().parse::<f64>().ok()?;
    Some(v / 100_000.0)
}

/// How a line looks.
#[derive(Clone, Copy)]
struct Stroke {
    color: ColorRgba,
    width: f64,
    dash: StrokeStyle,
    arrow_start: bool,
    arrow_end: bool,
}

// ---------------------------------------------------------------- the text of a shape

#[derive(Clone, PartialEq)]
struct Look {
    family: String,
    size: f64,
    bold: bool,
    italic: bool,
    color: ColorRgba,
    align: TextAlign,
    spacing: f64,
    letters: f64,
    margin: f64,
    before: f64,
}

struct Block {
    look: Look,
    lines: Vec<String>,
}

fn points_to_px(points: f64) -> f64 {
    points * 96.0 / 72.0
}

// ---------------------------------------------------------------- the walker

struct Walker<'x, 'p> {
    pkg: &'p mut Package,
    theme: &'x Theme,
    rels: Rels,
    layout: Option<N<'x>>,
    master: Option<N<'x>>,
    /// `txStyles` of the slide master.
    styles: Option<N<'x>>,
    page_w: f64,
    page_h: f64,
    out: Vec<(String, StoredObject)>,
    skipped: usize,
}

/// The `p:ph` of a shape: its type and index.
fn placeholder<'a>(sp: N<'a>) -> Option<(&'a str, Option<&'a str>)> {
    let ph = child(sp, "nvSpPr").and_then(|n| child(n, "nvPr")).and_then(|n| child(n, "ph"))?;
    Some((ph.attribute("type").unwrap_or("body"), ph.attribute("idx")))
}

fn ph_family(kind: &str) -> &'static str {
    match kind {
        "title" | "ctrTitle" => "title",
        "body" | "subTitle" | "obj" => "body",
        _ => "other",
    }
}

fn find_placeholder<'a>(tree: N<'a>, kind: &str, idx: Option<&str>, by_index: bool) -> Option<N<'a>> {
    let sps = tree.descendants().filter(|n| n.is_element() && n.tag_name().name() == "sp");
    let mut best = None;
    for sp in sps {
        let Some((k, i)) = placeholder(sp) else { continue };
        if by_index && idx.is_some() && i == idx && idx != Some("0") {
            return Some(sp);
        }
        if ph_family(k) == ph_family(kind) && (best.is_none() || k == kind) {
            best = Some(sp);
        }
    }
    best
}

impl<'x> Walker<'x, '_> {
    fn frame_of(&self, emu_per_page_unit: f64) -> Frame {
        Frame { tx: 0.0, ty: 0.0, sx: emu_per_page_unit, sy: emu_per_page_unit }
    }

    fn name_of(n: N<'_>, fallback: &str) -> String {
        let name = n
            .descendants()
            .find(|c| c.is_element() && c.tag_name().name() == "cNvPr")
            .and_then(|c| c.attribute("name"))
            .unwrap_or("");
        if name.is_empty() { fallback.to_owned() } else { name.to_owned() }
    }

    fn hidden(n: N<'_>) -> bool {
        n.descendants()
            .find(|c| c.is_element() && c.tag_name().name() == "cNvPr")
            .is_some_and(|c| c.attribute("hidden") == Some("1"))
    }

    fn walk(&mut self, tree: N<'x>, frame: Frame, skip_placeholders: bool) {
        for n in tree.children().filter(Node::is_element) {
            if Self::hidden(n) {
                continue;
            }
            match n.tag_name().name() {
                "sp" => {
                    if !(skip_placeholders && placeholder(n).is_some()) {
                        self.sp(n, frame);
                    }
                }
                "pic" => self.pic(n, frame),
                "cxnSp" => self.connector(n, frame),
                "grpSp" => {
                    let Some(props) = child(n, "grpSpPr") else { continue };
                    let inner = (|| {
                        let x = xfrm_in(props)?;
                        let c = child(props, "xfrm")?;
                        let off = child(c, "chOff")?;
                        let ext = child(c, "chExt")?;
                        let (cw, ch) = (num(ext, "cx")?, num(ext, "cy")?);
                        let rx = if cw > 0.0 { x.w / cw } else { 1.0 };
                        let ry = if ch > 0.0 { x.h / ch } else { 1.0 };
                        let (sx, sy) = (frame.sx * rx, frame.sy * ry);
                        Some(Frame {
                            tx: frame.tx + frame.sx * x.x - sx * num(off, "x")?,
                            ty: frame.ty + frame.sy * x.y - sy * num(off, "y")?,
                            sx,
                            sy,
                        })
                    })();
                    if let Some(inner) = inner {
                        self.walk(n, inner, skip_placeholders);
                    }
                }
                "graphicFrame" => self.graphic_frame(n, frame),
                "AlternateContent" => {
                    if let Some(choice) = child(n, "Choice").or_else(|| child(n, "Fallback")) {
                        self.walk(choice, frame, skip_placeholders);
                    }
                }
                _ => {}
            }
        }
    }

    // ---- fills and lines

    fn style_color(&self, sp: N<'_>, which: &str) -> Option<ColorRgba> {
        let r = child(child(sp, "style")?, which)?;
        if num(r, "idx") == Some(0.0) {
            return None;
        }
        color_in(r, self.theme, None)
    }

    fn fill_of(&self, chain: &[N<'x>]) -> Option<ColorRgba> {
        for sp in chain {
            if let Some(props) = child(*sp, "spPr") {
                match fill_in(props, self.theme, None) {
                    Fill::Solid(c) => return Some(c),
                    Fill::None | Fill::Other => return None,
                    Fill::Inherit => {}
                }
            }
        }
        chain.first().and_then(|sp| self.style_color(*sp, "fillRef"))
    }

    fn stroke_of(&self, chain: &[N<'x>]) -> Option<Stroke> {
        let sp = *chain.first()?;
        let mut width = None;
        let mut color = None;
        let mut dash = StrokeStyle::Solid;
        let (mut arrow_start, mut arrow_end) = (false, false);
        let mut off = false;
        for holder in chain {
            let Some(ln) = child(*holder, "spPr").and_then(|p| child(p, "ln")) else { continue };
            if width.is_none() {
                width = num(ln, "w");
            }
            if color.is_none() && !off {
                match fill_in(ln, self.theme, None) {
                    Fill::Solid(c) => color = Some(c),
                    Fill::None => off = true,
                    _ => {}
                }
            }
            if let Some(d) = child(ln, "prstDash").and_then(|d| d.attribute("val")) {
                dash = match d {
                    "solid" => StrokeStyle::Solid,
                    "sysDot" | "dot" => StrokeStyle::Dotted,
                    _ => StrokeStyle::Dashed,
                };
            }
            let is_arrow = |name: &str| child(ln, name).and_then(|e| e.attribute("type")).is_some_and(|t| t != "none");
            arrow_start |= is_arrow("headEnd");
            arrow_end |= is_arrow("tailEnd");
        }
        if off {
            return None;
        }
        let color = color.or_else(|| self.style_color(sp, "lnRef"))?;
        let idx = child(sp, "style").and_then(|s| child(s, "lnRef")).and_then(|r| num(r, "idx")).unwrap_or(1.0);
        let width = width.map_or(idx.max(1.0), |w| w / EMU_PER_PIXEL).max(0.5);
        Some(Stroke { color, width, dash, arrow_start, arrow_end })
    }

    // ---- objects

    fn push(&mut self, name: String, object: StoredObject) {
        self.out.push((name, object));
    }

    fn line(&mut self, name: &str, p: &Placed, stroke: Stroke) {
        let (x1, x2) = if p.flip_h { (p.x + p.w, p.x) } else { (p.x, p.x + p.w) };
        let (y1, y2) = if p.flip_v { (p.y + p.h, p.y) } else { (p.y, p.y + p.h) };
        let length = (x2 - x1).hypot(y2 - y1).max(1.0);
        let angle = (y2 - y1).atan2(x2 - x1).to_degrees();
        let (cx, cy) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
        let thickness = stroke.width.max(2.0);
        let mut data = ShapeData::new(ShapeKind::Line, None, Some(stroke.color), stroke.width);
        data.dash = stroke.dash;
        data.arrow_start = stroke.arrow_start;
        data.arrow_end = stroke.arrow_end;
        let b = ObjectBox::new(cx - length / 2.0, cy - thickness / 2.0, length, thickness, angle);
        self.push(name.to_owned(), StoredObject::new_shape(b, &data));
    }

    fn connector(&mut self, n: N<'x>, frame: Frame) {
        let Some(x) = child(n, "spPr").and_then(xfrm_in) else { return };
        let placed = Placed::of(&x, frame);
        if let Some(stroke) = self.stroke_of(&[n]) {
            self.line(&Self::name_of(n, "Line"), &placed, stroke);
        }
    }

    fn sp(&mut self, sp: N<'x>, frame: Frame) {
        let ph = placeholder(sp);
        let mut chain = vec![sp];
        if let Some((kind, idx)) = ph {
            if let Some(l) = self.layout.and_then(|t| find_placeholder(t, kind, idx, true)) {
                chain.push(l);
            }
            if let Some(m) = self.master.and_then(|t| find_placeholder(t, kind, idx, false)) {
                chain.push(m);
            }
        }
        let Some(x) = chain.iter().find_map(|c| child(*c, "spPr").and_then(xfrm_in)) else { return };
        let placed = Placed::of(&x, frame);
        if placed.w <= 0.0 && placed.h <= 0.0 {
            return;
        }
        let name = Self::name_of(sp, "Shape");
        let geometry = chain.iter().find_map(|c| child(*c, "spPr").and_then(|p| child(p, "prstGeom")));
        let prst = geometry.and_then(|g| g.attribute("prst")).unwrap_or(if ph.is_some() { "rect" } else { "" });
        let fill = self.fill_of(&chain);
        let stroke = self.stroke_of(&chain);
        let own_props = child(sp, "spPr");
        let blip = own_props.and_then(|p| child(p, "blipFill")).and_then(|b| child(b, "blip"));
        let custom = own_props.and_then(|p| child(p, "custGeom"));
        if let Some(blip) = blip {
            // A shape filled with a picture: the picture at the place of the shape.
            let part = rid(blip, "embed").and_then(|id| self.rels.get(id)).cloned();
            let crop = own_props.and_then(|p| child(p, "blipFill")).and_then(|b| child(b, "srcRect"));
            match part.and_then(|p| self.picture(&p, crop)) {
                Some(mut object) => {
                    object.set_box(placed.object_box());
                    object.set_flips(placed.flip_h, placed.flip_v);
                    self.push(name.clone(), object);
                }
                None => self.skipped += 1,
            }
        } else if is_line(prst) {
            if let Some(stroke) = stroke {
                self.line(&name, &placed, stroke);
            }
        } else if let Some(custom) = custom {
            if fill.is_some() || stroke.is_some() {
                for (b, vertices, closed) in custom_paths(custom, &placed) {
                    let mut data = ShapeData::new(
                        ShapeKind::Polygon,
                        if closed { fill } else { None },
                        stroke.map(|s| s.color).or(if closed { None } else { fill }),
                        stroke.map_or(if closed { 0.0 } else { 1.0 }, |s| s.width),
                    );
                    data.vertices = Some(vertices);
                    if let Some(s) = stroke {
                        data.dash = s.dash;
                    }
                    let mut object = StoredObject::new_shape(b, &data);
                    object.set_flips(placed.flip_h, placed.flip_v);
                    self.push(name.clone(), object);
                }
            }
        } else if fill.is_some() || stroke.is_some() {
            let adjust = geometry.and_then(adjust_of);
            let stroke_width = stroke.map_or(0.0, |s| s.width);
            // Something this program cannot draw the same way: a plain box at the same place is the closest.
            if let Some((_, mut data)) =
                shape_kind(prst, adjust, placed.w.min(placed.h)).or_else(|| shape_kind("rect", None, 0.0))
            {
                data.fill = fill;
                data.stroke = stroke.map(|s| s.color);
                data.stroke_width = stroke_width;
                if let Some(s) = stroke {
                    data.dash = s.dash;
                }
                let mut object = StoredObject::new_shape(placed.object_box(), &data);
                object.set_flips(placed.flip_h, placed.flip_v);
                self.push(name.clone(), object);
            }
        }
        if let Some(body) = child(sp, "txBody") {
            self.text(sp, &chain, body, &placed, &name);
        }
    }

    fn pic(&mut self, n: N<'x>, frame: Frame) {
        let Some(x) = child(n, "spPr").and_then(xfrm_in) else { return };
        let placed = Placed::of(&x, frame);
        let blip = child(n, "blipFill").and_then(|b| child(b, "blip"));
        let Some(part) = blip.and_then(|b| rid(b, "embed")).and_then(|id| self.rels.get(id)).cloned() else {
            self.skipped += 1;
            return;
        };
        let name = Self::name_of(n, "Picture");
        let src = child(n, "blipFill").and_then(|b| child(b, "srcRect"));
        match self.picture(&part, src) {
            Some(mut object) => {
                object.set_box(placed.object_box());
                object.set_flips(placed.flip_h, placed.flip_v);
                self.push(name, object);
            }
            None => self.skipped += 1,
        }
    }

    /// A picture of the package as an image object (cropped when the slide crops it).
    fn picture(&mut self, part: &str, crop: Option<N<'_>>) -> Option<StoredObject> {
        let bytes = self.pkg.bytes(part)?;
        let extension = part.rsplit('.').next().map(str::to_ascii_lowercase);
        let mut picture = decode::load_bytes(&bytes, extension.as_deref()).ok()?;
        if let Some(c) = crop {
            let side = |name: &str| num(c, name).unwrap_or(0.0) / 100_000.0;
            let (w, h) = (f64::from(picture.width()), f64::from(picture.height()));
            let (l, t) = ((w * side("l")).round() as i32, (h * side("t")).round() as i32);
            let (r, b) = ((w * side("r")).round() as i32, (h * side("b")).round() as i32);
            let area = PixelRect::new(l, t, picture.width() as i32 - l - r, picture.height() as i32 - t - b);
            if area.width > 0 && area.height > 0 && area != picture.bounds() {
                picture = picture.crop(area.intersect(&picture.bounds()));
            }
        }
        let (w, h) = (f64::from(picture.width()), f64::from(picture.height()));
        Some(StoredObject::new_image(ObjectBox::new(0.0, 0.0, w, h, 0.0), picture_bytes(&picture), "Picture"))
    }

    fn graphic_frame(&mut self, n: N<'x>, frame: Frame) {
        let Some(x) = xfrm_in(n) else {
            self.skipped += 1;
            return;
        };
        let placed = Placed::of(&x, frame);
        let table = n.descendants().find(|c| c.is_element() && c.tag_name().name() == "tbl");
        let Some(table) = table else {
            self.skipped += 1; // a chart, SmartArt, an embedded object...
            return;
        };
        let columns: Vec<f64> = child(table, "tblGrid")
            .map(|g| kids(g, "gridCol").filter_map(|c| num(c, "w")).map(|w| w * frame.sx).collect())
            .unwrap_or_default();
        let mut y = placed.y;
        for row in kids(table, "tr") {
            let height = num(row, "h").unwrap_or(370_840.0) * frame.sy;
            let mut x = placed.x;
            let mut column = 0;
            for cell in kids(row, "tc") {
                let span = num(cell, "gridSpan").map_or(1, |s| s.max(1.0) as usize);
                let width: f64 = columns.iter().skip(column).take(span).sum();
                column += span;
                let here = Placed { x, y, w: width, h: height, rot: 0.0, flip_h: false, flip_v: false };
                x += width;
                if cell.attribute("hMerge") == Some("1") || cell.attribute("vMerge") == Some("1") {
                    continue;
                }
                let props = child(cell, "tcPr");
                let fill = props.and_then(|p| match fill_in(p, self.theme, None) {
                    Fill::Solid(c) => Some(c),
                    _ => None,
                });
                let mut data = ShapeData::new(ShapeKind::Rectangle, fill, Some(ColorRgba::rgb(191, 191, 191)), 1.0);
                data.corner_radius = 0.0;
                self.push("Table cell".into(), StoredObject::new_shape(here.object_box(), &data));
                if let Some(body) = child(cell, "txBody") {
                    self.text(cell, &[cell], body, &here, "Table text");
                }
            }
            y += height;
        }
    }

    // ---- text

    /// Finds an attribute through the places a paragraph can get it from.
    fn attr<'a>(places: &[N<'a>], name: &str) -> Option<&'a str> {
        places.iter().find_map(|p| p.attribute(name))
    }

    fn text(&mut self, owner: N<'x>, chain: &[N<'x>], body: N<'x>, placed: &Placed, name: &str) {
        let body_props =
            chain.iter().filter_map(|c| child(*c, "txBody").and_then(|b| child(b, "bodyPr"))).collect::<Vec<_>>();
        let own_props = child(body, "bodyPr");
        let mut props: Vec<N<'x>> = own_props.into_iter().collect();
        props.extend(body_props);
        let inset =
            |name: &str, default: f64| Self::attr(&props, name).and_then(|v| v.parse::<f64>().ok()).unwrap_or(default);
        let (l, t) = (inset("lIns", 91_440.0) / EMU_PER_PIXEL, inset("tIns", 45_720.0) / EMU_PER_PIXEL);
        let (r, b) = (inset("rIns", 91_440.0) / EMU_PER_PIXEL, inset("bIns", 45_720.0) / EMU_PER_PIXEL);
        let wrap = Self::attr(&props, "wrap") != Some("none");
        let anchor = Self::attr(&props, "anchor").unwrap_or("t");
        let font_scale = props
            .iter()
            .find_map(|p| child(*p, "normAutofit"))
            .and_then(|a| num(a, "fontScale"))
            .map_or(1.0, |s| s / 100_000.0);
        let (inner_x, inner_w) = (placed.x + l, (placed.w - l - r).max(1.0));

        // The styles that apply: the shape's own lists, then those of the layout and the master.
        let kind = placeholder(owner).map_or("other", |(k, _)| ph_family(k));
        let style_list = self.styles.and_then(|s| {
            child(
                s,
                match kind {
                    "title" => "titleStyle",
                    "body" => "bodyStyle",
                    _ => "otherStyle",
                },
            )
        });
        let font_ref = child(owner, "style").and_then(|s| child(s, "fontRef"));
        let default_color = font_ref
            .and_then(|f| color_in(f, self.theme, None))
            .or_else(|| self.theme.scheme("tx1"))
            .unwrap_or(ColorRgba::BLACK);

        let mut blocks: Vec<Block> = Vec::new();
        for paragraph in kids(body, "p") {
            let ppr = child(paragraph, "pPr");
            let level = ppr.and_then(|p| num(p, "lvl")).unwrap_or(0.0) as usize + 1;
            let level_name = format!("lvl{level}pPr");
            let mut ppr_places: Vec<N<'x>> = ppr.into_iter().collect();
            for c in chain {
                if let Some(l) =
                    child(*c, "txBody").and_then(|b| child(b, "lstStyle")).and_then(|s| child(s, &level_name))
                {
                    ppr_places.push(l);
                }
            }
            if let Some(l) = style_list.and_then(|s| child(s, &level_name)) {
                ppr_places.push(l);
            }
            let runs: Vec<N<'x>> =
                paragraph.children().filter(|c| c.is_element() && matches!(c.tag_name().name(), "r" | "fld")).collect();
            let mut text = String::new();
            for part in paragraph.children().filter(Node::is_element) {
                match part.tag_name().name() {
                    "r" | "fld" => {
                        if let Some(t) = child(part, "t") {
                            text.push_str(t.text().unwrap_or(""));
                        }
                    }
                    "br" => text.push('\n'),
                    _ => {}
                }
            }
            let first =
                runs.iter().copied().find(|r| child(*r, "t").and_then(|t| t.text()).is_some_and(|t| !t.is_empty()));
            let run_props = first.and_then(|r| child(r, "rPr"));
            let end_props = child(paragraph, "endParaRPr");
            let mut rpr_places: Vec<N<'x>> = run_props.into_iter().collect();
            if first.is_none() {
                rpr_places.extend(end_props);
            }
            rpr_places.extend(ppr_places.iter().filter_map(|p| child(*p, "defRPr")));

            let size = Self::attr(&rpr_places, "sz").and_then(|v| v.parse::<f64>().ok()).unwrap_or(1800.0);
            let bold = Self::attr(&rpr_places, "b") == Some("1");
            let italic = Self::attr(&rpr_places, "i") == Some("1");
            let family = rpr_places
                .iter()
                .find_map(|p| child(*p, "latin").and_then(|l| l.attribute("typeface")))
                .map_or_else(|| self.theme.font(""), |f| self.theme.font(f));
            let color = rpr_places
                .iter()
                .find_map(|p| child(*p, "solidFill").and_then(|f| color_in(f, self.theme, None)))
                .unwrap_or(default_color);
            let align = match Self::attr(&ppr_places, "algn") {
                Some("ctr") => TextAlign::Center,
                Some("r") => TextAlign::Right,
                _ => TextAlign::Left,
            };
            let spacing =
                ppr_places.iter().find_map(|p| child(*p, "lnSpc").and_then(|s| percent_of(s, "spcPct"))).unwrap_or(1.0);
            let letters = Self::attr(&rpr_places, "spc").and_then(|v| v.parse::<f64>().ok()).map_or(0.0, |s| s / 75.0);
            let margin =
                Self::attr(&ppr_places, "marL").and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) / EMU_PER_PIXEL;
            let before = ppr_places
                .iter()
                .find_map(|p| child(*p, "spcBef").and_then(|s| num(child(s, "spcPts")?, "val")))
                .map_or(0.0, |v| points_to_px(v / 100.0));
            let bulleted = ppr_places.iter().find_map(|p| {
                p.children()
                    .find(|c| c.is_element() && matches!(c.tag_name().name(), "buChar" | "buAutoNum" | "buNone"))
                    .map(|c| c.tag_name().name() != "buNone")
            }) == Some(true);
            if bulleted && !text.is_empty() {
                let bullet = ppr_places
                    .iter()
                    .find_map(|p| child(*p, "buChar").and_then(|c| c.attribute("char")))
                    .filter(|c| c.chars().all(|ch| ch.is_ascii() || ch == '\u{2022}'))
                    .unwrap_or("\u{2022}");
                text = format!("{bullet} {text}");
            }
            let look = Look {
                family,
                size: (points_to_px(size / 100.0) * font_scale).max(4.0),
                bold,
                italic,
                color,
                align,
                spacing,
                letters,
                margin,
                before,
            };
            let lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
            match blocks.last_mut() {
                Some(last) if last.look == look => last.lines.extend(lines),
                _ => blocks.push(Block { look, lines }),
            }
        }
        // Empty paragraphs at the end change nothing.
        while blocks.last().is_some_and(|b| b.lines.iter().all(String::is_empty)) {
            blocks.pop();
        }
        if blocks.is_empty() {
            return;
        }

        // Make every block a text object (wrapped to the width of the shape) and find how tall everything is.
        struct Made {
            data: TextData,
            w: f64,
            h: f64,
            before: f64,
            margin: f64,
        }
        let mut made = Vec::new();
        for block in blocks {
            let look = block.look;
            let available = (inner_w - look.margin).max(look.size);
            let mut data = TextData::new("", &look.family, look.size, look.color);
            data.bold = look.bold;
            data.italic = look.italic;
            data.align = look.align;
            data.line_spacing = look.spacing;
            data.letter_spacing = look.letters;
            let lines: Vec<String> = block
                .lines
                .iter()
                .flat_map(|line| if wrap { wrap_line(&data, line, available) } else { vec![line.clone()] })
                .collect();
            data.text = lines.join("\n");
            if data.text.trim().is_empty() {
                // A blank line still takes room.
                let h = look.size * 1.2 * look.spacing * lines.len() as f64;
                made.push(Made { data, w: 0.0, h, before: look.before, margin: look.margin });
                continue;
            }
            let (w, h) = lay_out(&data).map_or_else(
                || {
                    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1) as f64;
                    (longest * look.size * 0.5, lines.len() as f64 * look.size * 1.2 * look.spacing)
                },
                |l| (l.width, l.height),
            );
            made.push(Made { data, w, h, before: look.before, margin: look.margin });
        }
        let total: f64 = made.iter().enumerate().map(|(i, m)| m.h + if i > 0 { m.before } else { 0.0 }).sum();
        let inner_h = (placed.h - t - b).max(1.0);
        let top = match anchor {
            "ctr" => (inner_h - total) / 2.0,
            "b" => inner_h - total,
            _ => 0.0,
        };
        let top = if total > inner_h { 0.0 } else { top };
        let mut y = placed.y + t + top;
        let (cx, cy) = placed.center();
        let angle = placed.rot.to_radians();
        for (i, m) in made.into_iter().enumerate() {
            if i > 0 {
                y += m.before;
            }
            let h = m.h;
            if m.w > 0.0 {
                let x = match m.data.align {
                    TextAlign::Left => inner_x + m.margin,
                    TextAlign::Center => inner_x + (inner_w - m.w) / 2.0,
                    TextAlign::Right => inner_x + inner_w - m.w,
                };
                // A rotated shape carries its text around its middle.
                let (mx, my) = (x + m.w / 2.0, y + h / 2.0);
                let (dx, dy) = (mx - cx, my - cy);
                let (rx, ry) = (dx * angle.cos() - dy * angle.sin(), dx * angle.sin() + dy * angle.cos());
                let object = StoredObject::new_text(
                    ObjectBox::new(cx + rx - m.w / 2.0, cy + ry - h / 2.0, m.w, h, placed.rot),
                    &m.data,
                );
                self.push(name.to_owned(), object);
            }
            y += h;
        }
    }
}

/// The outlines of a freeform shape (`a:custGeom`): one polygon per sub-path (curves are flattened), each with the box it
/// covers on the page and its corners as fractions of that box.
fn custom_paths(geom: N<'_>, placed: &Placed) -> Vec<(ObjectBox, Vec<PointD>, bool)> {
    let mut result = Vec::new();
    let Some(list) = child(geom, "pathLst") else { return result };
    for path in kids(list, "path") {
        let (pw, ph) = (num(path, "w").unwrap_or(0.0), num(path, "h").unwrap_or(0.0));
        let (pw, ph) = (if pw > 0.0 { pw } else { 1.0 }, if ph > 0.0 { ph } else { 1.0 });
        let at =
            |p: N<'_>| -> Option<(f64, f64)> { Some((num(p, "x")? / pw * placed.w, num(p, "y")? / ph * placed.h)) };
        let mut runs: Vec<(Vec<(f64, f64)>, bool)> = Vec::new();
        let mut current: Vec<(f64, f64)> = Vec::new();
        let mut last = (0.0, 0.0);
        for seg in path.children().filter(Node::is_element) {
            let pts: Vec<(f64, f64)> = kids(seg, "pt").filter_map(at).collect();
            match (seg.tag_name().name(), pts.as_slice()) {
                ("moveTo", [p, ..]) => {
                    if current.len() > 1 {
                        runs.push((std::mem::take(&mut current), false));
                    }
                    current.clear();
                    current.push(*p);
                    last = *p;
                }
                ("lnTo", [p, ..]) => {
                    current.push(*p);
                    last = *p;
                }
                ("cubicBezTo", [c1, c2, e]) => {
                    for i in 1..=12 {
                        let t = f64::from(i) / 12.0;
                        let u = 1.0 - t;
                        let mix = |a: f64, b: f64, c: f64, d: f64| {
                            u * u * u * a + 3.0 * u * u * t * b + 3.0 * u * t * t * c + t * t * t * d
                        };
                        current.push((mix(last.0, c1.0, c2.0, e.0), mix(last.1, c1.1, c2.1, e.1)));
                    }
                    last = *e;
                }
                ("quadBezTo", [c, e]) => {
                    for i in 1..=10 {
                        let t = f64::from(i) / 10.0;
                        let u = 1.0 - t;
                        let mix = |a: f64, b: f64, d: f64| u * u * a + 2.0 * u * t * b + t * t * d;
                        current.push((mix(last.0, c.0, e.0), mix(last.1, c.1, e.1)));
                    }
                    last = *e;
                }
                ("close", _) => {
                    if current.len() > 2 {
                        runs.push((std::mem::take(&mut current), true));
                    }
                    current.clear();
                }
                _ => {}
            }
        }
        if current.len() > 1 {
            runs.push((current, false));
        }
        for (points, closed) in runs {
            let min_x = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
            let min_y = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
            let max_x = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
            let max_y = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
            let (w, h) = ((max_x - min_x).max(1.0), (max_y - min_y).max(1.0));
            let unit = points.iter().map(|p| PointD::new((p.0 - min_x) / w, (p.1 - min_y) / h)).collect();
            result.push((ObjectBox::new(placed.x + min_x, placed.y + min_y, w, h, 0.0), unit, closed));
        }
    }
    result
}

/// Breaks a line into lines that fit `width` pixels, at spaces.
fn wrap_line(look: &TextData, line: &str, width: f64) -> Vec<String> {
    if line.chars().count() < 3 {
        return vec![line.to_owned()];
    }
    let measure = |s: &str| {
        let mut t = look.clone();
        t.text = s.to_owned();
        lay_out(&t).map_or(s.chars().count() as f64 * look.size * 0.5, |l| l.width - 2.0)
    };
    let space = measure("a a") - measure("aa");
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut used = 0.0;
    for word in line.split(' ') {
        let w = measure(word);
        if current.is_empty() {
            current.push_str(word);
            used = w;
        } else if used + space + w <= width {
            current.push(' ');
            current.push_str(word);
            used += space + w;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            used = w;
        }
    }
    lines.push(current);
    lines
}

// ---------------------------------------------------------------- slides

/// A background fill found on a slide, a layout or a master.
fn background(tree: N<'_>, theme: &Theme) -> Fill {
    let Some(bg) = tree.descendants().find(|n| n.is_element() && n.tag_name().name() == "bg") else {
        return Fill::Inherit;
    };
    if let Some(props) = child(bg, "bgPr") {
        return fill_in(props, theme, None);
    }
    if let Some(r) = child(bg, "bgRef") {
        return color_in(r, theme, None).map_or(Fill::Inherit, Fill::Solid);
    }
    Fill::Inherit
}

/// Reads the bytes of a .pptx file.
pub fn read_pptx(data: Vec<u8>) -> Result<Imported, IoError> {
    let zip = zip::ZipArchive::new(Cursor::new(data)).map_err(|_| not_pptx())?;
    let mut pkg = Package { zip };
    let presentation = pkg.text("ppt/presentation.xml").ok_or_else(not_pptx)?;
    let presentation_doc = roxmltree::Document::parse(&presentation).map_err(|_| not_pptx())?;
    let root = presentation_doc.root_element();
    let size = child(root, "sldSz").ok_or_else(not_pptx)?;
    let (cx, cy) = (num(size, "cx").unwrap_or(9_144_000.0), num(size, "cy").unwrap_or(6_858_000.0));
    let scale = (MAX_SIDE / (cx.max(cy) / EMU_PER_PIXEL)).min(1.0);
    let width = (cx / EMU_PER_PIXEL * scale).round().max(1.0) as u32;
    let height = (cy / EMU_PER_PIXEL * scale).round().max(1.0) as u32;
    let unit = scale / EMU_PER_PIXEL;

    let rels = pkg.rels("ppt/presentation.xml");
    let slide_parts: Vec<String> = child(root, "sldIdLst")
        .map(|l| kids(l, "sldId").filter_map(|s| rid(s, "id").and_then(|id| rels.get(id)).cloned()).collect())
        .unwrap_or_default();
    if slide_parts.is_empty() {
        return Err(IoError::CannotOpen("This presentation has no slides.".into()));
    }

    let mut pages = Vec::new();
    let mut skipped = 0;
    for (number, part) in slide_parts.iter().enumerate() {
        let Some(slide_xml) = pkg.text(part) else { continue };
        let Ok(slide_doc) = roxmltree::Document::parse(&slide_xml) else { continue };
        let slide_rels = pkg.rels(part);
        let layout_part = slide_rels.values().find(|v| v.contains("slideLayouts/")).cloned();
        let layout_xml = layout_part.as_ref().and_then(|p| pkg.text(p));
        let layout_doc = layout_xml.as_ref().and_then(|x| roxmltree::Document::parse(x).ok());
        let layout_rels = layout_part.as_ref().map(|p| pkg.rels(p)).unwrap_or_default();
        let master_part = layout_rels.values().find(|v| v.contains("slideMasters/")).cloned();
        let master_xml = master_part.as_ref().and_then(|p| pkg.text(p));
        let master_doc = master_xml.as_ref().and_then(|x| roxmltree::Document::parse(x).ok());
        let master_rels = master_part.as_ref().map(|p| pkg.rels(p)).unwrap_or_default();
        let theme_part = master_rels.values().find(|v| v.contains("theme/")).cloned();
        let theme_xml = theme_part.as_ref().and_then(|p| pkg.text(p));
        let theme = Theme::read(theme_xml.as_deref(), master_doc.as_ref().map(|d| d.root_element()));

        let slide_root = slide_doc.root_element();
        let layout_root = layout_doc.as_ref().map(|d| d.root_element());
        let master_root = master_doc.as_ref().map(|d| d.root_element());
        let paper = [Some(slide_root), layout_root, master_root]
            .into_iter()
            .flatten()
            .map(|t| background(t, &theme))
            .find(|f| *f != Fill::Inherit);
        let color = match paper {
            Some(Fill::Solid(c)) => ColorRgba::rgb(c.r, c.g, c.b),
            _ => ColorRgba::WHITE,
        };
        let mut walker = Walker {
            pkg: &mut pkg,
            theme: &theme,
            rels: Rels::new(),
            layout: layout_root,
            master: master_root,
            styles: master_root.and_then(|m| child(m, "txStyles")),
            page_w: f64::from(width),
            page_h: f64::from(height),
            out: Vec::new(),
            skipped: 0,
        };
        let frame = walker.frame_of(unit);
        // A picture as the background of the slide.
        if paper == Some(Fill::Other) {
            let holder = [(Some(slide_root), &slide_rels), (layout_root, &layout_rels), (master_root, &master_rels)]
                .into_iter()
                .find_map(|(t, r)| {
                    let blip = t?
                        .descendants()
                        .find(|n| n.is_element() && n.tag_name().name() == "bg")?
                        .descendants()
                        .find(|n| n.is_element() && n.tag_name().name() == "blip")?;
                    r.get(rid(blip, "embed")?).cloned()
                });
            if let Some(part) = holder
                && let Some(mut object) = walker.picture(&part, None)
            {
                object.set_box(ObjectBox::new(0.0, 0.0, walker.page_w, walker.page_h, 0.0));
                walker.push("Background".into(), object);
            }
        }
        let show_master = slide_root.attribute("showMasterSp") != Some("0");
        if show_master {
            if let Some(m) = master_root
                && layout_root.is_none_or(|l| l.attribute("showMasterSp") != Some("0"))
                && let Some(tree) = m.descendants().find(|n| n.is_element() && n.tag_name().name() == "spTree")
            {
                walker.rels = master_rels.clone();
                walker.walk(tree, frame, true);
            }
            if let Some(l) = layout_root
                && let Some(tree) = l.descendants().find(|n| n.is_element() && n.tag_name().name() == "spTree")
            {
                walker.rels = layout_rels.clone();
                walker.walk(tree, frame, true);
            }
        }
        if let Some(tree) = slide_root.descendants().find(|n| n.is_element() && n.tag_name().name() == "spTree") {
            walker.rels = slide_rels.clone();
            walker.walk(tree, frame, false);
        }
        skipped += walker.skipped;
        let objects = std::mem::take(&mut walker.out);

        let mut document = Document::blank(width, height, color);
        let canvas = document.bounds();
        for (n, (name, object)) in objects.into_iter().enumerate() {
            let name = if name.is_empty() { format!("Object {}", n + 1) } else { name };
            document.push_layer(object_ops::object_layer(name, object, canvas));
        }
        pages.push(Page::new(format!("Slide {}", number + 1), document));
    }
    if pages.is_empty() {
        return Err(not_pptx());
    }
    Ok(Imported { pages, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;
    use crate::io::pptx::{EditableOptions, build_editable};
    use crate::objects::ShapeKind;

    #[test]
    fn a_slide_we_wrote_opens_again_with_its_shapes_and_text() {
        let mut doc = Document::blank(640, 360, ColorRgba::rgb(240, 240, 250));
        let canvas = doc.bounds();
        let star = ShapeData::new(ShapeKind::Star, Some(ColorRgba::rgb(250, 200, 0)), None, 0.0);
        doc.push_layer(object_ops::object_layer(
            "Star",
            StoredObject::new_shape(ObjectBox::new(20.0, 20.0, 100.0, 100.0, 0.0), &star),
            canvas,
        ));
        let text = TextData::new("Hello world", "Arial", 32.0, ColorRgba::rgb(10, 20, 30));
        doc.push_layer(object_ops::object_layer(
            "Title",
            StoredObject::new_text(ObjectBox::new(150.0, 150.0, 300.0, 50.0, 0.0), &text),
            canvas,
        ));
        let bytes = build_editable(&[&doc, &doc], EditableOptions::default()).expect("built");
        let imported = read_pptx(bytes).expect("opened");
        assert_eq!(imported.pages.len(), 2);
        let page = &imported.pages[0].doc;
        assert_eq!((page.width(), page.height()), (640, 360));
        let objects: Vec<&StoredObject> = page.layers().iter().filter_map(|l| l.object()).collect();
        let shape =
            objects.iter().filter_map(|o| o.shape()).find(|s| s.kind == ShapeKind::Star).expect("the star is a shape");
        assert_eq!((shape.kind, shape.points), (ShapeKind::Star, 5));
        assert_eq!(shape.fill, Some(ColorRgba::rgb(250, 200, 0)));
        let text = objects.iter().find_map(|o| o.text()).expect("the text is a text object");
        assert_eq!((text.text.as_str(), text.family.as_str()), ("Hello world", "Arial"));
        assert!((text.size - 32.0).abs() < 0.6, "the size is kept: {}", text.size);
        let b = objects.iter().find(|o| o.kind() == "text").and_then(|o| o.box_()).expect("box");
        assert!((b.x - 150.0).abs() < 2.0 && (b.y - 150.0).abs() < 2.0, "the place is kept: {} {}", b.x, b.y);
    }

    #[test]
    fn something_that_is_not_pptx_is_refused() {
        assert!(read_pptx(b"not a zip".to_vec()).is_err());
        assert!(is_pptx("a.PPTX") && !is_pptx("a.ppt"));
    }
}
