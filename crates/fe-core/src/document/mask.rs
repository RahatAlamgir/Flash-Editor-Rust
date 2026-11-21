//! Layer masks: 0 hides the layer, 255 shows it, values between are see-through.
//! Stored in sparse 1-byte tiles; untouched areas cost no memory.

use crate::imaging::{PixelBuffer, PixelRect, TileGrid, TileKey};
use crate::selection::Selection;

/// A canvas-sized mask (see the module docs).
#[derive(Clone, Debug)]
pub struct LayerMask {
    width: u32,
    height: u32,
    alpha: TileGrid<1>,
}

impl LayerMask {
    /// A mask where every pixel is `fill` (255 = all visible, 0 = all hidden).
    pub fn new(width: u32, height: u32, fill: u8) -> Self {
        Self { width, height, alpha: TileGrid::new([fill]) }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bounds(&self) -> PixelRect {
        PixelRect::new(0, 0, self.width as i32, self.height as i32)
    }

    /// The raw tiles (for drawing and saving).
    pub fn tiles(&self) -> &TileGrid<1> {
        &self.alpha
    }

    /// The raw tiles to change (the Mask Brush paints on them).
    pub fn tiles_mut(&mut self) -> &mut TileGrid<1> {
        &mut self.alpha
    }

    /// A mask made from a selection: shows only the selected area (`reveal_selected`), or hides it.
    pub fn from_selection(selection: &Selection, reveal_selected: bool) -> Self {
        let (w, h) = (selection.canvas_width(), selection.canvas_height());
        let mut mask = Self::new(w, h, if reveal_selected { 0 } else { 255 });
        let b = selection.bounds();
        let values: Vec<u8> = selection.mask().iter().map(|v| if reveal_selected { *v } else { 255 - *v }).collect();
        mask.alpha.write_rect(b, &values);
        mask
    }

    /// Hides (or shows) the selected area on top of what the mask already says (C# `ApplySelection`).
    pub fn apply_selection(&mut self, selection: &Selection, reveal: bool) {
        let area = selection.bounds().intersect(&self.bounds());
        let mut values = self.alpha.read_rect(area);
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let s = selection.value(x, y) as u32;
                if s == 0 {
                    continue;
                }
                let v = &mut values[((y - area.y) * area.width + (x - area.x)) as usize];
                let a = *v as u32;
                *v = if reveal { (a + (255 - a) * s / 255) as u8 } else { (a - a * s / 255) as u8 };
            }
        }
        self.alpha.write_rect(area, &values);
    }

    /// Multiplies this mask by another one of the same size (both must show for the result to show).
    pub fn multiply(&mut self, other: &LayerMask) {
        let area = self.bounds();
        let mut values = self.alpha.read_rect(area);
        for (v, o) in values.iter_mut().zip(other.alpha.read_rect(area)) {
            *v = (*v as u32 * o as u32 / 255) as u8;
        }
        let fill = (self.fill_value() as u32 * other.fill_value() as u32 / 255) as u8;
        let mut result = TileGrid::new([fill]);
        result.write_rect(area, &values);
        result.prune();
        self.alpha = result;
    }

    /// Memory used by stored tiles.
    pub fn memory_bytes(&self) -> usize {
        self.alpha.memory_bytes()
    }

    /// The mask value at (x, y); outside the canvas reads as fully hidden.
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if self.bounds().contains(x, y) { self.alpha.get(x, y)[0] } else { 0 }
    }

    pub fn set(&mut self, x: i32, y: i32, value: u8) {
        if self.bounds().contains(x, y) {
            self.alpha.set(x, y, [value]);
        }
    }

    /// How much of the layer shows at (x, y), 0..1.
    pub fn coverage(&self, x: i32, y: i32) -> f64 {
        self.get(x, y) as f64 / 255.0
    }

    /// True when the whole mask is 255.
    pub fn is_all_visible(&self) -> bool {
        if self.alpha.fill_value()[0] == 255 {
            // Only stored tiles can differ; check just their part on the canvas.
            let bounds = self.bounds();
            return self
                .alpha
                .keys()
                .all(|k| self.alpha.read_rect(k.rect().intersect(&bounds)).iter().all(|&v| v == 255));
        }
        self.alpha.read_rect(self.bounds()).iter().all(|&v| v == 255)
    }

    /// Hidden becomes visible and the other way round.
    pub fn invert(&mut self) {
        let fill = 255 - self.alpha.fill_value()[0];
        let mut inverted = TileGrid::new([fill]);
        for key in self.alpha.keys().collect::<Vec<_>>() {
            if let Some(tile) = self.alpha.tile(key) {
                let flipped: Vec<u8> = tile.iter().map(|v| 255 - v).collect();
                inverted.set_tile(key, Some(flipped.into()));
            }
        }
        self.alpha = inverted;
    }

    /// The mask values of a rectangle (clipped to the canvas), row by row.
    pub fn copy_region(&self, region: PixelRect) -> Vec<u8> {
        self.alpha.read_rect(region.intersect(&self.bounds()))
    }

    /// Writes values copied with [`Self::copy_region`] back.
    pub fn paste_region(&mut self, region: PixelRect, data: &[u8]) {
        self.alpha.write_rect(region.intersect(&self.bounds()), data);
    }

    /// The mask as an opaque grey picture (how projects save it).
    pub fn to_gray_picture(&self) -> PixelBuffer {
        let values = self.alpha.read_rect(self.bounds());
        let mut picture = PixelBuffer::new(self.width, self.height);
        for (p, v) in picture.data_mut().as_chunks_mut::<4>().0.iter_mut().zip(values) {
            p.copy_from_slice(&[v, v, v, 255]);
        }
        picture
    }

    /// Reads a mask saved as a grey picture (the green channel is used).
    pub fn from_gray_picture(picture: &PixelBuffer) -> Self {
        let mut mask = Self::new(picture.width(), picture.height(), 255);
        let values: Vec<u8> = picture.data().as_chunks::<4>().0.iter().map(|p| p[1]).collect();
        mask.alpha.write_rect(mask.bounds(), &values);
        mask.alpha.prune();
        mask
    }

    /// The mask value for each pixel of one tile (`None` = the whole tile is the fill value).
    pub fn tile_values(&self, key: TileKey) -> Option<&[u8]> {
        self.alpha.tile(key)
    }

    /// The value of pixels in tiles that are not stored.
    pub fn fill_value(&self) -> u8 {
        self.alpha.fill_value()[0]
    }

    /// `source` with the mask baked into its alpha (pixel (0,0) of `source` sits at canvas (offset_x, offset_y)).
    pub fn bake(&self, source: &PixelBuffer, offset_x: i32, offset_y: i32) -> PixelBuffer {
        let mut result = source.clone();
        for y in 0..result.height() as i32 {
            for x in 0..result.width() as i32 {
                let m = self.get(x + offset_x, y + offset_y) as u32;
                if m == 255 {
                    continue;
                }
                let i = result.index_of(x as u32, y as u32) + 3;
                let d = result.data_mut();
                d[i] = (d[i] as u32 * m / 255) as u8;
            }
        }
        result
    }
}
