//! The looks every object can have (C# `ShadowStyle`, `GlowStyle`, `InnerShadowStyle`, `OutlineStyle`, `ThreeDStyle`,
//! `BlurAmount`): kept in the object's description, with typed views here.

use serde_json::{Map, Value};

use super::StoredObject;
use super::json::{self, number};
use crate::imaging::ColorRgba;

/// A drop shadow behind an object (follows its real outline). The direction is on the canvas: turning the object does
/// not turn its shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowStyle {
    /// Its alpha is the strength of the shadow.
    pub color: ColorRgba,
    pub distance: f64,
    /// 0 = right, 90 = down.
    pub angle: f64,
    pub blur: f64,
    /// 1 = as big as the object, 2 = twice as big (around its middle).
    pub size: f64,
}

impl ShadowStyle {
    /// What "add shadow" starts with: soft, dark and a little to the bottom right.
    pub const DEFAULT: Self =
        Self { color: ColorRgba::new(0, 0, 0, 115), distance: 8.0, angle: 45.0, blur: 12.0, size: 1.0 };

    pub fn offset(&self) -> (f64, f64) {
        offset(self.distance, self.angle)
    }

    /// How far the shadow can reach beyond the outline of an object of this size.
    pub fn extent_for(&self, w: f64, h: f64) -> f64 {
        self.distance.abs() + self.blur.max(0.0) * 2.0 + 2.0 + (self.size - 1.0).max(0.0) * w.max(h) / 2.0
    }

    fn scaled(&self, f: f64) -> Self {
        Self { distance: self.distance * f, blur: self.blur * f, ..*self }
    }
}

/// A soft light around an object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowStyle {
    pub color: ColorRgba,
    pub size: f64,
    /// Percent: 100 = normal.
    pub intensity: f64,
}

impl GlowStyle {
    pub const DEFAULT: Self = Self { color: ColorRgba::new(255, 214, 90, 235), size: 14.0, intensity: 100.0 };

    pub fn extent(&self) -> f64 {
        self.size.max(0.0) * 2.0 + 2.0
    }
}

/// A shadow inside the object's edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InnerShadowStyle {
    pub color: ColorRgba,
    pub distance: f64,
    pub angle: f64,
    pub blur: f64,
}

impl InnerShadowStyle {
    pub const DEFAULT: Self = Self { color: ColorRgba::new(0, 0, 0, 150), distance: 6.0, angle: 45.0, blur: 10.0 };

    pub fn offset(&self) -> (f64, f64) {
        offset(self.distance, self.angle)
    }
}

/// A line around the outside of an object.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutlineStyle {
    pub color: ColorRgba,
    pub width: f64,
}

impl OutlineStyle {
    pub const DEFAULT: Self = Self { color: ColorRgba::new(255, 255, 255, 255), width: 4.0 };

    pub fn extent(&self) -> f64 {
        self.width.max(0.0) + 2.0
    }
}

/// A solid side (extrusion) and a bevel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThreeDStyle {
    pub depth: f64,
    pub angle: f64,
    /// `None` = the object's own color, darker.
    pub color: Option<ColorRgba>,
    /// 0..1.
    pub bevel: f64,
}

impl ThreeDStyle {
    pub const DEFAULT: Self = Self { depth: 20.0, angle: 45.0, color: None, bevel: 0.5 };

    pub fn offset(&self) -> (f64, f64) {
        offset(self.depth, self.angle)
    }

    pub fn extent(&self) -> f64 {
        self.depth.max(0.0) + 2.0
    }
}

fn offset(distance: f64, angle: f64) -> (f64, f64) {
    let a = angle.to_radians();
    (distance * a.cos(), distance * a.sin())
}

/// All looks of one object.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Looks {
    pub shadow: Option<ShadowStyle>,
    pub glow: Option<GlowStyle>,
    pub inner_shadow: Option<InnerShadowStyle>,
    pub outline: Option<OutlineStyle>,
    pub three_d: Option<ThreeDStyle>,
    /// How much the object itself is blurred, in pixels (0 = sharp).
    pub blur: f64,
}

impl Looks {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
            || (self.shadow.is_none()
                && self.glow.is_none()
                && self.inner_shadow.is_none()
                && self.outline.is_none()
                && self.three_d.is_none()
                && self.blur < 0.3)
    }

    /// How far the looks reach beyond the object's outline, in pixels (C# `EffectReach`).
    pub fn reach(&self, w: f64, h: f64) -> f64 {
        self.shadow.map_or(0.0, |s| s.extent_for(w, h))
            + self.three_d.map_or(0.0, |t| t.extent())
            + self.glow.map_or(0.0, |g| g.extent())
            + self.outline.map_or(0.0, |o| o.extent())
            + self.blur * 3.0
    }

    /// The same looks for an object that was stretched by `f` (inside a group).
    pub fn scaled(&self, f: f64) -> Self {
        Self {
            shadow: self.shadow.map(|s| s.scaled(f)),
            glow: self.glow.map(|g| GlowStyle { size: g.size * f, ..g }),
            inner_shadow: self.inner_shadow.map(|i| InnerShadowStyle {
                distance: i.distance * f,
                blur: i.blur * f,
                ..i
            }),
            outline: self.outline.map(|o| OutlineStyle { width: o.width * f, ..o }),
            three_d: self.three_d.map(|t| ThreeDStyle { depth: t.depth * f, ..t }),
            blur: self.blur,
        }
    }
}

fn color_field(v: &Value) -> Option<ColorRgba> {
    json::color(v.get("Color")?)
}

impl Looks {
    pub fn from_json(map: &Map<String, Value>) -> Self {
        let sub = |name: &str| map.get(name).filter(|v| v.is_object());
        let num = |v: &Value, n: &str, d: f64| v.get(n).and_then(Value::as_f64).unwrap_or(d);
        Self {
            shadow: sub("Shadow").and_then(|v| {
                Some(ShadowStyle {
                    color: color_field(v)?,
                    distance: num(v, "Distance", 8.0),
                    angle: num(v, "Angle", 45.0),
                    blur: num(v, "Blur", 12.0),
                    size: num(v, "Size", 1.0),
                })
            }),
            glow: sub("Glow").and_then(|v| {
                Some(GlowStyle {
                    color: color_field(v)?,
                    size: num(v, "Size", 14.0),
                    intensity: num(v, "Intensity", 100.0),
                })
            }),
            inner_shadow: sub("InnerShadow").and_then(|v| {
                Some(InnerShadowStyle {
                    color: color_field(v)?,
                    distance: num(v, "Distance", 6.0),
                    angle: num(v, "Angle", 45.0),
                    blur: num(v, "Blur", 10.0),
                })
            }),
            outline: sub("Outline")
                .and_then(|v| Some(OutlineStyle { color: color_field(v)?, width: num(v, "Width", 4.0) })),
            three_d: sub("ThreeD").map(|v| ThreeDStyle {
                depth: num(v, "Depth", 20.0),
                angle: num(v, "Angle", 45.0),
                color: color_field(v),
                bevel: num(v, "Bevel", 0.5),
            }),
            blur: json::f64_of(map, "BlurAmount", 0.0),
        }
    }

    pub fn write(&self, map: &mut Map<String, Value>) {
        let obj = |fields: Vec<(&str, Value)>| {
            let mut m = Map::new();
            for (k, v) in fields {
                m.insert(k.to_owned(), v);
            }
            Value::Object(m)
        };
        let c = |c: ColorRgba| json::color_value(c);
        json::set(
            map,
            "Shadow",
            self.shadow.map(|s| {
                obj(vec![
                    ("Color", c(s.color)),
                    ("Distance", number(s.distance)),
                    ("Angle", number(s.angle)),
                    ("Blur", number(s.blur)),
                    ("Size", number(s.size)),
                ])
            }),
        );
        json::set(
            map,
            "Glow",
            self.glow.map(|g| {
                obj(vec![("Color", c(g.color)), ("Size", number(g.size)), ("Intensity", number(g.intensity))])
            }),
        );
        json::set(
            map,
            "InnerShadow",
            self.inner_shadow.map(|i| {
                obj(vec![
                    ("Color", c(i.color)),
                    ("Distance", number(i.distance)),
                    ("Angle", number(i.angle)),
                    ("Blur", number(i.blur)),
                ])
            }),
        );
        json::set(map, "Outline", self.outline.map(|o| obj(vec![("Color", c(o.color)), ("Width", number(o.width))])));
        json::set(
            map,
            "ThreeD",
            self.three_d.map(|t| {
                let mut fields = vec![("Depth", number(t.depth)), ("Angle", number(t.angle))];
                if let Some(col) = t.color {
                    fields.push(("Color", c(col)));
                }
                fields.push(("Bevel", number(t.bevel)));
                obj(fields)
            }),
        );
        json::set(map, "BlurAmount", Some(number(self.blur)));
    }
}

impl StoredObject {
    /// The looks (shadow, glow, 3D...) of the object.
    pub fn looks(&self) -> Looks {
        self.json.as_object().map(Looks::from_json).unwrap_or_default()
    }

    pub fn set_looks(&mut self, looks: &Looks) {
        if let Some(map) = self.json.as_object_mut() {
            looks.write(map);
        }
    }
}

/// A ready-made shadow (C# `ShadowPreset`).
pub struct ShadowPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub distance: f64,
    pub angle: f64,
    pub blur: f64,
    pub opacity: u8,
    pub color: Option<ColorRgba>,
}

impl ShadowPreset {
    /// The shadow this preset gives; a preset without its own color keeps the color of the current shadow.
    pub fn create(&self, current: Option<ShadowStyle>) -> ShadowStyle {
        let color = self.color.or(current.map(|s| s.color)).unwrap_or(ColorRgba::BLACK);
        ShadowStyle {
            color: ColorRgba { a: self.opacity, ..color },
            distance: self.distance,
            angle: self.angle,
            blur: self.blur,
            size: current.map_or(1.0, |s| s.size),
        }
    }
}

const fn sp(
    name: &'static str,
    description: &'static str,
    distance: f64,
    angle: f64,
    blur: f64,
    opacity: u8,
    color: Option<ColorRgba>,
) -> ShadowPreset {
    ShadowPreset { name, description, distance, angle, blur, opacity, color }
}

pub const SHADOW_PRESETS: [ShadowPreset; 10] = [
    sp("Soft", "Gentle, wide shadow under the object", 8.0, 90.0, 24.0, 90, None),
    sp("Hard", "Crisp shadow with no blur, offset to the bottom right", 10.0, 45.0, 0.0, 190, None),
    sp("Lifted", "Big, faint shadow far below: the object floats", 30.0, 90.0, 44.0, 80, None),
    sp("Subtle", "Very light, small shadow", 3.0, 90.0, 8.0, 60, None),
    sp("Contact", "Small, tight shadow: the object lies on the page", 2.0, 90.0, 4.0, 130, None),
    sp("Sticker", "Solid black offset shadow, comic style", 7.0, 45.0, 0.0, 255, None),
    sp("Long", "Long, hard shadow stretching away", 44.0, 45.0, 0.0, 70, None),
    sp("Glow", "Soft blue halo around the object", 0.0, 0.0, 34.0, 210, Some(ColorRgba::new(70, 150, 255, 255))),
    sp("Neon", "Bright pink neon glow", 0.0, 0.0, 16.0, 255, Some(ColorRgba::new(255, 60, 170, 255))),
    sp("Left", "Shadow falling to the bottom left", 14.0, 135.0, 16.0, 110, None),
];

/// A ready-made 3D look (C# `ThreeDPreset`).
pub struct ThreeDPreset {
    pub name: &'static str,
    pub description: &'static str,
    pub depth: f64,
    pub angle: f64,
    pub bevel: f64,
}

impl ThreeDPreset {
    /// The effect this preset gives; the color the object's 3D effect already has is kept.
    pub fn create(&self, current: Option<ThreeDStyle>) -> ThreeDStyle {
        ThreeDStyle { depth: self.depth, angle: self.angle, color: current.and_then(|t| t.color), bevel: self.bevel }
    }
}

pub const THREE_D_PRESETS: [ThreeDPreset; 5] = [
    ThreeDPreset {
        name: "Block",
        description: "A solid block going to the bottom right",
        depth: 24.0,
        angle: 45.0,
        bevel: 0.35,
    },
    ThreeDPreset { name: "Long", description: "A deep side stretching far away", depth: 70.0, angle: 45.0, bevel: 0.2 },
    ThreeDPreset {
        name: "Pop",
        description: "Raised face with shiny edges, very little depth",
        depth: 6.0,
        angle: 90.0,
        bevel: 0.9,
    },
    ThreeDPreset {
        name: "Floor",
        description: "A thick side going straight down, like a button",
        depth: 18.0,
        angle: 90.0,
        bevel: 0.5,
    },
    ThreeDPreset {
        name: "Left",
        description: "A block going to the bottom left",
        depth: 24.0,
        angle: 135.0,
        bevel: 0.35,
    },
];
