//! A plain, dense RGBA picture. Used for small or temporary images (thumbnails, export, decoding).
//! Layers keep their pixels in a [`super::TileMap`] instead.

use super::{ColorRgba, PixelRect};

/// A dense RGBA image, rows top to bottom, 4 bytes per pixel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelBuffer {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl PixelBuffer {
    /// A fully transparent picture. Panics if a side is 0.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "invalid image size {width}x{height}");
        Self { width, height, data: vec![0; width as usize * height as usize * 4] }
    }

    /// Wraps existing RGBA bytes; `None` when the length does not match the size.
    pub fn from_raw(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        (width > 0 && height > 0 && data.len() == width as usize * height as usize * 4).then_some(Self {
            width,
            height,
            data,
        })
    }

    /// A picture filled with one color.
    pub fn filled(width: u32, height: u32, color: ColorRgba) -> Self {
        let mut buffer = Self::new(width, height);
        buffer.fill(color);
        buffer
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

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn into_raw(self) -> Vec<u8> {
        self.data
    }

    pub fn stride(&self) -> usize {
        self.width as usize * 4
    }

    /// Byte index of pixel (x, y).
    pub fn index_of(&self, x: u32, y: u32) -> usize {
        (y as usize * self.width as usize + x as usize) * 4
    }

    pub fn pixel(&self, x: u32, y: u32) -> ColorRgba {
        ColorRgba::from_slice(&self.data[self.index_of(x, y)..])
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: ColorRgba) {
        let i = self.index_of(x, y);
        self.data[i..i + 4].copy_from_slice(&color.to_array());
    }

    /// One row of pixels as bytes.
    pub fn row(&self, y: u32) -> &[u8] {
        let s = self.stride();
        &self.data[y as usize * s..(y as usize + 1) * s]
    }

    pub fn row_mut(&mut self, y: u32) -> &mut [u8] {
        let s = self.stride();
        &mut self.data[y as usize * s..(y as usize + 1) * s]
    }

    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    pub fn fill(&mut self, color: ColorRgba) {
        let c = color.to_array();
        self.data.as_chunks_mut::<4>().0.iter_mut().for_each(|p| p.copy_from_slice(&c));
    }

    /// Makes a rectangle transparent (clipped to the picture).
    pub fn clear_region(&mut self, rect: PixelRect) {
        let r = rect.intersect(&self.bounds());
        for y in r.y..r.bottom() {
            let i = self.index_of(r.x as u32, y as u32);
            self.data[i..i + r.width as usize * 4].fill(0);
        }
    }

    /// A copy of a rectangle; parts outside the picture are transparent.
    pub fn crop(&self, rect: PixelRect) -> Self {
        let mut result = Self::new(rect.width as u32, rect.height as u32);
        result.copy_from(self, rect, 0, 0);
        result
    }

    /// Copies `source_rect` of `source` so that its top-left lands at (dest_x, dest_y). Clipped on both sides.
    pub fn copy_from(&mut self, source: &Self, source_rect: PixelRect, dest_x: i32, dest_y: i32) {
        let src = source_rect.intersect(&source.bounds());
        if src.is_empty() {
            return;
        }
        let dx = dest_x + src.x - source_rect.x;
        let dy = dest_y + src.y - source_rect.y;
        let dst = PixelRect::new(dx, dy, src.width, src.height).intersect(&self.bounds());
        if dst.is_empty() {
            return;
        }
        let sx = src.x + (dst.x - dx);
        let sy = src.y + (dst.y - dy);
        let bytes = dst.width as usize * 4;
        for row in 0..dst.height {
            let si = source.index_of(sx as u32, (sy + row) as u32);
            let di = self.index_of(dst.x as u32, (dst.y + row) as u32);
            self.data[di..di + bytes].copy_from_slice(&source.data[si..si + bytes]);
        }
    }

    /// Copies all of `source` with its top-left at (dest_x, dest_y).
    pub fn copy_all_from(&mut self, source: &Self, dest_x: i32, dest_y: i32) {
        self.copy_from(source, source.bounds(), dest_x, dest_y);
    }

    /// The smallest rectangle holding every pixel that is not fully transparent (empty if none).
    pub fn visible_bounds(&self) -> PixelRect {
        let (w, h) = (self.width as i32, self.height as i32);
        let visible_row = |y: i32| self.row(y as u32).as_chunks::<4>().0.iter().any(|p| p[3] != 0);
        let Some(top) = (0..h).find(|&y| visible_row(y)) else {
            return PixelRect::EMPTY;
        };
        let bottom = (top..h).rev().find(|&y| visible_row(y)).unwrap_or(top);
        let (mut min_x, mut max_x) = (w, -1);
        for y in top..=bottom {
            let row = self.row(y as u32);
            if let Some(x) = (0..min_x).find(|&x| row[x as usize * 4 + 3] != 0) {
                min_x = x;
            }
            if let Some(x) = ((max_x + 1)..w).rev().find(|&x| row[x as usize * 4 + 3] != 0) {
                max_x = x;
            }
        }
        PixelRect::new(min_x, top, max_x - min_x + 1, bottom - top + 1)
    }
}
