//! `TileMap`: the RGBA pixels of a layer, stored as sparse shared tiles (see [`super::TileGrid`]).

use super::{ColorRgba, PixelBuffer, PixelRect, TILE_SIZE, TileGrid, TileKey};

/// RGBA pixels in sparse tiles. Missing tiles are fully transparent.
pub type TileMap = TileGrid<4>;

impl Default for TileGrid<4> {
    fn default() -> Self {
        Self::new([0; 4])
    }
}

impl TileGrid<4> {
    /// An empty (fully transparent) pixel map.
    pub fn empty() -> Self {
        Self::default()
    }

    /// A map holding `buffer` with its top-left corner at (x, y).
    pub fn from_buffer(buffer: &PixelBuffer, x: i32, y: i32) -> Self {
        let mut map = Self::empty();
        map.write_buffer(buffer, x, y);
        map
    }

    /// Writes `buffer` with its top-left corner at (x, y), replacing what was there.
    pub fn write_buffer(&mut self, buffer: &PixelBuffer, x: i32, y: i32) {
        let rect = PixelRect::new(x, y, buffer.width() as i32, buffer.height() as i32);
        self.write_rect(rect, buffer.data());
    }

    /// A dense copy of a rectangle (transparent where nothing is stored).
    pub fn to_buffer(&self, rect: PixelRect) -> PixelBuffer {
        let data = self.read_rect(rect);
        PixelBuffer::from_raw(rect.width as u32, rect.height as u32, data).expect("size matches the rectangle")
    }

    pub fn pixel(&self, x: i32, y: i32) -> ColorRgba {
        let [r, g, b, a] = self.get(x, y);
        ColorRgba::new(r, g, b, a)
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: ColorRgba) {
        self.set(x, y, color.to_array());
    }

    /// Fills a rectangle with one color.
    pub fn fill_rect(&mut self, rect: PixelRect, color: ColorRgba) {
        if color.a == 0 {
            self.clear_rect(rect);
            return;
        }
        let c = color.to_array();
        for key in TileKey::covering(rect) {
            let part = key.rect().intersect(&rect);
            let tr = key.rect();
            let tile = self.tile_mut(key);
            for y in part.y..part.bottom() {
                let i = (((y - tr.y) * TILE_SIZE + (part.x - tr.x)) as usize) * 4;
                tile[i..i + part.width as usize * 4]
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .for_each(|p| p.copy_from_slice(&c));
            }
        }
    }

    /// The exact smallest rectangle holding every pixel that is not fully transparent (empty if none).
    /// May lie partly or fully outside the canvas.
    pub fn visible_bounds(&self) -> PixelRect {
        let mut bounds = PixelRect::EMPTY;
        for key in self.keys() {
            let Some(tile) = self.tile(key) else { continue };
            let Some(buffer) = PixelBuffer::from_raw(TILE_SIZE as u32, TILE_SIZE as u32, tile.to_vec()) else {
                continue;
            };
            let local = buffer.visible_bounds();
            if !local.is_empty() {
                bounds = bounds.union(&local.offset(key.rect().x, key.rect().y));
            }
        }
        bounds
    }

    /// Moves all pixels by whole pixels. Tile-aligned moves just re-label tiles; others copy pixels.
    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        if dx == 0 && dy == 0 {
            return self.clone();
        }
        if dx % TILE_SIZE == 0 && dy % TILE_SIZE == 0 {
            let mut moved = Self::empty();
            for key in self.keys() {
                let to = TileKey::new(key.tx + dx / TILE_SIZE, key.ty + dy / TILE_SIZE);
                moved.set_tile(to, self.tile_arc(key).cloned());
            }
            return moved;
        }
        let mut moved = Self::empty();
        for key in self.keys() {
            let rect = key.rect();
            moved.write_rect(rect.offset(dx, dy), &self.read_rect(rect));
        }
        moved
    }
}
