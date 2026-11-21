//! Reading and writing the fields of an object description (the C# JSON: PascalCase names, colors as
//! `{R, G, B, A}`, points as `{X, Y}`, enums as numbers, doubles written without ".0" when whole).

use serde_json::{Map, Value};

use crate::imaging::{ColorRgba, PointD};

/// A JSON number written the way C# writes a double: whole numbers without ".0".
pub fn number(v: f64) -> Value {
    if v.is_finite() && v.fract() == 0.0 && v.abs() < 9.0e15 { Value::from(v as i64) } else { Value::from(v) }
}

pub fn f64_of(map: &Map<String, Value>, name: &str, default: f64) -> f64 {
    map.get(name).and_then(Value::as_f64).unwrap_or(default)
}

pub fn bool_of(map: &Map<String, Value>, name: &str) -> bool {
    map.get(name).and_then(Value::as_bool).unwrap_or(false)
}

pub fn str_of<'a>(map: &'a Map<String, Value>, name: &str) -> Option<&'a str> {
    map.get(name).and_then(Value::as_str)
}

pub fn color(v: &Value) -> Option<ColorRgba> {
    let c = |n: &str| v.get(n).and_then(Value::as_u64).map(|x| x.min(255) as u8);
    Some(ColorRgba::new(c("R")?, c("G")?, c("B")?, c("A").unwrap_or(255)))
}

pub fn color_of(map: &Map<String, Value>, name: &str) -> Option<ColorRgba> {
    map.get(name).and_then(color)
}

pub fn color_value(c: ColorRgba) -> Value {
    let mut m = Map::new();
    m.insert("R".into(), c.r.into());
    m.insert("G".into(), c.g.into());
    m.insert("B".into(), c.b.into());
    m.insert("A".into(), c.a.into());
    Value::Object(m)
}

pub fn point(v: &Value) -> Option<PointD> {
    Some(PointD::new(v.get("X")?.as_f64()?, v.get("Y")?.as_f64()?))
}

pub fn point_value(p: PointD) -> Value {
    let mut m = Map::new();
    m.insert("X".into(), number(p.x));
    m.insert("Y".into(), number(p.y));
    Value::Object(m)
}

pub fn points_of(map: &Map<String, Value>, name: &str) -> Option<Vec<PointD>> {
    map.get(name)?.as_array()?.iter().map(point).collect()
}

/// Sets a field, or removes it for `None` (C# leaves nulls out). Keeps the field's place when it already exists.
pub fn set(map: &mut Map<String, Value>, name: &str, value: Option<Value>) {
    match value {
        Some(v) => {
            map.insert(name.to_owned(), v);
        }
        None => {
            map.shift_remove(name);
        }
    }
}
