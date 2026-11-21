//! Editable objects (shape, text, picture, group, chart, QR code, barcode): kept as their C# description
//! ([`StoredObject`]) with typed views for the parts the Rust version edits and draws: the box
//! ([`ObjectBox`]), shapes ([`ShapeData`]), pictures and groups. Drawing is in `crate::render`.

pub mod chart;
pub mod codes;
pub mod curve;
mod cutout;
mod foreign;
mod frames;
mod geometry_box;
mod gradient;
pub mod handles;
pub mod json;
mod looks;
pub mod pictures;
mod shape;
pub mod style;
pub mod transforms;

pub use chart::{ChartData, ChartItem, ChartKind};
pub use codes::{BarcodeObjData, QrData, QrStyle};
pub use foreign::{GroupMember, StoredObject, new_picture_name, picture_bytes};
pub use frames::{COLLAGE_LAYOUTS, CollageCell, CollageLayout};
pub use geometry_box::ObjectBox;
pub use gradient::{GRADIENT_PRESETS, GradientPreset, OVERLAY_PRESETS};
pub use looks::{
    GlowStyle, InnerShadowStyle, Looks, OutlineStyle, SHADOW_PRESETS, ShadowPreset, ShadowStyle, THREE_D_PRESETS,
    ThreeDPreset, ThreeDStyle,
};
pub use shape::{BendStyle, CurveNode, Gradient, PictureFill, ShapeData, ShapeKind, StrokeStyle};
