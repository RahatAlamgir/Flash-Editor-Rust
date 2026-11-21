//! Magic wand / paint bucket area: the pixels whose color is close to the clicked pixel (C# `FloodFill.BuildMask`).

use rayon::prelude::*;

use crate::imaging::PixelBuffer;

/// A mask (one byte per pixel of `image`, 255 = matches) of the pixels similar to the one at (x, y).
/// `tolerance`: 0 = exactly that color, 255 = everything. `contiguous`: only pixels connected to (x, y);
/// otherwise every matching pixel of the picture.
pub fn similar_area(image: &PixelBuffer, x: i32, y: i32, tolerance: u8, contiguous: bool) -> Vec<u8> {
    let (w, h) = (image.width() as usize, image.height() as usize);
    let mut mask = vec![0u8; w * h];
    if !image.bounds().contains(x, y) {
        return mask;
    }
    let seed = image.pixel(x as u32, y as u32).to_array();
    let data = image.data();
    let tol = tolerance as i32;
    let matches = |i: usize| -> bool {
        let p = &data[i * 4..i * 4 + 4];
        // Fully transparent pixels count as one color whatever their hidden RGB.
        if seed[3] == 0 && p[3] == 0 {
            return true;
        }
        (0..4).all(|c| (p[c] as i32 - seed[c] as i32).abs() <= tol)
    };

    if !contiguous {
        mask.par_chunks_mut(w).enumerate().for_each(|(py, row)| {
            for (px, m) in row.iter_mut().enumerate() {
                if matches(py * w + px) {
                    *m = 255;
                }
            }
        });
        return mask;
    }

    // Scanline flood fill: fill whole horizontal runs, then queue the rows above and below.
    let mut stack = vec![(x as usize, y as usize)];
    while let Some((sx, sy)) = stack.pop() {
        let row = sy * w;
        if mask[row + sx] != 0 || !matches(row + sx) {
            continue;
        }
        let (mut left, mut right) = (sx, sx);
        while left > 0 && mask[row + left - 1] == 0 && matches(row + left - 1) {
            left -= 1;
        }
        while right + 1 < w && mask[row + right + 1] == 0 && matches(row + right + 1) {
            right += 1;
        }
        for px in left..=right {
            mask[row + px] = 255;
            if sy > 0 && mask[row - w + px] == 0 && matches(row - w + px) {
                stack.push((px, sy - 1));
            }
            if sy + 1 < h && mask[row + w + px] == 0 && matches(row + w + px) {
                stack.push((px, sy + 1));
            }
        }
    }
    mask
}
