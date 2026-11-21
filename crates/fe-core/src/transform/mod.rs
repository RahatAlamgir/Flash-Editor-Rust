//! Crop, rotate, flip, resize, perspective and straighten (C# `Core/Transforms`).

#![allow(clippy::chunks_exact_to_as_chunks)] // plain 4-byte chunks read best here

mod perspective;
mod resample;
mod rotate;

pub use perspective::{largest_inside_rotated, perspective_crop, straighten, suggested_size};
pub use resample::{ResampleMode, resize, sample_bilinear};
pub use rotate::{flip_horizontal, flip_vertical, rotate, rotate_90_ccw, rotate_90_cw, rotate_180, rotated_size};
