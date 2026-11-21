//! AI upscaling / clean-up with Real-ESRGAN (C# `AiUpscaler`). The 4x model works on fixed 128 x 128 tiles, so the
//! picture is processed tile by tile with overlaps that are blended smoothly (no seams). Any output scale works: the 4x
//! result of each tile is resized to the wanted scale (1 = same size, which gives a cleaner, sharper picture).
//! Transparency is enlarged separately (the model only does color).

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use fe_core::imaging::PixelBuffer;
use fe_core::imaging::color_math::clamp_byte;
use fe_core::transform::{ResampleMode, resize};
use ort::value::Tensor;

use crate::models::ModelInfo;
use crate::runtime::{self, Opened};

const MODEL_SCALE: usize = 4;
/// Input pixels shared with neighboring tiles.
const OVERLAP: usize = 16;

pub struct AiUpscaler {
    path: PathBuf,
    tile: usize,
    session: Mutex<Option<Opened>>,
}

/// 1 in the middle of a tile, fading to almost 0 at edges that overlap a neighbor.
fn edge_weight(i: usize, size: usize, fade: f64, fade_start: bool, fade_end: bool) -> f64 {
    let mut w: f64 = 1.0;
    if fade_start {
        w = w.min((i as f64 + 0.5) / fade);
    }
    if fade_end {
        w = w.min((size as f64 - i as f64 - 0.5) / fade);
    }
    w.clamp(0.001, 1.0)
}

impl AiUpscaler {
    pub fn new(model: &'static ModelInfo, path: PathBuf) -> Self {
        Self { path, tile: model.input_size, session: Mutex::new(None) }
    }

    fn step(&self) -> usize {
        self.tile - 2 * OVERLAP
    }

    /// Tile start positions along one side (the last tile is aligned to the end).
    fn positions(&self, length: usize) -> Vec<usize> {
        let mut list = Vec::new();
        if length <= self.tile {
            return vec![0];
        }
        let mut p = 0;
        loop {
            if p + self.tile >= length {
                list.push(length - self.tile);
                break;
            }
            list.push(p);
            p += self.step();
        }
        list
    }

    /// How many tiles a picture needs (for a time estimate).
    pub fn tile_count(&self, width: usize, height: usize) -> usize {
        self.positions(width).len() * self.positions(height).len()
    }

    pub fn device_name(&self) -> String {
        self.session.lock().ok().and_then(|s| s.as_ref().map(|o| o.device.to_owned())).unwrap_or_default()
    }

    /// `scale`: output size factor (1 = same size, 2 = twice as large...). `progress` gets 0..1.
    pub fn upscale(
        &self,
        source: &PixelBuffer,
        scale: f64,
        progress: &dyn Fn(f64),
        cancel: &AtomicBool,
    ) -> Result<PixelBuffer, String> {
        let (w, h) = (source.width() as usize, source.height() as usize);
        let out_w = ((w as f64 * scale).round() as usize).max(1);
        let out_h = ((h as f64 * scale).round() as usize).max(1);
        let mut color = vec![0f32; out_w * out_h * 3];
        let mut weight = vec![0f32; out_w * out_h];
        let (xs, ys) = (self.positions(w), self.positions(h));
        let total = xs.len() * ys.len();
        let mut done = 0;
        for &y0 in &ys {
            for &x0 in &xs {
                if cancel.load(Ordering::Relaxed) {
                    return Err("Cancelled.".into());
                }
                let output = self.run_tile(source, x0, y0)?; // tile * 4 square, RGB 0..1, planar
                self.blend(&output, (x0, y0), scale, (w, h), (out_w, out_h), &mut color, &mut weight);
                done += 1;
                progress(done as f64 / total as f64);
            }
        }
        // Transparency: a smooth resize of the alpha channel (the model only handles color).
        let alpha = resize(source, out_w as u32, out_h as u32, ResampleMode::Lanczos);
        let mut result = PixelBuffer::new(out_w as u32, out_h as u32);
        for (p, px) in result.data_mut().chunks_exact_mut(4).enumerate() {
            let wsum = weight[p].max(1e-6);
            px[0] = clamp_byte(f64::from(color[p * 3] / wsum * 255.0));
            px[1] = clamp_byte(f64::from(color[p * 3 + 1] / wsum * 255.0));
            px[2] = clamp_byte(f64::from(color[p * 3 + 2] / wsum * 255.0));
            px[3] = alpha.data()[p * 4 + 3];
        }
        Ok(result)
    }

    fn run_tile(&self, source: &PixelBuffer, x0: usize, y0: usize) -> Result<Vec<f32>, String> {
        let tile = self.tile;
        let plane = tile * tile;
        let mut input = vec![0f32; 3 * plane];
        let (sw, sh) = (source.width() as usize, source.height() as usize);
        for y in 0..tile {
            let sy = (y0 + y).min(sh - 1);
            for x in 0..tile {
                let sx = (x0 + x).min(sw - 1); // small pictures: edges repeat
                let i = source.index_of(sx as u32, sy as u32);
                let p = y * tile + x;
                let d = source.data();
                input[p] = f32::from(d[i]) / 255.0;
                input[plane + p] = f32::from(d[i + 1]) / 255.0;
                input[2 * plane + p] = f32::from(d[i + 2]) / 255.0;
            }
        }
        let mut guard = self.session.lock().map_err(|_| "The AI model is busy.".to_owned())?;
        if guard.is_none() {
            *guard = Some(runtime::open(&self.path, true)?);
        }
        let opened = guard.as_mut().ok_or("The AI model is not loaded.")?;
        let name = opened.session.inputs[0].name.clone();
        let tensor = Tensor::from_array(([1usize, 3, tile, tile], input)).map_err(|e| e.to_string())?;
        let outputs =
            opened.session.run(ort::inputs![name => tensor]).map_err(|e| format!("The AI model failed: {e}"))?;
        let (_, data) = outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        Ok(data.to_vec())
    }

    /// Adds one tile's result (resized to the output scale) into the sums, with weights that fade out toward edges
    /// shared with other tiles, so overlapping tiles blend without visible seams.
    #[allow(clippy::too_many_arguments)]
    fn blend(
        &self,
        output: &[f32],
        origin: (usize, usize),
        scale: f64,
        src: (usize, usize),
        out: (usize, usize),
        color: &mut [f32],
        weight: &mut [f32],
    ) {
        let big = self.tile * MODEL_SCALE;
        let big_plane = big * big;
        let fade = OVERLAP as f64 * scale;
        let left = (origin.0 as f64 * scale).round() as i64;
        let top = (origin.1 as f64 * scale).round() as i64;
        let size = ((self.tile as f64 * scale).round() as usize).max(1);
        let ratio = big as f64 / size as f64;
        let (has_left, has_top) = (origin.0 > 0, origin.1 > 0);
        let (has_right, has_bottom) = (origin.0 + self.tile < src.0, origin.1 + self.tile < src.1);
        for y in 0..size {
            let oy = top + y as i64;
            if oy < 0 || oy >= out.1 as i64 {
                continue;
            }
            let wy = edge_weight(y, size, fade, has_top, has_bottom);
            let (sy0, sy1) = (y as f64 * ratio, y as f64 * ratio + ratio);
            let iy0 = sy0 as usize;
            let iy1 = big.min((iy0 + 1).max(sy1.ceil() as usize));
            for x in 0..size {
                let ox = left + x as i64;
                if ox < 0 || ox >= out.0 as i64 {
                    continue;
                }
                let wgt = (wy * edge_weight(x, size, fade, has_left, has_right)) as f32;
                let (sx0, sx1) = (x as f64 * ratio, x as f64 * ratio + ratio);
                let ix0 = sx0 as usize;
                let ix1 = big.min((ix0 + 1).max(sx1.ceil() as usize));
                // Area-average the 4x result down to this output pixel (or sample it at 4x).
                let (mut r, mut g, mut b, mut n) = (0.0f64, 0.0f64, 0.0f64, 0u32);
                for yy in iy0..iy1 {
                    for xx in ix0..ix1 {
                        let q = yy * big + xx;
                        r += f64::from(output[q]);
                        g += f64::from(output[big_plane + q]);
                        b += f64::from(output[2 * big_plane + q]);
                        n += 1;
                    }
                }
                let n = f64::from(n.max(1));
                let p = oy as usize * out.0 + ox as usize;
                color[p * 3] += (r / n) as f32 * wgt;
                color[p * 3 + 1] += (g / n) as f32 * wgt;
                color[p * 3 + 2] += (b / n) as f32 * wgt;
                weight[p] += wgt;
            }
        }
    }

    pub fn release(&self) {
        if let Ok(mut s) = self.session.lock() {
            *s = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::REAL_ESRGAN;

    #[test]
    fn tiles_cover_the_picture_and_the_last_one_is_aligned_to_the_end() {
        let up = AiUpscaler::new(&REAL_ESRGAN, PathBuf::new());
        assert_eq!(up.positions(100), vec![0]);
        assert_eq!(up.positions(128), vec![0]);
        let p = up.positions(300);
        assert_eq!(*p.first().unwrap(), 0);
        assert_eq!(*p.last().unwrap(), 300 - 128);
        assert!(p.windows(2).all(|w| w[1] - w[0] <= 128 - OVERLAP));
        assert_eq!(up.tile_count(300, 100), p.len());
    }

    #[test]
    fn edges_fade_only_where_tiles_overlap() {
        assert_eq!(edge_weight(0, 100, 16.0, false, false), 1.0);
        assert!(edge_weight(0, 100, 16.0, true, false) < 0.1);
        assert!(edge_weight(99, 100, 16.0, false, true) < 0.1);
    }
}
