//! Zoom levels: level 0 = full size tiles; level n = tiles that cover 2^n times more canvas, shrunk to the
//! same 256 px texture.

/// Size of one screen tile (texture) at level 0. Separate from the layers' storage tiles (`fe_core`), which
/// are smaller to waste less memory on small objects.
pub const RENDER_TILE: i32 = 256;

/// The deepest level (1/64 size): enough for 16384 px pictures on a small window.
pub const MAX_LEVEL: u32 = 6;

/// The level to use at a zoom: the smallest copy that is still at least as sharp as the screen.
pub fn level_for_zoom(zoom: f32) -> u32 {
    if zoom >= 1.0 || zoom <= 0.0 {
        return 0;
    }
    ((1.0 / zoom).log2().floor() as u32).min(MAX_LEVEL)
}

/// How many canvas pixels one tile covers (along each side) at a level.
pub fn level_span(level: u32) -> i32 {
    RENDER_TILE << level
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_the_zoom() {
        assert_eq!(0, level_for_zoom(4.0));
        assert_eq!(0, level_for_zoom(1.0));
        assert_eq!(0, level_for_zoom(0.6)); // shrinking up to 2x stays on full size tiles
        assert_eq!(1, level_for_zoom(0.5));
        assert_eq!(1, level_for_zoom(0.3));
        assert_eq!(2, level_for_zoom(0.25));
        assert_eq!(MAX_LEVEL, level_for_zoom(0.001));
        assert_eq!(1024, level_span(2));
    }
}
