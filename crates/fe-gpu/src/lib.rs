//! The canvas renderer. It shows a document on the graphics card:
//!
//! * the canvas is cut into square tiles; each tile's layers are combined on the processor (in parallel,
//!   only for tiles that are on screen and changed — see `fe_core::paint::signature`) and uploaded once as a
//!   texture;
//! * zooming and panning only move the textures, so they cost almost nothing;
//! * when zoomed out, smaller copies ("levels") are used: level 1 is half size, level 2 a quarter...,
//!   so a huge picture at 25 % costs what a small one does;
//! * textures that have not been on screen for a while are dropped to give memory back.
//!
//! It uses egui textures only, so it works the same with the wgpu and the OpenGL (glow) renderer.

mod checker;
mod levels;
mod renderer;
mod tile_image;

pub use checker::paint_checkerboard;
pub use levels::{level_for_zoom, level_span};
pub use renderer::{CanvasRenderer, RenderStats, ViewTransform};
pub use tile_image::render_tile;
