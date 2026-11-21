//! Makes the picture of one tile at one level: combine the layers for the tile's canvas area, then shrink it
//! by 2^level (box filter, weighted by alpha so see-through edges do not darken).

use egui::ColorImage;
use fe_core::document::Document;
use fe_core::imaging::{PixelBuffer, PixelRect, TileKey};
use fe_core::paint::compositor;

use crate::levels::level_span;

/// The canvas area a tile covers at a level (not clipped).
pub fn tile_area(level: u32, key: TileKey) -> PixelRect {
    let span = level_span(level);
    PixelRect::new(key.tx * span, key.ty * span, span, span)
}

/// The picture of tile `key` at `level` and the canvas area it shows (clipped to the canvas), or `None` when
/// the tile is outside the canvas.
pub fn render_tile(doc: &Document, level: u32, key: TileKey) -> Option<(ColorImage, PixelRect)> {
    let area = tile_area(level, key).intersect(&doc.bounds());
    let full = compositor::composite_region(doc, area)?;
    let picture = if level == 0 { full } else { shrink(&full, 1 << level) };
    let size = [picture.width() as usize, picture.height() as usize];
    Some((ColorImage::from_rgba_unmultiplied(size, picture.data()), area))
}

/// Shrinks a picture by `factor` (a power of two) with an alpha-weighted box filter.
fn shrink(source: &PixelBuffer, factor: u32) -> PixelBuffer {
    let w = source.width().div_ceil(factor);
    let h = source.height().div_ceil(factor);
    let mut out = PixelBuffer::new(w, h);
    let src = source.data();
    let sw = source.width() as usize;
    for oy in 0..h {
        let y0 = (oy * factor) as usize;
        let y1 = ((oy + 1) * factor).min(source.height()) as usize;
        for ox in 0..w {
            let x0 = (ox * factor) as usize;
            let x1 = ((ox + 1) * factor).min(source.width()) as usize;
            let (mut r, mut g, mut b, mut a, mut n) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * sw + x) * 4;
                    let alpha = src[i + 3] as u64;
                    r += src[i] as u64 * alpha;
                    g += src[i + 1] as u64 * alpha;
                    b += src[i + 2] as u64 * alpha;
                    a += alpha;
                    n += 1;
                }
            }
            let o = out.index_of(ox, oy);
            let px = &mut out.data_mut()[o..o + 4];
            if let (Some(r), Some(g), Some(b), Some(alpha)) =
                (r.checked_div(a), g.checked_div(a), b.checked_div(a), a.checked_div(n))
            {
                px.copy_from_slice(&[r as u8, g as u8, b as u8, alpha as u8]);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use fe_core::imaging::ColorRgba;

    #[test]
    fn a_level_2_tile_is_a_quarter_size_copy_clipped_to_the_canvas() {
        let doc = Document::blank(1500, 700, ColorRgba::rgb(10, 20, 30));
        let (image, area) = render_tile(&doc, 2, TileKey::new(1, 0)).expect("inside");
        assert_eq!(PixelRect::new(1024, 0, 476, 700), area);
        assert_eq!([119, 175], image.size);
        assert_eq!(egui::Color32::from_rgb(10, 20, 30), image.pixels[0]);
        assert!(render_tile(&doc, 2, TileKey::new(2, 0)).is_none());
    }
}
