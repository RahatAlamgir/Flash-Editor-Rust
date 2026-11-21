//! A look that can be saved under a name and put on other objects (C# `ObjectStyle`, `SavedStyle`): shadow, 3D,
//! glow, inner shadow, outline, gradient and, for shapes and text, the colors. Size, position, rotation, text and
//! pictures are never part of it.

use serde_json::{Map, Value};

use super::json;
use super::{Gradient, Looks, StoredObject};
use crate::imaging::ColorRgba;

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectStyle {
    /// What the look was taken from: "shape", "text", "image" or "group". Colors and the gradient only go onto the
    /// same kind of object.
    pub kind: String,
    pub looks: Looks,
    pub gradient: Option<Gradient>,
    pub gradient_border: bool,
    /// Fill color of a shape, or the color of a text.
    pub fill: Option<ColorRgba>,
    pub stroke: Option<ColorRgba>,
    pub stroke_width: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedStyle {
    pub name: String,
    pub style: ObjectStyle,
}

fn kind_of(o: &StoredObject) -> &'static str {
    match o.kind() {
        "shape" => "shape",
        "text" => "text",
        "image" => "image",
        _ => "group",
    }
}

impl ObjectStyle {
    /// The look of an object.
    pub fn from(o: &StoredObject) -> Self {
        let mut style = Self {
            kind: kind_of(o).to_owned(),
            looks: Looks { blur: 0.0, ..o.looks() }, // blur is not part of a saved look
            gradient: o.gradient(),
            gradient_border: false,
            fill: None,
            stroke: None,
            stroke_width: 0.0,
        };
        if let Some(s) = o.shape() {
            style.gradient_border = s.gradient_border;
            style.fill = s.fill;
            style.stroke = s.stroke;
            style.stroke_width = s.stroke_width;
        } else if let Some(t) = o.text() {
            style.fill = Some(t.color);
        }
        style
    }

    /// Puts the look on an object. Shadow and the other effects go onto anything; the rest only onto the same kind.
    pub fn apply_to(&self, o: &mut StoredObject) {
        let blur = o.looks().blur;
        o.set_looks(&Looks { blur, ..self.looks });
        if kind_of(o) != self.kind {
            return;
        }
        o.set_gradient(self.gradient.as_ref());
        if let Some(mut s) = o.shape() {
            s.gradient_border = self.gradient_border;
            if s.can_fill() {
                s.fill = self.fill;
            }
            s.stroke = self.stroke;
            if self.stroke.is_some() {
                s.stroke_width = self.stroke_width.max(1.0);
            }
            o.set_shape(&s);
        } else if let Some(mut t) = o.text() {
            t.color = self.fill.unwrap_or(t.color);
            o.set_text(&t);
        }
    }

    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        m.insert("Kind".into(), self.kind.clone().into());
        let mut looks = Map::new();
        self.looks.write(&mut looks);
        looks.shift_remove("BlurAmount");
        for (k, v) in looks {
            m.insert(k, v);
        }
        json::set(&mut m, "Gradient", self.gradient.as_ref().map(Gradient::to_json));
        m.insert("GradientBorder".into(), self.gradient_border.into());
        json::set(&mut m, "Fill", self.fill.map(json::color_value));
        json::set(&mut m, "Stroke", self.stroke.map(json::color_value));
        m.insert("StrokeWidth".into(), json::number(self.stroke_width));
        Value::Object(m)
    }

    pub fn from_json(v: &Value) -> Option<Self> {
        let map = v.as_object()?;
        Some(Self {
            kind: json::str_of(map, "Kind")?.to_owned(),
            looks: Looks::from_json(map),
            gradient: map.get("Gradient").and_then(Gradient::from_json),
            gradient_border: json::bool_of(map, "GradientBorder"),
            fill: json::color_of(map, "Fill"),
            stroke: json::color_of(map, "Stroke"),
            stroke_width: json::f64_of(map, "StrokeWidth", 0.0),
        })
    }
}

/// The saved looks as the text of `styles.json`.
pub fn styles_to_json(list: &[SavedStyle]) -> String {
    let items: Vec<Value> = list
        .iter()
        .map(|s| {
            let mut m = Map::new();
            m.insert("Name".into(), s.name.clone().into());
            m.insert("Style".into(), s.style.to_json());
            Value::Object(m)
        })
        .collect();
    serde_json::to_string_pretty(&Value::Array(items)).unwrap_or_else(|_| "[]".into())
}

/// A damaged list gives an empty one.
pub fn styles_from_json(text: &str) -> Vec<SavedStyle> {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(text) else { return Vec::new() };
    items
        .iter()
        .filter_map(|i| {
            Some(SavedStyle {
                name: i.get("Name")?.as_str()?.to_owned(),
                style: ObjectStyle::from_json(i.get("Style")?)?,
            })
        })
        .collect()
}
