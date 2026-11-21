//! A sparse grid of 256 x 256 tiles. This is how layers keep their pixels without wasting memory:
//!
//! * a tile that was never painted is not stored at all (it reads as the grid's `fill` value);
//! * tiles are shared with `Arc`, so copies (undo steps, duplicated layers, pages) cost almost nothing until
//!   one side changes a tile — then only that tile is copied ("copy on write");
//! * coordinates may be negative or past the canvas, so pixels moved off the canvas are simply kept.
//!
//! `C` is the number of bytes per pixel: 4 for RGBA pixels, 1 for masks.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::PixelRect;

/// Width and height of one tile in pixels. 64 px measured best: small objects waste little memory (the C#
/// tutorial: 114 MB of tiles at 64 px vs 288 MB at 256 px) and drawing is just as fast.
pub const TILE_SIZE: i32 = 64;
const TILE_PIXELS: usize = (TILE_SIZE * TILE_SIZE) as usize;

/// Which tile: tile column and row (may be negative).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileKey {
    pub tx: i32,
    pub ty: i32,
}

impl TileKey {
    pub const fn new(tx: i32, ty: i32) -> Self {
        Self { tx, ty }
    }

    /// The tile holding pixel (x, y) and the pixel's position inside it.
    pub fn containing(x: i32, y: i32) -> (Self, usize, usize) {
        let key = Self::new(x.div_euclid(TILE_SIZE), y.div_euclid(TILE_SIZE));
        (key, x.rem_euclid(TILE_SIZE) as usize, y.rem_euclid(TILE_SIZE) as usize)
    }

    /// The canvas area this tile covers.
    pub const fn rect(&self) -> PixelRect {
        PixelRect::new(self.tx * TILE_SIZE, self.ty * TILE_SIZE, TILE_SIZE, TILE_SIZE)
    }

    /// Every tile that touches `rect`, row by row.
    pub fn covering(rect: PixelRect) -> impl Iterator<Item = TileKey> {
        let (tx0, ty0, tx1, ty1) = if rect.is_empty() {
            (0, 0, -1, -1)
        } else {
            (
                rect.x.div_euclid(TILE_SIZE),
                rect.y.div_euclid(TILE_SIZE),
                (rect.right() - 1).div_euclid(TILE_SIZE),
                (rect.bottom() - 1).div_euclid(TILE_SIZE),
            )
        };
        (ty0..=ty1).flat_map(move |ty| (tx0..=tx1).map(move |tx| TileKey::new(tx, ty)))
    }
}

/// A stored tile and its change stamp.
#[derive(Clone, Debug)]
struct Stored {
    data: Arc<Vec<u8>>,
    /// A number no other tile content ever had: it changes every time the tile may have changed.
    stamp: u64,
}

/// A new, never used change stamp.
fn next_stamp() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Sparse tiles with `C` bytes per pixel. Missing tiles read as `fill`.
///
/// Every stored tile carries a change stamp (see [`Self::tile_stamp`]): caches such as the canvas textures
/// compare stamps to know exactly which tiles to redraw. Copies of a grid share stamps with their source
/// until one side changes a tile.
#[derive(Clone, Debug)]
pub struct TileGrid<const C: usize> {
    tiles: HashMap<TileKey, Stored>,
    fill: [u8; C],
}

impl<const C: usize> TileGrid<C> {
    /// Bytes in one tile.
    pub const TILE_BYTES: usize = TILE_PIXELS * C;

    /// An empty grid where every pixel reads as `fill`.
    pub fn new(fill: [u8; C]) -> Self {
        Self { tiles: HashMap::new(), fill }
    }

    /// The value of pixels that were never written.
    pub fn fill_value(&self) -> [u8; C] {
        self.fill
    }

    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Memory held by the tiles (shared tiles are counted for every grid that holds them).
    pub fn memory_bytes(&self) -> usize {
        self.tiles.len() * Self::TILE_BYTES
    }

    pub fn keys(&self) -> impl Iterator<Item = TileKey> + '_ {
        self.tiles.keys().copied()
    }

    /// The stored tile, if any (`None` = every pixel is `fill`).
    pub fn tile(&self, key: TileKey) -> Option<&[u8]> {
        self.tiles.get(&key).map(|t| t.data.as_slice())
    }

    /// The shared handle of a tile (for cheap snapshots and "did it change?" checks).
    pub fn tile_arc(&self, key: TileKey) -> Option<&Arc<Vec<u8>>> {
        self.tiles.get(&key).map(|t| &t.data)
    }

    /// The change stamp of a tile (`None` = not stored, every pixel is `fill`). Equal stamps mean equal pixels.
    pub fn tile_stamp(&self, key: TileKey) -> Option<u64> {
        self.tiles.get(&key).map(|t| t.stamp)
    }

    /// Puts a tile in place (or removes it with `None`).
    pub fn set_tile(&mut self, key: TileKey, tile: Option<Arc<Vec<u8>>>) {
        match tile {
            Some(t) => {
                debug_assert_eq!(t.len(), Self::TILE_BYTES);
                self.tiles.insert(key, Stored { data: t, stamp: next_stamp() });
            }
            None => {
                self.tiles.remove(&key);
            }
        }
    }

    /// A tile to write into. Creates it (filled) when missing; copies it first when it is shared.
    pub fn tile_mut(&mut self, key: TileKey) -> &mut [u8] {
        let fill = self.fill;
        let tile =
            self.tiles.entry(key).or_insert_with(|| Stored { data: Arc::new(fill.repeat(TILE_PIXELS)), stamp: 0 });
        tile.stamp = next_stamp();
        Arc::make_mut(&mut tile.data).as_mut_slice()
    }

    pub fn get(&self, x: i32, y: i32) -> [u8; C] {
        let (key, lx, ly) = TileKey::containing(x, y);
        match self.tiles.get(&key) {
            Some(t) => {
                let i = (ly * TILE_SIZE as usize + lx) * C;
                t.data[i..i + C].try_into().unwrap_or(self.fill)
            }
            None => self.fill,
        }
    }

    pub fn set(&mut self, x: i32, y: i32, value: [u8; C]) {
        let (key, lx, ly) = TileKey::containing(x, y);
        if value == self.fill && !self.tiles.contains_key(&key) {
            return;
        }
        let i = (ly * TILE_SIZE as usize + lx) * C;
        self.tile_mut(key)[i..i + C].copy_from_slice(&value);
    }

    /// Copies a rectangle out into rows of `C`-byte pixels.
    pub fn read_rect(&self, rect: PixelRect) -> Vec<u8> {
        let mut out = self.fill.repeat(rect.area().max(0) as usize);
        let row_bytes = rect.width.max(0) as usize * C;
        for key in TileKey::covering(rect) {
            let Some(tile) = self.tile(key) else { continue };
            let part = key.rect().intersect(&rect);
            let tr = key.rect();
            let bytes = part.width as usize * C;
            for y in part.y..part.bottom() {
                let si = (((y - tr.y) * TILE_SIZE + (part.x - tr.x)) as usize) * C;
                let di = (y - rect.y) as usize * row_bytes + (part.x - rect.x) as usize * C;
                out[di..di + bytes].copy_from_slice(&tile[si..si + bytes]);
            }
        }
        out
    }

    /// Writes rows of `C`-byte pixels into a rectangle. Tiles that would stay all `fill` are not created.
    pub fn write_rect(&mut self, rect: PixelRect, data: &[u8]) {
        assert_eq!(data.len(), rect.area().max(0) as usize * C, "data does not match the rectangle");
        let row_bytes = rect.width.max(0) as usize * C;
        for key in TileKey::covering(rect) {
            let part = key.rect().intersect(&rect);
            let tr = key.rect();
            let bytes = part.width as usize * C;
            let src_row = |y: i32| {
                let si = (y - rect.y) as usize * row_bytes + (part.x - rect.x) as usize * C;
                &data[si..si + bytes]
            };
            if !self.tiles.contains_key(&key) && (part.y..part.bottom()).all(|y| self.is_fill(src_row(y))) {
                continue;
            }
            let tile = self.tile_mut(key);
            for y in part.y..part.bottom() {
                let di = (((y - tr.y) * TILE_SIZE + (part.x - tr.x)) as usize) * C;
                tile[di..di + bytes].copy_from_slice(src_row(y));
            }
        }
    }

    /// Sets a rectangle back to `fill`; tiles that become all `fill` are dropped.
    pub fn clear_rect(&mut self, rect: PixelRect) {
        let keys: Vec<_> = TileKey::covering(rect).filter(|k| self.tiles.contains_key(k)).collect();
        for key in keys {
            let part = key.rect().intersect(&rect);
            if part == key.rect() {
                self.tiles.remove(&key);
                continue;
            }
            let fill = self.fill;
            let tr = key.rect();
            let tile = self.tile_mut(key);
            for y in part.y..part.bottom() {
                let di = (((y - tr.y) * TILE_SIZE + (part.x - tr.x)) as usize) * C;
                tile[di..di + part.width as usize * C]
                    .as_chunks_mut::<C>()
                    .0
                    .iter_mut()
                    .for_each(|p| p.copy_from_slice(&fill));
            }
            self.drop_if_fill(key);
        }
    }

    /// Removes every tile whose pixels are all `fill` (frees memory after erasing).
    pub fn prune(&mut self) {
        let fill = self.fill;
        self.tiles.retain(|_, t| !t.data.as_chunks::<C>().0.iter().all(|p| *p == fill));
    }

    /// Removes one tile if all of its pixels are `fill`.
    pub fn drop_if_fill(&mut self, key: TileKey) {
        if self.tiles.get(&key).is_some_and(|t| self.is_fill(&t.data)) {
            self.tiles.remove(&key);
        }
    }

    /// The area covered by stored tiles (a quick, rough bound of the content).
    pub fn tile_bounds(&self) -> PixelRect {
        self.tiles.keys().fold(PixelRect::EMPTY, |acc, k| acc.union(&k.rect()))
    }

    /// True when both grids hold the very same tile objects (no pixel compare needed).
    pub fn shares_all_tiles_with(&self, other: &Self) -> bool {
        self.fill == other.fill
            && self.tiles.len() == other.tiles.len()
            && self.tiles.iter().all(|(k, t)| other.tiles.get(k).is_some_and(|o| Arc::ptr_eq(&t.data, &o.data)))
    }

    fn is_fill(&self, bytes: &[u8]) -> bool {
        bytes.as_chunks::<C>().0.iter().all(|p| *p == self.fill)
    }
}
