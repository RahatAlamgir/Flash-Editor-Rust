//! Shapes (C# `ShapeObject`, `ShapeKind`, `CurveNode`, `PictureFill`, `GradientStyle`): the typed view of a
//! `"kind": "shape"` description.

use serde_json::{Map, Value};

use super::json::{self, number};
use crate::imaging::{ColorRgba, PointD};

macro_rules! shape_kinds {
    ($($kind:ident = $name:literal),* $(,)?) => {
        /// The shapes, numbered like the C# `ShapeKind` (the number is what project files store).
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum ShapeKind { $($kind),* }

        impl ShapeKind {
            pub const ALL: &[ShapeKind] = &[$(ShapeKind::$kind),*];

            /// The name shown in menus and used for new layers.
            pub const fn name(self) -> &'static str {
                match self { $(ShapeKind::$kind => $name),* }
            }
        }
    };
}

shape_kinds! {
    Rectangle = "Rectangle", Ellipse = "Ellipse", Triangle = "Triangle", Star = "Star", Polygon = "Polygon",
    Heart = "Heart", Arrow = "Arrow", SpeechBubble = "Speech bubble", Line = "Line", Ring = "Ring",
    HalfRing = "Half ring", HalfCircle = "Half circle", QuarterCircle = "Quarter circle", Pie = "Pie",
    Crescent = "Crescent", Diamond = "Diamond", RightTriangle = "Right triangle", Trapezoid = "Trapezoid",
    Parallelogram = "Parallelogram", Cross = "Cross", Frame = "Frame", Chevron = "Chevron",
    Lightning = "Lightning bolt", Cloud = "Cloud", Teardrop = "Teardrop", Pentagon = "Pentagon",
    Hexagon = "Hexagon", Octagon = "Octagon", Sparkle = "Sparkle", Burst = "Burst", DoubleArrow = "Double arrow",
    Check = "Check", XMark = "X mark", Home = "Home", Tag = "Tag", Shield = "Shield", OvalBubble = "Oval bubble",
    Path = "Curve",
}

impl ShapeKind {
    pub fn from_number(n: u64) -> Option<Self> {
        Self::ALL.get(n as usize).copied()
    }

    pub fn number(self) -> u64 {
        Self::ALL.iter().position(|k| *k == self).unwrap_or(0) as u64
    }

    /// Shapes made of straight sides, whose corners can be dragged (C# `ShapeVertices.IsPolygonal`).
    pub fn is_polygonal(self) -> bool {
        use ShapeKind::*;
        matches!(
            self,
            Rectangle
                | Triangle
                | Star
                | Polygon
                | Arrow
                | Diamond
                | RightTriangle
                | Trapezoid
                | Parallelogram
                | Cross
                | Chevron
                | Lightning
                | Pentagon
                | Hexagon
                | Octagon
                | Sparkle
                | Burst
                | DoubleArrow
                | Check
                | XMark
                | Home
                | Tag
        )
    }

    /// Shapes that use the "Inner" setting.
    pub fn has_inner(self) -> bool {
        use ShapeKind::*;
        matches!(self, Ring | HalfRing | Pie | Crescent | Cross | Frame)
    }

    /// What the "Inner" setting means for this shape.
    pub fn inner_label(self) -> &'static str {
        match self {
            ShapeKind::Crescent => "Bite",
            ShapeKind::Cross => "Arms",
            _ => "Hole",
        }
    }
}

/// How a border or line is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StrokeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// How a bent line curves.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BendStyle {
    #[default]
    Arc,
    Wave,
}

/// One point of a curve: where it is and its two handles (a handle on the point = no bend on that side).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CurveNode {
    pub point: PointD,
    pub handle_in: PointD,
    pub handle_out: PointD,
}

impl CurveNode {
    pub fn corner(p: PointD) -> Self {
        Self { point: p, handle_in: p, handle_out: p }
    }

    pub fn has_in(&self) -> bool {
        self.handle_in != self.point
    }

    pub fn has_out(&self) -> bool {
        self.handle_out != self.point
    }

    pub fn mapped(&self, f: impl Fn(PointD) -> PointD) -> Self {
        Self { point: f(self.point), handle_in: f(self.handle_in), handle_out: f(self.handle_out) }
    }
}

/// A picture shown inside a shape (a frame).
#[derive(Clone, Debug, PartialEq)]
pub struct PictureFill {
    /// The picture's file name in the project (`images/N.png`).
    pub image: String,
    pub name: String,
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}

impl PictureFill {
    /// Where the picture lies in the shape's own space, for a picture of `iw` x `ih` pixels (C# `Place`).
    pub fn place(&self, iw: f64, ih: f64, sw: f64, sh: f64) -> (f64, f64, f64, f64) {
        let scale = (sw / iw.max(1.0)).max(sh / ih.max(1.0)) * self.zoom.clamp(0.25, 5.0);
        let (w, h) = (iw * scale, ih * scale);
        let x = (sw - w) / 2.0 + self.pan_x.clamp(-1.0, 1.0) * (w - sw).abs() / 2.0;
        let y = (sh - h) / 2.0 + self.pan_y.clamp(-1.0, 1.0) * (h - sh).abs() / 2.0;
        (x, y, w, h)
    }
}

/// A color that fades through two or more colors (C# `GradientStyle`).
#[derive(Clone, Debug, PartialEq)]
pub struct Gradient {
    /// (color, position 0..1), in any order.
    pub stops: Vec<(ColorRgba, f64)>,
    /// Degrees: 0 = left to right, 90 = top to bottom.
    pub angle: f64,
    pub radial: bool,
}

impl Gradient {
    pub fn from_json(v: &Value) -> Option<Self> {
        let map = v.as_object()?;
        let stops: Vec<(ColorRgba, f64)> = map
            .get("Stops")?
            .as_array()?
            .iter()
            .filter_map(|s| Some((json::color(s.get("Color")?)?, s.get("Position")?.as_f64()?)))
            .collect();
        (stops.len() >= 2).then(|| Self {
            stops,
            angle: json::f64_of(map, "Angle", 90.0),
            radial: json::bool_of(map, "Radial"),
        })
    }

    pub fn sorted(&self) -> Vec<(ColorRgba, f64)> {
        let mut s = self.stops.clone();
        s.sort_by(|a, b| a.1.total_cmp(&b.1));
        s
    }
}

/// Everything a shape description says (C# `ShapeObject` without the box and the looks).
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeData {
    pub kind: ShapeKind,
    pub fill: Option<ColorRgba>,
    pub stroke: Option<ColorRgba>,
    pub stroke_width: f64,
    pub corner_radius: f64,
    pub points: i32,
    pub inner: f64,
    pub sweep: f64,
    pub vertices: Option<Vec<PointD>>,
    pub gradient_border: bool,
    pub dash: StrokeStyle,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub dash_length: f64,
    pub dash_gap: f64,
    pub nodes: Option<Vec<CurveNode>>,
    pub closed: bool,
    pub bend: f64,
    pub bend_style: BendStyle,
    pub picture: Option<PictureFill>,
}

impl ShapeData {
    /// A new shape with the C# defaults.
    pub fn new(kind: ShapeKind, fill: Option<ColorRgba>, stroke: Option<ColorRgba>, stroke_width: f64) -> Self {
        Self {
            kind,
            fill,
            stroke,
            stroke_width,
            corner_radius: 0.0,
            points: 5,
            inner: 0.5,
            sweep: 270.0,
            vertices: None,
            gradient_border: false,
            dash: StrokeStyle::Solid,
            arrow_start: false,
            arrow_end: false,
            dash_length: 0.0,
            dash_gap: 0.0,
            nodes: None,
            closed: false,
            bend: 0.0,
            bend_style: BendStyle::Arc,
            picture: None,
        }
    }

    pub fn is_line(&self) -> bool {
        self.kind == ShapeKind::Line
    }

    pub fn can_fill(&self) -> bool {
        !self.is_line()
    }

    pub fn from_json(map: &Map<String, Value>) -> Option<Self> {
        let kind = ShapeKind::from_number(map.get("Shape")?.as_u64()?)?;
        let node = |v: &Value| -> Option<CurveNode> {
            Some(CurveNode {
                point: json::point(v.get("Point")?)?,
                handle_in: json::point(v.get("HandleIn")?)?,
                handle_out: json::point(v.get("HandleOut")?)?,
            })
        };
        let picture = map.get("Picture").and_then(Value::as_object).and_then(|p| {
            Some(PictureFill {
                image: json::str_of(p, "Image")?.to_owned(),
                name: json::str_of(p, "Name").unwrap_or("Picture").to_owned(),
                zoom: json::f64_of(p, "Zoom", 1.0),
                pan_x: json::f64_of(p, "PanX", 0.0),
                pan_y: json::f64_of(p, "PanY", 0.0),
            })
        });
        Some(Self {
            kind,
            fill: json::color_of(map, "Fill"),
            stroke: json::color_of(map, "Stroke"),
            stroke_width: json::f64_of(map, "StrokeWidth", 0.0),
            corner_radius: json::f64_of(map, "CornerRadius", 0.0),
            points: json::f64_of(map, "Points", 5.0) as i32,
            inner: json::f64_of(map, "Inner", 0.5),
            sweep: json::f64_of(map, "Sweep", 270.0),
            vertices: json::points_of(map, "Vertices"),
            gradient_border: json::bool_of(map, "GradientBorder"),
            dash: match json::f64_of(map, "Dash", 0.0) as i32 {
                1 => StrokeStyle::Dashed,
                2 => StrokeStyle::Dotted,
                _ => StrokeStyle::Solid,
            },
            arrow_start: json::bool_of(map, "ArrowStart"),
            arrow_end: json::bool_of(map, "ArrowEnd"),
            dash_length: json::f64_of(map, "DashLength", 0.0),
            dash_gap: json::f64_of(map, "DashGap", 0.0),
            nodes: map.get("Nodes").and_then(Value::as_array).and_then(|a| a.iter().map(node).collect()),
            closed: json::bool_of(map, "Closed"),
            bend: json::f64_of(map, "Bend", 0.0),
            bend_style: if json::f64_of(map, "BendStyle", 0.0) as i32 == 1 { BendStyle::Wave } else { BendStyle::Arc },
            picture,
        })
    }

    /// Writes the shape's fields into a description (other fields, such as the box and the looks, stay).
    pub fn write(&self, map: &mut Map<String, Value>) {
        let pts = |list: &[PointD]| Value::Array(list.iter().map(|p| json::point_value(*p)).collect());
        json::set(map, "Shape", Some(self.kind.number().into()));
        json::set(map, "Fill", self.fill.map(json::color_value));
        json::set(map, "Stroke", self.stroke.map(json::color_value));
        json::set(map, "StrokeWidth", Some(number(self.stroke_width)));
        json::set(map, "CornerRadius", Some(number(self.corner_radius)));
        json::set(map, "Points", Some(self.points.into()));
        json::set(map, "Inner", Some(number(self.inner)));
        json::set(map, "Sweep", Some(number(self.sweep)));
        json::set(map, "Vertices", self.vertices.as_deref().map(pts));
        json::set(map, "GradientBorder", Some(self.gradient_border.into()));
        let dash = match self.dash {
            StrokeStyle::Solid => 0,
            StrokeStyle::Dashed => 1,
            StrokeStyle::Dotted => 2,
        };
        json::set(map, "Dash", Some(dash.into()));
        json::set(map, "ArrowStart", Some(self.arrow_start.into()));
        json::set(map, "ArrowEnd", Some(self.arrow_end.into()));
        json::set(map, "DashLength", Some(number(self.dash_length)));
        json::set(map, "DashGap", Some(number(self.dash_gap)));
        let nodes = self.nodes.as_ref().map(|list| {
            Value::Array(
                list.iter()
                    .map(|n| {
                        let mut m = Map::new();
                        m.insert("Point".into(), json::point_value(n.point));
                        m.insert("HandleIn".into(), json::point_value(n.handle_in));
                        m.insert("HandleOut".into(), json::point_value(n.handle_out));
                        Value::Object(m)
                    })
                    .collect(),
            )
        });
        json::set(map, "Nodes", nodes);
        json::set(map, "Closed", Some(self.closed.into()));
        json::set(map, "Bend", Some(number(self.bend)));
        json::set(map, "BendStyle", Some((if self.bend_style == BendStyle::Wave { 1 } else { 0 }).into()));
        let picture = self.picture.as_ref().map(|p| {
            let mut m = Map::new();
            m.insert("Image".into(), p.image.clone().into());
            m.insert("Name".into(), p.name.clone().into());
            m.insert("Zoom".into(), number(p.zoom));
            m.insert("PanX".into(), number(p.pan_x));
            m.insert("PanY".into(), number(p.pan_y));
            Value::Object(m)
        });
        json::set(map, "Picture", picture);
    }
}
