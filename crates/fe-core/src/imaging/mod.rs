//! Tiles, pixel buffers, colors, rectangles, blend math and resampling.
//!
//! Pixels are 8-bit RGBA with straight (not premultiplied) alpha, like the C# version, so blend results match.

mod blend;
mod blend_mode;
mod buffer;
mod color;
pub mod color_math;
pub mod palette;
mod point;
mod rect;
mod thumbnail;
mod tile_grid;
mod tiles;

pub use blend::{blend_channel, blend_pixel, lerp_pixel, over, over_pixel};
pub use blend_mode::BlendMode;
pub use buffer::PixelBuffer;
pub use color::ColorRgba;
pub use point::PointD;
pub use rect::PixelRect;
pub use thumbnail::{create_thumbnail, fill_thumbnail, fit_size};
pub use tile_grid::{TILE_SIZE, TileGrid, TileKey};
pub use tiles::TileMap;
