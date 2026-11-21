//! Builds a ready-made picture (a badge, a graphic, a diagram) as a group of shapes and text, in a design space
//! `0..100` wide that is scaled and centered on the canvas (the C# `Badges`, `Graphics.Maker` and `SmartArt.Board`).

use crate::imaging::{ColorRgba, PointD};
use crate::objects::{BendStyle, CurveNode, GroupMember, ObjectBox, ShapeData, ShapeKind, StoredObject, StrokeStyle};
use crate::render::measure_text;
use crate::text::{TextAlign, TextData};

pub fn c(r: u8, g: u8, b: u8) -> ColorRgba {
    ColorRgba::rgb(r, g, b)
}

/// One shape of the picture, still in design units; change it with the methods before the next one is added.
pub struct Spec {
    name: String,
    b: ObjectBox,
    data: ShapeData,
    s: f64,
}

impl Spec {
    pub fn named(&mut self, name: &str) -> &mut Self {
        self.name = name.to_owned();
        self
    }

    pub fn fill(&mut self, color: ColorRgba) -> &mut Self {
        self.data.fill = Some(color);
        self
    }

    /// A border of `width` design units.
    pub fn stroke(&mut self, color: ColorRgba, width: impl Into<f64>) -> &mut Self {
        self.data.stroke = Some(color);
        self.data.stroke_width = width.into() * self.s;
        self
    }

    pub fn radius(&mut self, r: impl Into<f64>) -> &mut Self {
        self.data.corner_radius = r.into() * self.s;
        self
    }

    pub fn rotate(&mut self, degrees: impl Into<f64>) -> &mut Self {
        self.b.rotation = degrees.into();
        self
    }

    /// A line with dashes of the given length and gap (design units).
    pub fn dashed(&mut self, length: impl Into<f64>, gap: impl Into<f64>) -> &mut Self {
        self.data.dash = StrokeStyle::Dashed;
        self.data.dash_length = length.into() * self.s;
        self.data.dash_gap = gap.into() * self.s;
        self
    }

    pub fn dotted(&mut self, gap: impl Into<f64>) -> &mut Self {
        self.data.dash = StrokeStyle::Dotted;
        self.data.dash_gap = gap.into() * self.s;
        self
    }

    pub fn arrow(&mut self) -> &mut Self {
        self.data.arrow_end = true;
        self
    }

    pub fn bend(&mut self, amount: impl Into<f64>) -> &mut Self {
        self.data.bend = amount.into();
        self.data.bend_style = BendStyle::Arc;
        self
    }
}

enum Part {
    Shape(Spec),
    Object(GroupMember),
}

pub struct Maker {
    s: f64,
    ox: f64,
    oy: f64,
    parts: Vec<Part>,
}

impl Maker {
    /// A design `design_height` units tall and 100 wide, drawn `width` pixels wide in the middle of the canvas.
    pub fn new(canvas_w: f64, canvas_h: f64, width: f64, design_height: f64) -> Self {
        let s = width / 100.0;
        Self { s, ox: (canvas_w - width) / 2.0, oy: (canvas_h - design_height * s) / 2.0, parts: Vec::new() }
    }

    fn rect(&self, x: f64, y: f64, w: f64, h: f64, rotation: f64) -> ObjectBox {
        ObjectBox::new(
            self.ox + x * self.s,
            self.oy + y * self.s,
            (w * self.s).max(0.5),
            (h * self.s).max(0.5),
            rotation,
        )
    }

    fn push(&mut self, name: &str, b: ObjectBox, data: ShapeData) -> &mut Spec {
        self.parts.push(Part::Shape(Spec { name: name.to_owned(), b, data, s: self.s }));
        match self.parts.last_mut() {
            Some(Part::Shape(spec)) => spec,
            _ => unreachable!("a shape was just added"),
        }
    }

    /// A shape in the box (x, y, w, h) of the design; set its colors with the returned [`Spec`].
    pub fn shape(
        &mut self,
        name: &str,
        kind: ShapeKind,
        x: impl Into<f64>,
        y: impl Into<f64>,
        w: impl Into<f64>,
        h: impl Into<f64>,
    ) -> &mut Spec {
        let b = self.rect(x.into(), y.into(), w.into(), h.into(), 0.0);
        self.push(name, b, ShapeData::new(kind, None, None, 0.0))
    }

    /// A straight line through (cx, cy) of the given length and angle (degrees).
    pub fn line(
        &mut self,
        cx: impl Into<f64>,
        cy: impl Into<f64>,
        length: impl Into<f64>,
        angle: impl Into<f64>,
        color: ColorRgba,
        width: impl Into<f64>,
    ) -> &mut Spec {
        let (cx, cy, length, angle, width) = (cx.into(), cy.into(), length.into(), angle.into(), width.into());
        let thickness = (width * self.s).max(1.0);
        let b = ObjectBox::new(
            self.ox + (cx - length / 2.0) * self.s,
            self.oy + cy * self.s - thickness / 2.0,
            length * self.s,
            thickness,
            angle,
        );
        let mut data = ShapeData::new(ShapeKind::Line, None, Some(color), thickness);
        data.stroke_width = thickness;
        self.push("Line", b, data)
    }

    /// A curve through nodes given in the unit box (0..1) of (x, y, w, h).
    #[allow(clippy::too_many_arguments)]
    pub fn path(
        &mut self,
        name: &str,
        x: impl Into<f64>,
        y: impl Into<f64>,
        w: impl Into<f64>,
        h: impl Into<f64>,
        nodes: Vec<CurveNode>,
        closed: bool,
    ) -> &mut Spec {
        let b = self.rect(x.into(), y.into(), w.into(), h.into(), 0.0);
        let mut data = ShapeData::new(ShapeKind::Path, None, None, 0.0);
        data.nodes = Some(nodes);
        data.closed = closed;
        self.push(name, b, data)
    }

    /// A flat-sided shape through the given points (design units).
    pub fn poly(&mut self, name: &str, color: ColorRgba, pts: &[(f64, f64)]) {
        let min_x = pts.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let max_x = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        let min_y = pts.iter().map(|p| p.1).fold(f64::MAX, f64::min);
        let max_y = pts.iter().map(|p| p.1).fold(f64::MIN, f64::max);
        let (w, h) = ((max_x - min_x).max(0.01), (max_y - min_y).max(0.01));
        let nodes = pts.iter().map(|p| CurveNode::corner(PointD::new((p.0 - min_x) / w, (p.1 - min_y) / h))).collect();
        self.path(name, min_x, min_y, w, h, nodes, true).fill(color);
    }

    #[allow(clippy::too_many_arguments)]
    fn text_object(&mut self, text: &str, size: f64, color: ColorRgba, bold: bool, center: bool, cx: f64, cy: f64) {
        let mut data = TextData::new(text, "Segoe UI", size, color);
        data.bold = bold;
        if center {
            data.align = TextAlign::Center;
        }
        let (w, h) = measure_text(&data);
        let b = ObjectBox::new(self.ox + cx * self.s - w / 2.0, self.oy + cy * self.s - h / 2.0, w, h, 0.0);
        let object = StoredObject::new_text(b, &data);
        self.parts.push(Part::Object(GroupMember { object, name: text.replace('\n', " "), opacity: 1.0 }));
    }

    /// Bold words centered at (cx, cy); `size` is in design units.
    pub fn badge_text(
        &mut self,
        text: &str,
        cx: impl Into<f64>,
        cy: impl Into<f64>,
        size: impl Into<f64>,
        color: ColorRgba,
    ) {
        self.text_object(text, (size.into() * self.s).max(6.0), color, true, false, cx.into(), cy.into());
    }

    /// Words centered at a place; long words wrap and the letters shrink until everything fits in the given room.
    #[allow(clippy::too_many_arguments)]
    pub fn fit_text(
        &mut self,
        text: &str,
        cx: f64,
        cy: f64,
        max_w: f64,
        max_h: f64,
        size: f64,
        color: ColorRgba,
        bold: bool,
    ) {
        let text = text.trim();
        if text.is_empty() {
            return;
        }
        let (px_w, px_h) = (max_w * self.s, max_h * self.s);
        let mut px = size * self.s;
        loop {
            let wrapped = wrap(text, px_w, px, bold);
            let mut data = TextData::new(&wrapped, "Segoe UI", px.max(5.0), color);
            data.bold = bold;
            data.align = TextAlign::Center;
            let (w, h) = measure_text(&data);
            if (w <= px_w && h <= px_h) || px <= 6.0 {
                self.text_object(&wrapped, px.max(5.0), color, bold, true, cx, cy);
                // The layer name is the words without the line breaks.
                return;
            }
            px *= 0.9;
        }
    }

    /// The finished group, or `None` when nothing was added.
    pub fn done(self) -> Option<StoredObject> {
        let members: Vec<GroupMember> = self
            .parts
            .into_iter()
            .map(|p| match p {
                Part::Shape(spec) => {
                    GroupMember { object: StoredObject::new_shape(spec.b, &spec.data), name: spec.name, opacity: 1.0 }
                }
                Part::Object(m) => m,
            })
            .collect();
        StoredObject::new_group(&members)
    }
}

fn width_of(text: &str, size: f64, bold: bool) -> f64 {
    let mut data = TextData::new(text, "Segoe UI", size.max(5.0), ColorRgba::WHITE);
    data.bold = bold;
    measure_text(&data).0
}

/// Breaks the text into lines no wider than `max_pixels` (words stay whole).
fn wrap(text: &str, max_pixels: f64, size: f64, bold: bool) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split(' ').filter(|w| !w.is_empty()) {
        let trial = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
        if !line.is_empty() && width_of(&trial, size, bold) > max_pixels {
            lines.push(std::mem::take(&mut line));
            line = word.to_owned();
        } else {
            line = trial;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}
