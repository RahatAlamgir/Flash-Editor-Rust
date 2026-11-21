//! One tutorial page and the pieces it is made of (C# `DemoProject.Page`): header, numbered steps, notes, captions.

use crate::document::{Document, Page as ProjectPage, object_ops};
use crate::imaging::{ColorRgba, PixelRect};
use crate::objects::{Looks, ObjectBox, ShadowStyle, ShapeData, ShapeKind, StoredObject};
use crate::render::measure_text;
use crate::text::{TextAlign, TextData};

pub const W: f64 = 1280.0;
pub const H: f64 = 720.0;

pub fn c(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

pub fn ca(r: u8, g: u8, b: u8, a: u8) -> ColorRgba {
    ColorRgba::new(r, g, b, a)
}

pub const INK: ColorRgba = ColorRgba::rgb(30, 41, 59);
pub const SUBTLE: ColorRgba = ColorRgba::rgb(100, 116, 139);
pub const WHITE: ColorRgba = ColorRgba::WHITE;

pub struct Page {
    name: String,
    background: ColorRgba,
    items: Vec<(String, StoredObject)>,
}

impl Page {
    pub fn new(name: &str, background: ColorRgba) -> Self {
        Self { name: name.to_owned(), background, items: Vec::new() }
    }

    pub fn add(&mut self, object: StoredObject, name: &str) -> &mut StoredObject {
        self.items.push((name.to_owned(), object));
        &mut self.items.last_mut().expect("just added").1
    }

    pub fn finish(self) -> ProjectPage {
        let mut doc = Document::blank(W as u32, H as u32, self.background);
        let canvas = PixelRect::new(0, 0, W as i32, H as i32);
        for (name, object) in self.items {
            let layer = object_ops::object_layer(name, object, canvas);
            doc.push_layer(layer);
        }
        ProjectPage::new(self.name, doc)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn shape(
        &mut self,
        name: &str,
        kind: ShapeKind,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        fill: Option<ColorRgba>,
    ) -> &mut StoredObject {
        let object = StoredObject::new_shape(ObjectBox::new(x, y, w, h, 0.0), &ShapeData::new(kind, fill, None, 0.0));
        self.add(object, name)
    }

    /// Text with its top-left corner at (x, y), or centered on x when `center`. Lines are separated by line breaks.
    #[allow(clippy::too_many_arguments)]
    pub fn text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        size: f64,
        color: ColorRgba,
        bold: bool,
        center: bool,
    ) -> &mut StoredObject {
        self.text_with(text, x, y, size, color, bold, center, 1.15, |_| {})
    }

    #[allow(clippy::too_many_arguments)]
    pub fn text_with(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        size: f64,
        color: ColorRgba,
        bold: bool,
        center: bool,
        line_spacing: f64,
        more: impl FnOnce(&mut TextData),
    ) -> &mut StoredObject {
        let mut t = TextData::new(text, "Segoe UI", size, color);
        t.bold = bold;
        t.align = if center { TextAlign::Center } else { TextAlign::Left };
        t.line_spacing = line_spacing;
        more(&mut t);
        let (w, h) = measure_text(&t);
        let object = StoredObject::new_text(ObjectBox::new(if center { x - w / 2.0 } else { x }, y, w, h, 0.0), &t);
        let name = text.split('\n').next().unwrap_or("").trim().to_owned();
        self.add(object, &name)
    }

    /// The colored header of a page: the page number, the title and one line about what it teaches.
    pub fn header(&mut self, number: usize, total: usize, title: &str, lesson: &str, accent: ColorRgba) {
        self.shape("Header", ShapeKind::Rectangle, 0.0, 0.0, W, 112.0, Some(accent));
        self.shape("Header shade", ShapeKind::Rectangle, 0.0, 100.0, W, 12.0, Some(ca(255, 255, 255, 38)));
        self.text(title, 56.0, 18.0, 40.0, WHITE, true, false);
        self.text(lesson, 58.0, 68.0, 19.0, ca(255, 255, 255, 225), false, false);
        let chip =
            self.shape("Page chip", ShapeKind::Rectangle, W - 150.0, 34.0, 104.0, 40.0, Some(ca(255, 255, 255, 50)));
        set_radius(chip, 20.0);
        self.text(&format!("{number} / {total}"), W - 98.0, 41.0, 20.0, WHITE, true, true);
    }

    /// A list of numbered steps down the left side.
    pub fn steps(&mut self, steps: &[&str], accent: ColorRgba, top: f64, line_gap: f64) {
        for (i, step) in steps.iter().enumerate() {
            let y = top + i as f64 * line_gap;
            self.shape(&format!("Step {}", i + 1), ShapeKind::Ellipse, 56.0, y, 34.0, 34.0, Some(accent));
            self.text(&(i + 1).to_string(), 73.0, y + 4.0, 19.0, WHITE, true, true);
            self.text_with(step, 106.0, y + 1.0, 19.0, INK, false, false, 1.2, |_| {});
        }
    }

    /// A little yellow note with a hint.
    pub fn note(&mut self, text: &str, x: f64, y: f64, w: f64, h: f64) {
        let o = self.shape("Note", ShapeKind::Rectangle, x, y, w, h, Some(c(254, 240, 138)));
        set_radius(o, 8.0);
        shadow(o, ShadowStyle { color: ca(0, 0, 0, 55), distance: 5.0, angle: 60.0, blur: 10.0, size: 1.0 });
        self.text(text, x + 16.0, y + 12.0, 17.0, c(113, 63, 18), false, false);
    }

    pub fn caption(&mut self, text: &str, cx: f64, y: f64, size: f64) {
        self.text(text, cx, y, size, SUBTLE, false, true);
    }

    /// Puts a group built centered on the canvas at a new center.
    pub fn place(&mut self, group: Option<StoredObject>, cx: f64, cy: f64, name: &str) {
        if let Some(g) = group {
            self.add(g.offset(cx - W / 2.0, cy - H / 2.0), name);
        }
    }
}

pub fn set_radius(o: &mut StoredObject, r: f64) {
    if let Some(mut s) = o.shape() {
        s.corner_radius = r;
        o.set_shape(&s);
    }
}

pub fn edit_shape(o: &mut StoredObject, f: impl FnOnce(&mut ShapeData)) {
    if let Some(mut s) = o.shape() {
        f(&mut s);
        o.set_shape(&s);
    }
}

pub fn shadow(o: &mut StoredObject, s: ShadowStyle) {
    o.set_looks(&Looks { shadow: Some(s), ..o.looks() });
}

pub fn looks(o: &mut StoredObject, f: impl FnOnce(&mut Looks)) {
    let mut l = o.looks();
    f(&mut l);
    o.set_looks(&l);
}
