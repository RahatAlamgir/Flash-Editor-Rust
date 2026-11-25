//! Image decode / encode, .flash projects, PSD, SVG, PDF, PPTX, GIF, ICO and batch convert.
#![allow(clippy::chunks_exact_to_as_chunks)] // plain 4-byte pixel chunks read best here

pub mod batch;
pub mod decode;
pub mod encode;
mod error;
pub mod export;
pub mod format;
pub mod gif;
pub mod ico;
pub mod pdf;
pub mod pptx;
pub mod project;
pub mod psd;
pub mod svg;
pub mod wic;

pub use error::IoError;
pub use format::ImageFileFormat;
