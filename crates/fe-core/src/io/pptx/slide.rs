//! One slide being built (C# `PptxSlide`): a list of shapes, text boxes and pictures in stacking order (first = at the
//! back). All positions and sizes are in pixels of the page (96 per inch); the writer turns them into the units
//! PowerPoint uses.

use std::fmt::Write as _;

use crate::imaging::ColorRgba;

const EMU: f64 = 9525.0;

fn emu(px: f64) -> i64 {
    (px * EMU).round_ties_even() as i64
}

/// Text for an XML attribute or element: escaped, without characters XML does not allow.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) >= 0x20 && c != '\u{FFFE}' && c != '\u{FFFF}' => out.push(c),
            _ => {}
        }
    }
    out
}

/// A picture used by a slide.
pub struct Media {
    pub data: Vec<u8>,
    pub png: bool,
}

/// Where a shape is and how it is turned.
#[derive(Clone, Copy, Debug, Default)]
pub struct Place {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub rotation: f64,
    pub flip_x: bool,
    pub flip_y: bool,
}

/// The look of a shape.
#[derive(Clone, Debug)]
pub struct ShapeLook {
    pub fill: Option<ColorRgba>,
    pub stroke: Option<ColorRgba>,
    pub stroke_width: f64,
    pub opacity: f64,
    pub dash: &'static str,
    pub arrow_start: bool,
    pub arrow_end: bool,
    /// Shape settings (name, value), like ("adj", 16667) for rounded corners.
    pub adjust: Vec<(&'static str, i32)>,
}

impl Default for ShapeLook {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: None,
            stroke_width: 0.0,
            opacity: 1.0,
            dash: "solid",
            arrow_start: false,
            arrow_end: false,
            adjust: Vec::new(),
        }
    }
}

/// The look of a text box.
#[derive(Clone, Debug)]
pub struct TextLook {
    pub font_family: String,
    /// In pixels (96 per inch); PowerPoint wants points.
    pub font_size_px: f64,
    pub color: ColorRgba,
    pub bold: bool,
    pub italic: bool,
    /// "l", "ctr" or "r".
    pub align: &'static str,
    pub opacity: f64,
    /// 1 = normal.
    pub line_spacing: f64,
    pub letter_spacing_px: f64,
}

#[derive(Default)]
pub struct PptxSlide {
    xml: String,
    next_id: u32,
    /// The pictures used by this slide (the first one is relationship rId2).
    pub media: Vec<Media>,
}

fn mark(px: f64) -> String {
    format!("{{{{{}}}}}", emu(px))
}

fn color(c: ColorRgba, opacity: f64) -> String {
    let alpha = f64::from(c.a) / 255.0 * opacity.clamp(0.0, 1.0);
    let a = if alpha >= 0.999 {
        String::new()
    } else {
        format!("<a:alpha val=\"{}\"/>", (alpha * 100_000.0).round_ties_even() as i64)
    };
    format!("<a:srgbClr val=\"{:02X}{:02X}{:02X}\">{a}</a:srgbClr>", c.r, c.g, c.b)
}

fn xfrm(p: Place) -> String {
    // The box is given unrotated, top-left at (x, y): PowerPoint also turns shapes around their center.
    let rot = (((p.rotation % 360.0) + 360.0) % 360.0 * 60000.0).round_ties_even() as i64;
    let mut attrs = String::new();
    if rot != 0 {
        let _ = write!(attrs, " rot=\"{rot}\"");
    }
    if p.flip_x {
        attrs.push_str(" flipH=\"1\"");
    }
    if p.flip_y {
        attrs.push_str(" flipV=\"1\"");
    }
    format!(
        "<a:xfrm{attrs}><a:off x=\"{}\" y=\"{}\"/><a:ext cx=\"{}\" cy=\"{}\"/></a:xfrm>",
        mark(p.x),
        mark(p.y),
        mark(p.w.max(0.0)),
        mark(p.h.max(0.0))
    )
}

impl PptxSlide {
    pub fn new() -> Self {
        Self { xml: String::new(), next_id: 2, media: Vec::new() }
    }

    /// The shapes as XML; `scale` (slides too big for PowerPoint are shrunk) is applied to every position and size.
    pub(crate) fn to_xml(&self, scale: f64) -> String {
        let mut out = String::with_capacity(self.xml.len());
        let mut rest = self.xml.as_str();
        while let Some(start) = rest.find("{{") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            let Some(end) = after.find("}}") else { break };
            let value: i64 = after[..end].parse().unwrap_or(0);
            let _ = write!(out, "{}", (value as f64 * scale).round_ties_even() as i64);
            rest = &after[end + 2..];
        }
        out.push_str(rest);
        out
    }

    fn id(&mut self) -> u32 {
        let id = self.next_id.max(2);
        self.next_id = id + 1;
        id
    }

    pub fn add_picture(&mut self, data: Vec<u8>, png: bool, name: &str, place: Place, opacity: f64) {
        self.media.push(Media { data, png });
        let rel = format!("rId{}", self.media.len() + 1);
        let alpha = if opacity >= 0.999 {
            String::new()
        } else {
            format!("<a:alphaModFix amt=\"{}\"/>", (opacity.clamp(0.0, 1.0) * 100_000.0).round_ties_even() as i64)
        };
        let id = self.id();
        let name = escape(name);
        let _ = write!(
            self.xml,
            "<p:pic><p:nvPicPr><p:cNvPr id=\"{id}\" name=\"{name}\" descr=\"{name}\"/><p:cNvPicPr><a:picLocks noChangeAspect=\"1\"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed=\"{rel}\">{alpha}</a:blip><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr>{}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr></p:pic>",
            xfrm(place)
        );
    }

    /// `preset` is the PowerPoint shape name (rect, ellipse, star5, ...).
    pub fn add_shape(&mut self, preset: &str, name: &str, place: Place, look: &ShapeLook) {
        let id = self.id();
        let gd: String = look.adjust.iter().map(|(n, v)| format!("<a:gd name=\"{n}\" fmla=\"val {v}\"/>")).collect();
        let fill = look.fill.map_or_else(
            || "<a:noFill/>".to_owned(),
            |f| format!("<a:solidFill>{}</a:solidFill>", color(f, look.opacity)),
        );
        let line = match look.stroke {
            Some(s) if look.stroke_width > 0.0 => {
                let dash =
                    if look.dash == "solid" { String::new() } else { format!("<a:prstDash val=\"{}\"/>", look.dash) };
                let heads = format!(
                    "{}{}",
                    if look.arrow_start { "<a:headEnd type=\"triangle\"/>" } else { "" },
                    if look.arrow_end { "<a:tailEnd type=\"triangle\"/>" } else { "" }
                );
                format!(
                    "<a:ln w=\"{}\"><a:solidFill>{}</a:solidFill>{dash}{heads}</a:ln>",
                    emu(look.stroke_width),
                    color(s, look.opacity)
                )
            }
            _ => "<a:ln><a:noFill/></a:ln>".to_owned(),
        };
        let _ = write!(
            self.xml,
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr/><p:nvPr/></p:nvSpPr><p:spPr>{}<a:prstGeom prst=\"{preset}\"><a:avLst>{gd}</a:avLst></a:prstGeom>{fill}{line}</p:spPr><p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang=\"en-US\"/></a:p></p:txBody></p:sp>",
            escape(name),
            xfrm(place)
        );
    }

    pub fn add_text(&mut self, text: &str, name: &str, place: Place, look: &TextLook) {
        let id = self.id();
        let size = ((look.font_size_px * 0.75 * 100.0).round_ties_even() as i64).max(100);
        let spacing = if (look.line_spacing - 1.0).abs() < 0.01 {
            String::new()
        } else {
            format!(
                "<a:lnSpc><a:spcPct val=\"{}\"/></a:lnSpc>",
                (look.line_spacing * 100_000.0).round_ties_even() as i64
            )
        };
        let spc = if look.letter_spacing_px.abs() < 0.01 {
            String::new()
        } else {
            format!(" spc=\"{}\"", (look.letter_spacing_px * 75.0).round_ties_even() as i64)
        };
        let font = escape(&look.font_family);
        let mut paragraphs = String::new();
        for line in text.replace('\r', "").split('\n') {
            let run = if line.is_empty() {
                format!("<a:endParaRPr lang=\"en-US\" sz=\"{size}\"/>")
            } else {
                format!(
                    "<a:r><a:rPr lang=\"en-US\" sz=\"{size}\" b=\"{}\" i=\"{}\"{spc} dirty=\"0\"><a:solidFill>{}</a:solidFill><a:latin typeface=\"{font}\"/><a:cs typeface=\"{font}\"/></a:rPr><a:t>{}</a:t></a:r>",
                    u8::from(look.bold),
                    u8::from(look.italic),
                    color(look.color, look.opacity),
                    escape(line)
                )
            };
            let _ = write!(paragraphs, "<a:p><a:pPr algn=\"{}\">{spacing}</a:pPr>{run}</a:p>", look.align);
        }
        let _ = write!(
            self.xml,
            "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr>{}<a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap=\"none\" lIns=\"0\" tIns=\"0\" rIns=\"0\" bIns=\"0\" rtlCol=\"0\" anchor=\"t\"><a:noAutofit/></a:bodyPr><a:lstStyle/>{paragraphs}</p:txBody></p:sp>",
            escape(name),
            xfrm(place)
        );
    }
}
