//! Clone, heal, spot heal, patch, dodge / burn, blur / sharpen and liquify (C# `Core/Retouch`, `Liquify`).

pub mod liquify;
mod ops;
mod spot;
mod stroke;

pub use liquify::LiquifyMode;
pub use ops::{RetouchOp, Scene, ToneRange};
pub use spot::find_offset as find_spot_source;
pub use stroke::{RetouchStroke, apply_region};
