//! Editable PowerPoint export (C# `EditablePptx`): text becomes text boxes, simple shapes become PowerPoint shapes, placed
//! pictures stay pictures. Anything PowerPoint cannot draw the same way (shadows, glow, 3D, charts, QR codes, painted
//! layers...) is placed as a picture at the right spot, so the slide always looks like the page. A page with adjustment
//! layers or blend modes becomes one picture.
#![allow(clippy::chunks_exact_to_as_chunks)] // plain 4-byte pixel chunks read best here

use super::slide::{Place, PptxSlide, ShapeLook, TextLook};
use super::{IoError, build_slides, write_file};
use crate::document::Document;
use crate::document::canvas_ops::bake_mask;
use crate::imaging::{ColorRgba, PixelBuffer, PixelRect};
use crate::io::{encode, format::ImageFileFormat};
use crate::objects::{ShapeKind, StoredObject, StrokeStyle, pictures};
use crate::paint::compositor;
use crate::text::TextAlign;

/// Settings of the editable export.
#[derive(Clone, Copy, Debug)]
pub struct EditableOptions {
    pub quality: u8,
    pub matte: Option<ColorRgba>,
}

impl Default for EditableOptions {
    fn default() -> Self {
        Self { quality: 90, matte: None }
    }
}

/// The looks PowerPoint shapes cannot show the same way.
const LOOKS: [&str; 9] = ["Shadow", "Glow", "InnerShadow", "Outline", "Gradient", "ThreeD", "Warp", "Tone", "Overlay"];

fn plain(o: &StoredObject) -> bool {
    o.json.as_object().is_some_and(|m| LOOKS.iter().all(|k| m.get(*k).is_none_or(serde_json::Value::is_null)))
        && o.blur_amount() <= 0.0
}

fn has_transparency(p: &PixelBuffer) -> bool {
    p.data().iter().skip(3).step_by(4).any(|a| *a < 255)
}

/// The color when every pixel of the buffer is the same (a flat background layer), otherwise `None`.
fn uniform(p: &PixelBuffer) -> Option<ColorRgba> {
    let d = p.data();
    if d.len() < 4 || d.chunks_exact(4).any(|px| px != &d[..4]) {
        return None;
    }
    let c = p.pixel(0, 0);
    (c.a != 0).then_some(c)
}

struct Context<'a> {
    slide: &'a mut PptxSlide,
    width: u32,
    height: u32,
    options: EditableOptions,
    canvas: PixelRect,
}

fn place(b: crate::objects::ObjectBox, flips: (bool, bool)) -> Place {
    Place { x: b.x, y: b.y, w: b.width, h: b.height, rotation: b.rotation, flip_x: flips.0, flip_y: flips.1 }
}

fn percent(v: f64) -> i32 {
    (v.clamp(0.0, 1.0) * 100_000.0).round_ties_even() as i32
}

fn dash_name(d: StrokeStyle) -> &'static str {
    match d {
        StrokeStyle::Dashed => "dash",
        StrokeStyle::Dotted => "sysDot",
        StrokeStyle::Solid => "solid",
    }
}

impl Context<'_> {
    /// A flat color over the whole page: a rectangle you can recolor in PowerPoint.
    fn fill(&mut self, color: ColorRgba, name: &str, opacity: f64) {
        let place = Place { w: f64::from(self.width), h: f64::from(self.height), ..Place::default() };
        self.slide.add_shape("rect", name, place, &ShapeLook { fill: Some(color), opacity, ..ShapeLook::default() });
    }

    fn picture_data(&self, part: &PixelBuffer) -> (Vec<u8>, bool) {
        let alpha = has_transparency(part);
        let bytes = if alpha {
            encode::encode(part, ImageFileFormat::Png, 100, None)
        } else {
            encode::encode(
                part,
                ImageFileFormat::Jpeg,
                self.options.quality,
                Some(self.options.matte.unwrap_or(ColorRgba::WHITE)),
            )
        };
        (bytes.unwrap_or_default(), alpha)
    }

    /// Places what is painted in `pixels` (canvas sized) as one picture of just the covered area.
    fn picture(&mut self, pixels: &PixelBuffer, name: &str, opacity: f64) {
        let area = pixels.visible_bounds().intersect(&pixels.bounds());
        if area.is_empty() {
            return;
        }
        let part = if area == pixels.bounds() { pixels.clone() } else { pixels.crop(area) };
        let (data, alpha) = self.picture_data(&part);
        let place = Place {
            x: f64::from(area.x),
            y: f64::from(area.y),
            w: f64::from(area.width),
            h: f64::from(area.height),
            ..Place::default()
        };
        self.slide.add_picture(data, alpha, name, place, opacity);
    }

    fn rendered(&self, object: &StoredObject) -> PixelBuffer {
        crate::render::render_object(object, self.canvas)
            .map_or_else(|| PixelBuffer::new(self.width, self.height), |tiles| tiles.to_buffer(self.canvas))
    }

    fn emit(&mut self, object: &StoredObject, opacity: f64, name: &str, render: &dyn Fn(&Self) -> PixelBuffer) {
        if object.kind() == "group"
            && plain(object)
            && let Some(members) = object.current_members()
        {
            for m in members {
                self.emit(&m.object, opacity * m.opacity, &m.name, &|c: &Self| c.rendered(&m.object));
            }
            return;
        }
        if plain(object) {
            match object.kind() {
                "text" => {
                    if let Some(t) = object.text()
                        && t.arc == 0.0
                        && t.outline_width <= 0.0
                        && object.flips() == (false, false)
                        && !t.text.trim().is_empty()
                        && let Some(b) = object.box_()
                    {
                        let align = match t.align {
                            TextAlign::Center => "ctr",
                            TextAlign::Right => "r",
                            TextAlign::Left => "l",
                        };
                        let look = TextLook {
                            font_family: t.family.clone(),
                            font_size_px: t.size,
                            color: t.color,
                            bold: t.bold,
                            italic: t.italic,
                            align,
                            opacity,
                            line_spacing: t.line_spacing,
                            letter_spacing_px: t.letter_spacing,
                        };
                        self.slide.add_text(&t.text, name, place(b, (false, false)), &look);
                        return;
                    }
                }
                "shape" if self.try_shape(object, opacity, name) => return,
                "image" if self.image(object, opacity) => return,
                _ => {}
            }
        }
        let pixels = render(self);
        self.picture(&pixels, name, opacity);
    }

    fn try_shape(&mut self, object: &StoredObject, opacity: f64, name: &str) -> bool {
        let (Some(s), Some(b)) = (object.shape(), object.box_()) else { return false };
        if s.picture.is_some() || s.vertices.is_some() || s.gradient_border || s.nodes.is_some() || s.bend != 0.0 {
            return false;
        }
        let min = b.width.min(b.height).max(1e-6);
        let mut adjust: Vec<(&'static str, i32)> = Vec::new();
        let preset: Option<&'static str> = match s.kind {
            ShapeKind::Rectangle => {
                if s.corner_radius > 0.5 {
                    adjust.push((
                        "adj",
                        (s.corner_radius / min).clamp(0.0, 0.5).mul_add(100_000.0, 0.0).round_ties_even() as i32,
                    ));
                    Some("roundRect")
                } else {
                    Some("rect")
                }
            }
            ShapeKind::Ellipse => Some("ellipse"),
            ShapeKind::Triangle => Some("triangle"),
            ShapeKind::RightTriangle => Some("rtTriangle"),
            ShapeKind::Diamond => Some("diamond"),
            ShapeKind::Trapezoid => Some("trapezoid"),
            ShapeKind::Parallelogram => Some("parallelogram"),
            ShapeKind::Heart => Some("heart"),
            ShapeKind::Arrow => Some("rightArrow"),
            ShapeKind::DoubleArrow => Some("leftRightArrow"),
            ShapeKind::Chevron => Some("chevron"),
            ShapeKind::Lightning => Some("lightningBolt"),
            ShapeKind::Cloud => Some("cloud"),
            ShapeKind::Teardrop => Some("teardrop"),
            ShapeKind::Crescent => Some("moon"),
            ShapeKind::Pentagon => Some("pentagon"),
            ShapeKind::Hexagon => Some("hexagon"),
            ShapeKind::Octagon => Some("octagon"),
            ShapeKind::XMark => Some("mathMultiply"),
            ShapeKind::Cross => {
                adjust.push(("adj", percent((1.0 - s.inner) / 2.0)));
                Some("plus")
            }
            ShapeKind::Frame => {
                adjust.push(("adj1", percent((1.0 - s.inner) / 2.0)));
                Some("frame")
            }
            ShapeKind::Ring => {
                adjust.push(("adj", percent((1.0 - s.inner) / 2.0)));
                Some("donut")
            }
            ShapeKind::SpeechBubble => {
                Some(if s.corner_radius > 0.5 { "wedgeRoundRectCallout" } else { "wedgeRectCallout" })
            }
            ShapeKind::OvalBubble => Some("wedgeEllipseCallout"),
            ShapeKind::Star => match s.points {
                4 => Some("star4"),
                5 => Some("star5"),
                6 => Some("star6"),
                7 => Some("star7"),
                8 => Some("star8"),
                10 => Some("star10"),
                12 => Some("star12"),
                16 => Some("star16"),
                24 => Some("star24"),
                32 => Some("star32"),
                _ => None,
            },
            ShapeKind::Polygon => match s.points {
                5 => Some("pentagon"),
                6 => Some("hexagon"),
                7 => Some("heptagon"),
                8 => Some("octagon"),
                10 => Some("decagon"),
                12 => Some("dodecagon"),
                _ => None,
            },
            ShapeKind::Line => {
                let line = Place {
                    x: b.x,
                    y: b.y + b.height / 2.0,
                    w: b.width,
                    h: 0.0,
                    rotation: b.rotation,
                    ..Place::default()
                };
                let look = ShapeLook {
                    stroke: s.stroke.or(s.fill),
                    stroke_width: s.stroke_width.max(1.0),
                    opacity,
                    dash: dash_name(s.dash),
                    arrow_start: s.arrow_start,
                    arrow_end: s.arrow_end,
                    ..ShapeLook::default()
                };
                self.slide.add_shape("line", name, line, &look);
                return true;
            }
            _ => None,
        };
        let Some(preset) = preset else { return false };
        let look = ShapeLook {
            fill: s.fill,
            stroke: s.stroke,
            stroke_width: s.stroke_width,
            opacity,
            dash: dash_name(s.dash),
            adjust,
            ..ShapeLook::default()
        };
        self.slide.add_shape(preset, name, place(b, object.flips()), &look);
        true
    }

    fn image(&mut self, object: &StoredObject, opacity: f64) -> bool {
        let (Some(bytes), Some(b)) = (object.picture(), object.box_()) else { return false };
        let Some(source) = pictures::decoded(bytes) else { return false };
        let (data, alpha) = self.picture_data(&source);
        self.slide.add_picture(data, alpha, object.image_name(), place(b, object.flips()), opacity);
        true
    }
}

/// One page as a slide.
pub fn build_slide(page: &Document, options: EditableOptions) -> PptxSlide {
    let mut slide = PptxSlide::new();
    let canvas = page.bounds();
    let mut ctx = Context { slide: &mut slide, width: page.width(), height: page.height(), options, canvas };
    // Looks that PowerPoint cannot combine the way this program does: keep the page as it is, as one picture.
    let blended = page
        .layers()
        .iter()
        .any(|l| l.visible && l.opacity() > 0.0 && (l.is_adjustment() || l.blend != crate::imaging::BlendMode::Normal));
    if blended {
        ctx.picture(&compositor::flatten(page), "Page", 1.0);
        return slide;
    }
    for layer in page.layers() {
        if !layer.visible || layer.opacity() <= 0.0 {
            continue;
        }
        let opacity = layer.opacity();
        match (layer.object(), layer.effective_mask()) {
            (Some(object), None) => {
                let pixels = layer.pixels().to_buffer(canvas);
                ctx.emit(object, opacity, &layer.name, &|_: &Context<'_>| pixels.clone());
            }
            (None, None) if uniform(&layer.pixels().to_buffer(canvas)).is_some() => {
                if let Some(paper) = uniform(&layer.pixels().to_buffer(canvas)) {
                    ctx.fill(paper, &layer.name, opacity);
                }
            }
            (_, mask) => {
                let baked = match mask {
                    Some(m) => bake_mask(layer.pixels(), m),
                    None => layer.pixels().clone(),
                };
                ctx.picture(&baked.to_buffer(canvas), &layer.name, opacity);
            }
        }
    }
    slide
}

/// A presentation of every page, text and simple shapes editable.
pub fn build_editable(pages: &[&Document], options: EditableOptions) -> Result<Vec<u8>, IoError> {
    build_editable_with(pages, options, &[])
}

/// Like `build_editable`, with a slide transition for each slide.
pub fn build_editable_with(
    pages: &[&Document],
    options: EditableOptions,
    transitions: &[crate::document::Transition],
) -> Result<Vec<u8>, IoError> {
    let Some(first) = pages.first() else {
        return Err(IoError::CannotSave("A presentation needs at least one slide.".into()));
    };
    let mut slides: Vec<PptxSlide> = pages.iter().map(|p| build_slide(p, options)).collect();
    for (slide, t) in slides.iter_mut().zip(transitions) {
        slide.set_transition(t);
    }
    build_slides(&slides, first.width(), first.height())
}

/// Saves the editable presentation.
pub fn save_editable(
    pages: &[&Document],
    path: impl AsRef<std::path::Path>,
    options: EditableOptions,
) -> Result<(), IoError> {
    save_editable_with(pages, path, options, &[])
}

/// Like `save_editable`, with a slide transition for each slide.
pub fn save_editable_with(
    pages: &[&Document],
    path: impl AsRef<std::path::Path>,
    options: EditableOptions,
    transitions: &[crate::document::Transition],
) -> Result<(), IoError> {
    write_file(path.as_ref(), build_editable_with(pages, options, transitions)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Layer;
    use crate::objects::{ObjectBox, ShapeData};
    use std::io::Read;

    fn slide_xml(bytes: &[u8], n: usize) -> String {
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("a zip");
        let mut text = String::new();
        zip.by_name(&format!("ppt/slides/slide{n}.xml")).expect("slide").read_to_string(&mut text).expect("text");
        text
    }

    #[test]
    fn text_and_shapes_stay_editable_and_painted_layers_become_pictures() {
        let mut doc = Document::blank(400, 300, ColorRgba::WHITE);
        let canvas = doc.bounds();
        let star = ShapeData::new(ShapeKind::Star, Some(ColorRgba::rgb(250, 200, 0)), None, 0.0);
        doc.push_layer(crate::document::object_ops::object_layer(
            "Star",
            StoredObject::new_shape(ObjectBox::new(20.0, 20.0, 100.0, 100.0, 0.0), &star),
            canvas,
        ));
        let text = crate::text::TextData::new("Hello <world>", "Segoe UI", 32.0, ColorRgba::rgb(0, 0, 0));
        doc.push_layer(crate::document::object_ops::object_layer(
            "Title",
            StoredObject::new_text(ObjectBox::new(30.0, 150.0, 300.0, 50.0, 0.0), &text),
            canvas,
        ));
        let mut painted = Layer::new("Paint");
        painted.pixels_mut().set_pixel(200, 200, ColorRgba::rgb(255, 0, 0));
        doc.push_layer(painted);
        let bytes = build_editable(&[&doc], EditableOptions::default()).expect("built");
        let xml = slide_xml(&bytes, 1);
        assert!(xml.contains("prst=\"star5\""), "the star is a PowerPoint shape");
        assert!(xml.contains("<a:t>Hello &lt;world&gt;</a:t>"), "the text is a text box");
        assert!(xml.contains("<p:pic>"), "painted pixels are a picture");
        assert!(xml.contains("prst=\"rect\"") && xml.contains("FFFFFF"), "the white background is a rectangle");
    }
}
