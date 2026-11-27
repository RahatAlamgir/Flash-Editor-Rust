//! "Start from" layouts for a new picture (C# `DocumentTemplates`): a background and a few editable shapes and texts, laid out
//! for any canvas size. Everything stays editable (change the words, colors, gradient...).

use crate::document::{Document, Layer, object_ops};
use crate::imaging::{ColorRgba, PointD};
use crate::objects::{
    GRADIENT_PRESETS, GradientPreset, Looks, ObjectBox, ShadowStyle, ShapeData, ShapeKind, StoredObject, ThreeDStyle,
};
use crate::render::measure_text;
use crate::text::{TextAlign, TextData};

pub const BLANK: &str = "Blank";

/// The layouts in the order of the list.
pub const NAMES: [&str; 6] = [BLANK, "Title slide", "Quote card", "Sale poster", "Photo with caption", "Announcement"];

fn rgb(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

fn gradient_named(name: &str) -> &'static GradientPreset {
    GRADIENT_PRESETS.iter().find(|p| p.name == name).unwrap_or(&GRADIENT_PRESETS[0])
}

/// A text centered at (cx, cy); its box is fitted to the text.
fn text(content: &str, cx: f64, cy: f64, size: f64, color: ColorRgba, bold: bool) -> StoredObject {
    let mut data = TextData::new(content, "Segoe UI", size, color);
    data.bold = bold;
    data.align = TextAlign::Center;
    let (w, h) = measure_text(&data);
    StoredObject::new_text(ObjectBox::from_center(PointD::new(cx, cy), w, h, 0.0), &data)
}

fn rectangle(x: f64, y: f64, w: f64, h: f64, fill: ColorRgba, radius: f64) -> StoredObject {
    let mut data = ShapeData::new(ShapeKind::Rectangle, Some(fill), None, 0.0);
    data.corner_radius = radius;
    StoredObject::new_shape(ObjectBox::new(x, y, w, h, 0.0), &data)
}

fn background(w: f64, h: f64, look: &str) -> StoredObject {
    let preset = gradient_named(look);
    let mut object = rectangle(0.0, 0.0, w, h, preset.colors[0], 0.0);
    object.set_gradient(Some(&preset.create()));
    object
}

fn with(mut object: StoredObject, looks: Looks) -> StoredObject {
    object.set_looks(&looks);
    object
}

/// Adds the layout's layers to a new document (a blank one with the background already in place). Unknown names do nothing.
pub fn apply(document: &mut Document, template: &str) {
    if template.is_empty() || template == BLANK || !NAMES.contains(&template) {
        return;
    }
    let (w, h) = (f64::from(document.width()), f64::from(document.height()));
    let s = w.min(h);
    let canvas = document.bounds();
    let white = rgb(255, 255, 255);
    let dark = rgb(30, 41, 59);
    let shadow =
        ShadowStyle { color: ColorRgba::new(0, 0, 0, 90), distance: s * 0.01, angle: 90.0, blur: s * 0.02, size: 1.0 };
    let shadow_look = Looks { shadow: Some(shadow), ..Looks::default() };
    let mut parts: Vec<(&str, StoredObject)> = Vec::new();
    match template {
        "Title slide" => {
            parts.push(("Background", background(w, h, "Ocean")));
            parts.push(("Title", with(text("Your Title Here", w / 2.0, h * 0.44, s * 0.11, white, true), shadow_look)));
            parts.push(("Line", rectangle(w / 2.0 - w * 0.08, h * 0.54, w * 0.16, (s * 0.008).max(3.0), white, 0.0)));
            parts.push((
                "Subtitle",
                text("A short line that explains it", w / 2.0, h * 0.62, s * 0.045, rgb(235, 245, 255), false),
            ));
        }
        "Quote card" => {
            parts.push(("Background", background(w, h, "Night")));
            parts.push((
                "Quote mark",
                text("\u{201C}", w / 2.0, h * 0.25, s * 0.28, ColorRgba::new(255, 255, 255, 120), true),
            ));
            parts.push((
                "Quote",
                text("Write a short quote here,\nin your own words.", w / 2.0, h * 0.5, s * 0.07, white, true),
            ));
            parts.push(("Author", text("\u{2014} Name", w / 2.0, h * 0.76, s * 0.04, rgb(200, 205, 255), false)));
        }
        "Sale poster" => {
            parts.push(("Background", background(w, h, "Fire")));
            let three_d = ThreeDStyle { depth: s * 0.025, angle: 90.0, color: Some(rgb(120, 20, 20)), bevel: 0.5 };
            let sale = with(
                text("SALE", w / 2.0, h * 0.38, s * 0.26, white, true),
                Looks { shadow: Some(shadow), three_d: Some(three_d), ..Looks::default() },
            );
            parts.push(("Sale", sale));
            parts.push(("Offer", text("UP TO 50% OFF", w / 2.0, h * 0.62, s * 0.07, white, true)));
            parts.push(("Button", rectangle(w * 0.25, h * 0.74, w * 0.5, s * 0.1, white, s * 0.05)));
            parts.push((
                "Button text",
                text("Shop now", w / 2.0, h * 0.74 + s * 0.05, s * 0.045, rgb(200, 30, 30), true),
            ));
        }
        "Photo with caption" => {
            parts.push(("Background", rectangle(0.0, 0.0, w, h, white, 0.0)));
            let (pw, ph) = (w * 0.8, h * 0.62);
            let (px, py) = ((w - pw) / 2.0, h * 0.08);
            parts.push((
                "Photo (place your picture here)",
                with(rectangle(px, py, pw, ph, rgb(226, 232, 240), s * 0.02), shadow_look),
            ));
            parts.push(("Caption", text("Caption goes here", w / 2.0, py + ph + h * 0.1, s * 0.055, dark, true)));
            parts.push((
                "Details",
                text(
                    "A few words about the picture",
                    w / 2.0,
                    py + ph + h * 0.19,
                    s * 0.035,
                    rgb(100, 116, 139),
                    false,
                ),
            ));
        }
        "Announcement" => {
            parts.push(("Background", background(w, h, "Candy")));
            parts.push((
                "Card",
                with(
                    rectangle(w * 0.1, h * 0.14, w * 0.8, h * 0.72, ColorRgba::new(255, 255, 255, 235), s * 0.04),
                    shadow_look,
                ),
            ));
            parts.push(("Headline", text("BIG NEWS", w / 2.0, h * 0.34, s * 0.1, rgb(147, 51, 234), true)));
            parts.push((
                "Text",
                text("Tell people what is happening,\nwhen and where.", w / 2.0, h * 0.54, s * 0.045, dark, false),
            ));
            parts
                .push(("Details", text("Date  \u{2022}  Place", w / 2.0, h * 0.7, s * 0.04, rgb(100, 116, 139), true)));
        }
        _ => return,
    }
    let mut layers: Vec<Layer> = document.layers().to_vec();
    for (name, object) in parts {
        layers.push(object_ops::object_layer(name, object, canvas));
    }
    let last = layers.last().map(Layer::id);
    document.set_layers(layers, last);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_layout_adds_editable_objects_for_any_canvas_size() {
        for name in &NAMES[1..] {
            for (w, h) in [(1080, 1080), (1920, 1080), (400, 900)] {
                let mut doc = Document::blank(w, h, ColorRgba::WHITE);
                apply(&mut doc, name);
                assert!(doc.layers().len() >= 4, "{name}");
                assert!(
                    doc.layers()[1..].iter().all(|l| l.object().is_some()),
                    "{name}: every added layer is an object"
                );
                assert!(doc.layers().iter().any(|l| l.object().is_some_and(|o| o.kind() == "text")), "{name}");
            }
        }
        let mut blank = Document::blank(100, 100, ColorRgba::WHITE);
        apply(&mut blank, BLANK);
        apply(&mut blank, "nope");
        assert_eq!(blank.layers().len(), 1);
    }

    #[test]
    fn the_title_slide_puts_its_title_in_the_middle_with_a_shadow() {
        let mut doc = Document::blank(1000, 500, ColorRgba::WHITE);
        apply(&mut doc, "Title slide");
        let title = doc.layers().iter().find(|l| l.name == "Title").and_then(|l| l.object()).expect("title");
        let b = title.box_().expect("box");
        assert!((b.center().x - 500.0).abs() < 1.0 && (b.center().y - 220.0).abs() < 1.0);
        assert!(title.looks().shadow.is_some());
        assert_eq!(title.text().expect("text").text, "Your Title Here");
    }
}
