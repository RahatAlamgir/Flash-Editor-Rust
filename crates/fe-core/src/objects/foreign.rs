//! `StoredObject`: an editable object (shape, text, picture, group, chart, QR code, barcode) kept as its C#
//! description (JSON) plus the pictures it uses. Keeping the description itself means nothing the Rust version
//! does not know yet (text, charts, shadows, 3D...) is ever lost when a project is saved; the typed views
//! (`box_`, `shape`, `group_members`...) read and change just the fields they know.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Map, Value};

use super::geometry_box::ObjectBox;
use super::json::{self, number};
use super::shape::{Gradient, ShapeData};
use crate::imaging::{ColorRgba, PixelBuffer, PointD};

/// An object description as saved in a project file, plus the pictures it refers to.
#[derive(Clone, Debug, PartialEq)]
pub struct StoredObject {
    /// The JSON description (with `"kind": "shape" | "text" | "image" | ...`).
    pub json: serde_json::Value,
    /// Pictures the description refers to, by the name it uses for them (`images/12.png`), as PNG bytes.
    pub images: HashMap<String, Arc<Vec<u8>>>,
}

/// A member of a group: the object, and the layer name and opacity it had before grouping.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupMember {
    pub object: StoredObject,
    pub name: String,
    pub opacity: f64,
}

/// The looks the Rust version cannot draw yet (Phase 9): objects with them keep their saved drawing while only
/// moved, resized or turned.
const LOOKS: [&str; 0] = [];

/// A name for a picture made in this program (the file gets real names when saved).
pub fn new_picture_name() -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    format!("images/new-{}.png", NEXT.fetch_add(1, Ordering::Relaxed))
}

fn box_value(b: ObjectBox) -> Value {
    let mut m = Map::new();
    m.insert("X".into(), number(b.x));
    m.insert("Y".into(), number(b.y));
    m.insert("Width".into(), number(b.width));
    m.insert("Height".into(), number(b.height));
    m.insert("Rotation".into(), number(b.rotation));
    Value::Object(m)
}

fn box_of(v: &Value) -> Option<ObjectBox> {
    let g = |n: &str| v.get(n).and_then(Value::as_f64);
    Some(ObjectBox::new(g("X")?, g("Y")?, g("Width")?, g("Height")?, g("Rotation").unwrap_or(0.0)))
}

impl StoredObject {
    fn map(&self) -> Option<&Map<String, Value>> {
        self.json.as_object()
    }

    fn map_mut(&mut self) -> &mut Map<String, Value> {
        if !self.json.is_object() {
            self.json = Value::Object(Map::new());
        }
        self.json.as_object_mut().expect("just made an object")
    }

    /// The kind of object: "shape", "text", "image", "group", "chart", "qr" or "barcode".
    pub fn kind(&self) -> &str {
        self.json.get("kind").and_then(|k| k.as_str()).unwrap_or("object")
    }

    /// A new shape.
    pub fn new_shape(b: ObjectBox, shape: &ShapeData) -> Self {
        let mut map = Map::new();
        map.insert("kind".into(), "shape".into());
        shape.write(&mut map);
        let mut object = Self { json: Value::Object(map), images: HashMap::new() };
        object.finish_new(b);
        object
    }

    /// A new text object.
    pub fn new_text(b: ObjectBox, text: &crate::text::TextData) -> Self {
        let mut map = Map::new();
        map.insert("kind".into(), "text".into());
        text.write(&mut map);
        let mut object = Self { json: Value::Object(map), images: HashMap::new() };
        object.finish_new(b);
        object
    }

    /// The text's settings, for a text object.
    pub fn text(&self) -> Option<crate::text::TextData> {
        if self.kind() != "text" {
            return None;
        }
        crate::text::TextData::from_json(self.map()?)
    }

    pub fn set_text(&mut self, text: &crate::text::TextData) {
        text.write(self.map_mut());
    }

    fn new_plain(kind: &str, b: ObjectBox, write: impl FnOnce(&mut Map<String, Value>)) -> Self {
        let mut map = Map::new();
        map.insert("kind".into(), kind.into());
        write(&mut map);
        let mut object = Self { json: Value::Object(map), images: HashMap::new() };
        object.finish_new(b);
        object
    }

    /// A new chart.
    pub fn new_chart(b: ObjectBox, chart: &super::chart::ChartData) -> Self {
        Self::new_plain("chart", b, |m| chart.write(m))
    }

    /// The chart's settings, for a chart.
    pub fn chart(&self) -> Option<super::chart::ChartData> {
        if self.kind() != "chart" {
            return None;
        }
        super::chart::ChartData::from_json(self.map()?)
    }

    pub fn set_chart(&mut self, chart: &super::chart::ChartData) {
        chart.write(self.map_mut());
    }

    /// A new QR code.
    pub fn new_qr(b: ObjectBox, qr: &super::codes::QrData) -> Self {
        Self::new_plain("qr", b, |m| qr.write(m))
    }

    pub fn qr(&self) -> Option<super::codes::QrData> {
        if self.kind() != "qr" {
            return None;
        }
        super::codes::QrData::from_json(self.map()?)
    }

    pub fn set_qr(&mut self, qr: &super::codes::QrData) {
        qr.write(self.map_mut());
    }

    /// A new barcode.
    pub fn new_barcode(b: ObjectBox, code: &super::codes::BarcodeObjData) -> Self {
        Self::new_plain("barcode", b, |m| code.write(m))
    }

    pub fn barcode(&self) -> Option<super::codes::BarcodeObjData> {
        if self.kind() != "barcode" {
            return None;
        }
        super::codes::BarcodeObjData::from_json(self.map()?)
    }

    pub fn set_barcode(&mut self, code: &super::codes::BarcodeObjData) {
        code.write(self.map_mut());
    }

    /// The object's own color adjustment (Properties > Adjust), if it has one.
    pub fn tone(&self) -> Option<crate::document::AdjustmentStyle> {
        let entry: crate::io::project::AdjustmentEntry =
            serde_json::from_value(self.map()?.get("Tone")?.clone()).ok()?;
        Some(crate::document::AdjustmentStyle::from(&entry))
    }

    /// Sets the color adjustment (a neutral one is the same as none).
    pub fn set_tone(&mut self, tone: Option<&crate::document::AdjustmentStyle>) {
        let value = tone
            .filter(|t| !t.is_neutral())
            .and_then(|t| serde_json::to_value(crate::io::project::AdjustmentEntry::from(t)).ok());
        json::set(self.map_mut(), "Tone", value);
    }

    /// A new placed picture (the PNG bytes of `picture`).
    pub fn new_image(b: ObjectBox, picture: Arc<Vec<u8>>, name: &str) -> Self {
        let file = new_picture_name();
        let mut map = Map::new();
        map.insert("kind".into(), "image".into());
        map.insert("Source".into(), file.clone().into());
        map.insert("Name".into(), name.into());
        map.insert("Revision".into(), 0.into());
        let mut object = Self { json: Value::Object(map), images: HashMap::from([(file, picture)]) };
        object.finish_new(b);
        object
    }

    /// Box, flips and blur go after the kind's own fields, like the C# files.
    fn finish_new(&mut self, b: ObjectBox) {
        let map = self.map_mut();
        map.insert("Box".into(), box_value(b));
        map.insert("FlipX".into(), false.into());
        map.insert("FlipY".into(), false.into());
        map.insert("BlurAmount".into(), 0.into());
    }

    pub fn box_(&self) -> Option<ObjectBox> {
        self.json.get("Box").and_then(box_of)
    }

    pub fn set_box(&mut self, b: ObjectBox) {
        self.map_mut().insert("Box".into(), box_value(b));
    }

    pub fn flips(&self) -> (bool, bool) {
        self.map().map_or((false, false), |m| (json::bool_of(m, "FlipX"), json::bool_of(m, "FlipY")))
    }

    pub fn set_flips(&mut self, x: bool, y: bool) {
        let map = self.map_mut();
        map.insert("FlipX".into(), x.into());
        map.insert("FlipY".into(), y.into());
    }

    /// The object's corners after Reshape (pictures, groups, curved shapes), in the box (0..1), if any.
    pub fn warp(&self) -> Option<Vec<PointD>> {
        self.map().and_then(|m| json::points_of(m, "Warp")).filter(|w| w.len() == 4)
    }

    pub fn set_warp(&mut self, warp: Option<&[PointD]>) {
        let value = warp.map(|w| Value::Array(w.iter().map(|p| json::point_value(*p)).collect()));
        json::set(self.map_mut(), "Warp", value);
    }

    pub fn gradient(&self) -> Option<Gradient> {
        self.json.get("Gradient").and_then(Gradient::from_json)
    }

    pub fn set_gradient(&mut self, g: Option<&Gradient>) {
        json::set(self.map_mut(), "Gradient", g.map(Gradient::to_json));
    }

    pub fn blur_amount(&self) -> f64 {
        self.map().map_or(0.0, |m| json::f64_of(m, "BlurAmount", 0.0))
    }

    /// True when the object has looks the Rust version does not draw yet (the per-object color adjustment).
    pub fn has_looks(&self) -> bool {
        self.map().is_some_and(|m| LOOKS.iter().any(|k| m.get(*k).is_some_and(|v| !v.is_null())))
            || self.group_members().is_some_and(|ms| ms.iter().any(|m| m.object.has_looks()))
    }

    /// True when the Rust version can draw the object itself (shapes, pictures, and groups made of them).
    pub fn can_draw(&self) -> bool {
        match self.kind() {
            "shape" | "image" | "text" | "chart" | "qr" | "barcode" => true,
            "group" => self.group_members().is_some_and(|ms| ms.iter().all(|m| m.object.can_draw())),
            _ => false,
        }
    }

    /// The shape's settings, for a shape.
    pub fn shape(&self) -> Option<ShapeData> {
        if self.kind() != "shape" {
            return None;
        }
        ShapeData::from_json(self.map()?)
    }

    pub fn set_shape(&mut self, shape: &ShapeData) {
        shape.write(self.map_mut());
    }

    /// The picture of a placed image, or of a shape's picture fill.
    pub fn picture(&self) -> Option<&Arc<Vec<u8>>> {
        let name = match self.kind() {
            "image" => self.json.get("Source")?.as_str()?.to_owned(),
            "shape" => self.json.get("Picture")?.get("Image")?.as_str()?.to_owned(),
            _ => return None,
        };
        self.images.get(&name)
    }

    /// The name of a placed picture ("Image" when it has none).
    pub fn image_name(&self) -> &str {
        self.json.get("Name").and_then(Value::as_str).unwrap_or("Image")
    }

    /// The members of a group (each with the group's pictures it needs).
    pub fn group_members(&self) -> Option<Vec<GroupMember>> {
        if self.kind() != "group" {
            return None;
        }
        let list = self.json.get("Members")?.as_array()?;
        Some(
            list.iter()
                .filter_map(|m| {
                    let object = m.get("Object")?.clone();
                    Some(GroupMember {
                        object: self.child(object),
                        name: m.get("Name").and_then(Value::as_str).unwrap_or("Object").to_owned(),
                        opacity: m.get("Opacity").and_then(Value::as_f64).unwrap_or(1.0),
                    })
                })
                .collect(),
        )
    }

    /// A member description with the pictures it uses.
    fn child(&self, json: Value) -> StoredObject {
        let mut names = Vec::new();
        let images = &self.images;
        Self::picture_names(&json, &|s| images.contains_key(s), &mut names);
        let images = names.into_iter().filter_map(|n| self.images.get(&n).map(|b| (n, b.clone()))).collect();
        StoredObject { json, images }
    }

    /// The box the members covered when they were grouped.
    pub fn original_box(&self) -> Option<ObjectBox> {
        self.json.get("OriginalBox").and_then(box_of)
    }

    /// Groups objects (bottom to top) without moving them (C# `GroupObject.Create`).
    pub fn new_group(members: &[GroupMember]) -> Option<Self> {
        let mut corners = members.iter().filter_map(|m| m.object.box_()).flat_map(|b| b.corners());
        let first = corners.next()?;
        let (mut l, mut t, mut r, mut b) = (first.x, first.y, first.x, first.y);
        for c in corners {
            (l, t, r, b) = (l.min(c.x), t.min(c.y), r.max(c.x), b.max(c.y));
        }
        let bx = ObjectBox::new(l, t, (r - l).max(1.0), (b - t).max(1.0), 0.0);
        let mut images = HashMap::new();
        let list: Vec<Value> = members
            .iter()
            .map(|m| {
                images.extend(m.object.images.iter().map(|(k, v)| (k.clone(), v.clone())));
                let mut entry = Map::new();
                entry.insert("Object".into(), m.object.json.clone());
                entry.insert("Name".into(), m.name.clone().into());
                entry.insert("Opacity".into(), number(m.opacity));
                Value::Object(entry)
            })
            .collect();
        let mut map = Map::new();
        map.insert("kind".into(), "group".into());
        map.insert("Members".into(), Value::Array(list));
        map.insert("OriginalBox".into(), box_value(bx));
        let mut group = Self { json: Value::Object(map), images };
        group.finish_new(bx);
        Some(group)
    }

    /// The members of a group as they look now on the canvas (moved, scaled, turned and flipped with the group).
    pub fn current_members(&self) -> Option<Vec<GroupMember>> {
        let members = self.group_members()?;
        let (gb, ob) = (self.box_()?, self.original_box()?);
        let (sx, sy) = (gb.width / ob.width.max(1e-9), gb.height / ob.height.max(1e-9));
        let (fx, fy) = self.flips();
        let mirrored = fx != fy;
        let c0 = ob.center();
        Some(
            members
                .into_iter()
                .map(|m| {
                    let mut child = m.object.scaled(sx, sy);
                    let (Some(mb), Some(cb)) = (m.object.box_(), child.box_()) else { return m };
                    let mc = mb.center();
                    let x = (mc.x - c0.x) * sx * if fx { -1.0 } else { 1.0 };
                    let y = (mc.y - c0.y) * sy * if fy { -1.0 } else { 1.0 };
                    let center = gb.to_canvas(x, y);
                    let rotation = if mirrored { -mb.rotation } else { mb.rotation } + gb.rotation;
                    child.set_box(ObjectBox::from_center(
                        center,
                        cb.width,
                        cb.height,
                        ObjectBox::normalize_angle(rotation),
                    ));
                    let (cfx, cfy) = child.flips();
                    child.set_flips(cfx != fx, cfy != fy);
                    GroupMember { object: child, ..m }
                })
                .collect(),
        )
    }

    /// One part of a group as it looks on the canvas now (`None` when there is no such part).
    pub fn part(&self, index: usize) -> Option<GroupMember> {
        self.current_members()?.into_iter().nth(index)
    }

    /// The group after one part was changed (C# `GroupObject.WithPart`): the group is fitted around its parts again;
    /// the group's own shadow is kept.
    pub fn with_part(&self, index: usize, changed: StoredObject) -> Option<Self> {
        let mut parts = self.current_members()?;
        parts.get_mut(index)?.object = changed;
        self.regrouped(&parts)
    }

    /// The group without one part (`None` when it was the only part).
    pub fn without_part(&self, index: usize) -> Option<Self> {
        let mut parts = self.current_members()?;
        if index >= parts.len() || parts.len() < 2 {
            return None;
        }
        parts.remove(index);
        self.regrouped(&parts)
    }

    /// The group after one part got a new name.
    pub fn with_part_name(&self, index: usize, name: &str) -> Option<Self> {
        let mut parts = self.current_members()?;
        parts.get_mut(index)?.name = name.to_owned();
        self.regrouped(&parts)
    }

    fn regrouped(&self, parts: &[GroupMember]) -> Option<Self> {
        let mut group = Self::new_group(parts)?;
        if let (Some(shadow), Some(map)) = (self.json.get("Shadow"), group.json.as_object_mut()) {
            map.insert("Shadow".into(), shadow.clone());
        }
        Some(group)
    }

    /// The same object moved by (dx, dy).
    pub fn offset(&self, dx: f64, dy: f64) -> Self {
        let mut moved = self.clone();
        if let Some(b) = self.box_() {
            moved.set_box(b.offset(dx, dy));
        }
        moved
    }

    /// Position and size scaled (C# `ObjectTransforms.Scale`): borders, corners, text sizes and looks grow too.
    pub fn scaled(&self, sx: f64, sy: f64) -> Self {
        let mut s = self.clone();
        if let Some(b) = self.box_() {
            s.set_box(ObjectBox::new(b.x * sx, b.y * sy, b.width * sx, b.height * sy, b.rotation));
        }
        let f = (sx * sy).abs().sqrt();
        let map = s.map_mut();
        let scale = |map: &mut Map<String, Value>, name: &str| {
            if let Some(v) = map.get(name).and_then(Value::as_f64) {
                map.insert(name.to_owned(), number(v * f));
            }
        };
        match map.get("kind").and_then(Value::as_str) {
            Some("shape") => {
                scale(map, "StrokeWidth");
                scale(map, "CornerRadius");
            }
            Some("text") => {
                scale(map, "FontSize");
                scale(map, "OutlineWidth");
            }
            _ => {}
        }
        for (look, fields) in [
            ("Shadow", &["Distance", "Blur"][..]),
            ("ThreeD", &["Depth"][..]),
            ("Glow", &["Size"][..]),
            ("InnerShadow", &["Distance", "Blur"][..]),
            ("Outline", &["Width"][..]),
        ] {
            if let Some(Value::Object(l)) = map.get_mut(look) {
                for field in fields {
                    scale(l, field);
                }
            }
        }
        s
    }

    /// The color of a shape's fill (or a line), for snapping guides and the Properties panel.
    pub fn fill(&self) -> Option<ColorRgba> {
        self.shape().and_then(|s| s.fill.or(s.stroke))
    }

    /// Every string in the description that names a picture inside the project file.
    pub fn picture_names(json: &serde_json::Value, is_picture: &dyn Fn(&str) -> bool, out: &mut Vec<String>) {
        match json {
            serde_json::Value::String(s) if is_picture(s) => out.push(s.clone()),
            serde_json::Value::Array(items) => items.iter().for_each(|v| Self::picture_names(v, is_picture, out)),
            serde_json::Value::Object(map) => map.values().for_each(|v| Self::picture_names(v, is_picture, out)),
            _ => {}
        }
    }

    /// A copy of the description with picture names replaced (when saving under new names).
    pub fn renamed(&self, rename: &dyn Fn(&str) -> Option<String>) -> serde_json::Value {
        fn walk(v: &serde_json::Value, rename: &dyn Fn(&str) -> Option<String>) -> serde_json::Value {
            match v {
                serde_json::Value::String(s) => rename(s).map_or_else(|| v.clone(), serde_json::Value::String),
                serde_json::Value::Array(items) => items.iter().map(|i| walk(i, rename)).collect(),
                serde_json::Value::Object(map) => {
                    map.iter().map(|(k, val)| (k.clone(), walk(val, rename))).collect::<serde_json::Map<_, _>>().into()
                }
                other => other.clone(),
            }
        }
        walk(&self.json, rename)
    }

    /// The object's box (x, y, width, height), if it has one.
    pub fn bounds(&self) -> Option<(f64, f64, f64, f64)> {
        self.box_().map(|b| (b.x, b.y, b.width, b.height))
    }
}

/// A picture for a new image object: its PNG bytes (quick compression; real files re-use them as they are).
pub fn picture_bytes(pixels: &PixelBuffer) -> Arc<Vec<u8>> {
    let bytes = crate::io::encode::encode_png(pixels).unwrap_or_default();
    let bytes = Arc::new(bytes);
    super::pictures::remember(&bytes, Arc::new(pixels.clone()));
    bytes
}
