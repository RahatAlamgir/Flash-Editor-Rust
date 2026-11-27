//! The `project.json` inside a `.flash` file, field for field as the C# `ProjectManifest` writes it
//! (PascalCase names, nulls left out, objects tagged with `"kind"`, pictures as file names inside the zip).

use serde::{Deserialize, Deserializer, Serialize};

use crate::document::{AdjustmentKind, AdjustmentStyle, CurvePoint};

pub const FORMAT_NAME: &str = "flash-project";
pub const CURRENT_VERSION: u32 = 3;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Manifest {
    pub format: String,
    pub version: u32,
    pub pages: Vec<PageEntry>,
    pub active_page: usize,
    // Version 1 files: one page written straight into the manifest.
    pub width: u32,
    pub height: u32,
    pub active_layer: usize,
    pub layers: Vec<LayerEntry>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            format: FORMAT_NAME.into(),
            version: CURRENT_VERSION,
            pages: Vec::new(),
            active_page: 0,
            width: 0,
            height: 0,
            active_layer: 0,
            layers: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct PageEntry {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub active_layer: usize,
    pub vertical_guides: Vec<f64>,
    pub horizontal_guides: Vec<f64>,
    pub layers: Vec<LayerEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct LayerEntry {
    pub name: String,
    pub opacity: f64,
    pub visible: bool,
    pub locked: bool,
    pub blend_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adjustment: Option<AdjustmentEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixels: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended: Option<String>,
    pub extended_x: i32,
    pub extended_y: i32,
    /// Version 3: where the top-left corner of the (cropped) `Pixels` picture lies on the canvas.
    pub pixels_x: i32,
    pub pixels_y: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore: Option<String>,
    pub restore_x: i32,
    pub restore_y: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_picture: Option<String>,
    pub mask_enabled: bool,
}

impl Default for LayerEntry {
    fn default() -> Self {
        Self {
            name: "Layer".into(),
            opacity: 1.0,
            visible: true,
            locked: false,
            blend_mode: "Normal".into(),
            adjustment: None,
            pixels: None,
            extended: None,
            extended_x: 0,
            extended_y: 0,
            pixels_x: 0,
            pixels_y: 0,
            restore: None,
            restore_x: 0,
            restore_y: 0,
            object: None,
            mask_picture: None,
            mask_enabled: true,
        }
    }
}

/// `AdjustmentStyle` as JSON. `Kind` is a number (0 Basic, 1 Levels, 2 Curves), like the C# enum default.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AdjustmentEntry {
    pub brightness: f64,
    pub contrast: f64,
    pub saturation: f64,
    pub hue: f64,
    #[serde(deserialize_with = "kind_from_json")]
    pub kind: u8,
    pub levels_black: f64,
    pub levels_white: f64,
    pub levels_gamma: f64,
    pub output_black: f64,
    pub output_white: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve: Option<Vec<PointEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_red: Option<Vec<PointEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_green: Option<Vec<PointEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_blue: Option<Vec<PointEntry>>,
    pub temperature: f64,
    pub tint: f64,
}

impl Default for AdjustmentEntry {
    fn default() -> Self {
        Self::from(&AdjustmentStyle::default())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PointEntry {
    pub x: f64,
    pub y: f64,
}

/// Accepts the kind as a number or as a name ("Levels").
fn kind_from_json<'de, D: Deserializer<'de>>(d: D) -> Result<u8, D::Error> {
    Ok(match serde_json::Value::deserialize(d)? {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as u8,
        serde_json::Value::String(s) => match s.to_ascii_lowercase().as_str() {
            "levels" => 1,
            "curves" => 2,
            _ => 0,
        },
        _ => 0,
    })
}

fn points(c: &Option<Vec<CurvePoint>>) -> Option<Vec<PointEntry>> {
    c.as_ref().map(|p| p.iter().map(|p| PointEntry { x: p.x, y: p.y }).collect())
}

fn curve(c: &Option<Vec<PointEntry>>) -> Option<Vec<CurvePoint>> {
    c.as_ref().map(|p| p.iter().map(|p| CurvePoint::new(p.x, p.y)).collect())
}

impl From<&AdjustmentStyle> for AdjustmentEntry {
    fn from(a: &AdjustmentStyle) -> Self {
        Self {
            brightness: a.brightness,
            contrast: a.contrast,
            saturation: a.saturation,
            hue: a.hue,
            kind: match a.kind {
                AdjustmentKind::Basic => 0,
                AdjustmentKind::Levels => 1,
                AdjustmentKind::Curves => 2,
            },
            levels_black: a.levels_black,
            levels_white: a.levels_white,
            levels_gamma: a.levels_gamma,
            output_black: a.output_black,
            output_white: a.output_white,
            curve: points(&a.curve),
            curve_red: points(&a.curve_red),
            curve_green: points(&a.curve_green),
            curve_blue: points(&a.curve_blue),
            temperature: a.temperature,
            tint: a.tint,
        }
    }
}

impl From<&AdjustmentEntry> for AdjustmentStyle {
    fn from(e: &AdjustmentEntry) -> Self {
        Self {
            brightness: e.brightness,
            contrast: e.contrast,
            saturation: e.saturation,
            hue: e.hue,
            kind: match e.kind {
                1 => AdjustmentKind::Levels,
                2 => AdjustmentKind::Curves,
                _ => AdjustmentKind::Basic,
            },
            levels_black: e.levels_black,
            levels_white: e.levels_white,
            levels_gamma: e.levels_gamma,
            output_black: e.output_black,
            output_white: e.output_white,
            curve: curve(&e.curve),
            curve_red: curve(&e.curve_red),
            curve_green: curve(&e.curve_green),
            curve_blue: curve(&e.curve_blue),
            temperature: e.temperature,
            tint: e.tint,
        }
    }
}
