//! Image decode / encode, .flash projects, PSD, SVG, PDF, PPTX, GIF, ICO and batch convert.

pub mod decode;
pub mod encode;
mod error;
pub mod format;
pub mod project;

pub use error::IoError;
pub use format::ImageFileFormat;
