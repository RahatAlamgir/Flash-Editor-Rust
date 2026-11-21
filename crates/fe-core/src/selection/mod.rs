//! Selections: which pixels of the canvas are selected (a mask with soft values) and the outline drawn as
//! "marching ants". Built from rectangles, ellipses, polygons, Bezier paths, the magic wand or the quick-select
//! brush, and combined with Shift (add) and Alt (subtract). Same rules as the C# `FlashEditor.Core.Selections`.

mod area;
pub mod bezier;
pub mod magic_wand;
pub mod outline;
pub mod polygon;
pub mod quick_select;

pub use area::{CombineMode, Selection, ellipse_points};
