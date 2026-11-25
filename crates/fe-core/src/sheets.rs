//! Split Asset Sheet (C# `SheetSplitter`): splits an asset / sprite sheet into its separate items, in reading order (top
//! row first, left to right). With a selection, every separate selected area is one piece (cut out in the selection's
//! shape); without, items are found automatically on a transparent or plain-colored background.
#![allow(clippy::chunks_exact_to_as_chunks)] // plain 4-byte pixel chunks read best here

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;

use crate::imaging::{PixelBuffer, PixelRect};

/// Settings of [`split`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitSettings {
    /// Parts closer than this (pixels) belong to the same piece (an icon and its label).
    pub gap: i32,
    /// Pieces smaller than this (width and height, pixels) are ignored as specks.
    pub min_size: i32,
    /// How different from the background color a pixel must be to count as part of a piece (0..255).
    pub tolerance: i32,
    /// Make the sheet's background color transparent inside each piece.
    pub remove_background: bool,
    /// Empty space kept around each piece.
    pub padding: i32,
}

impl Default for SplitSettings {
    fn default() -> Self {
        Self { gap: 8, min_size: 8, tolerance: 30, remove_background: true, padding: 0 }
    }
}

/// One item found on the sheet: where it was, and its pixels.
#[derive(Clone, Debug)]
pub struct SheetPiece {
    pub bounds: PixelRect,
    pub pixels: PixelBuffer,
}

type Rgb = (f64, f64, f64);

/// Up to 6 dominant background colors along the picture border. A color only counts as background when it shows up on at
/// least two sides: a subject that touches one edge (a person cut off at the bottom) is not mistaken for background.
pub fn detect_border_colors(image: &PixelBuffer) -> Vec<Rgb> {
    let (w, h) = (image.width() as i32, image.height() as i32);
    struct Bucket {
        sum: [f64; 3],
        count: u32,
        per_side: [u32; 4],
    }
    let mut buckets: HashMap<u32, Bucket> = HashMap::new();
    let mut side_totals = [0u32; 4];
    let mut total = 0u32;
    let mut add = |x: i32, y: i32, side: usize| {
        let p = image.pixel(x as u32, y as u32);
        if p.a < 128 {
            return;
        }
        let key = (u32::from(p.r) >> 4) << 8 | (u32::from(p.g) >> 4) << 4 | (u32::from(p.b) >> 4); // 16 levels per channel
        let e = buckets.entry(key).or_insert(Bucket { sum: [0.0; 3], count: 0, per_side: [0; 4] });
        e.per_side[side] += 1;
        e.sum[0] += f64::from(p.r);
        e.sum[1] += f64::from(p.g);
        e.sum[2] += f64::from(p.b);
        e.count += 1;
        side_totals[side] += 1;
        total += 1;
    };
    for x in 0..w {
        add(x, 0, 0);
        if h > 1 {
            add(x, h - 1, 2);
        }
    }
    for y in 1..h - 1 {
        add(0, y, 3);
        if w > 1 {
            add(w - 1, y, 1);
        }
    }
    if total == 0 {
        return Vec::new();
    }
    // On how many sides does this color cover a noticeable share (3%) of that side?
    let sides = |per_side: &[u32; 4]| {
        (0..4).filter(|s| side_totals[*s] > 0 && f64::from(per_side[*s]) >= f64::from(side_totals[*s]) * 0.03).count()
    };
    let required = side_totals.iter().filter(|t| **t > 0).count().min(2);
    let mut chosen: Vec<&Bucket> = buckets
        .values()
        .filter(|e| f64::from(e.count) >= f64::from(total) * 0.02 && sides(&e.per_side) >= required)
        .collect();
    chosen.sort_by_key(|e| std::cmp::Reverse(e.count));
    let mut result: Vec<Rgb> = chosen
        .iter()
        .take(6)
        .map(|e| (e.sum[0] / f64::from(e.count), e.sum[1] / f64::from(e.count), e.sum[2] / f64::from(e.count)))
        .collect();
    if result.is_empty() {
        // A very noisy border: use its overall average.
        let (mut r, mut g, mut b, mut c) = (0.0, 0.0, 0.0, 0u32);
        for e in buckets.values() {
            r += e.sum[0];
            g += e.sum[1];
            b += e.sum[2];
            c += e.count;
        }
        result.push((r / f64::from(c), g / f64::from(c), b / f64::from(c)));
    }
    result
}

/// A mostly transparent border means "transparent background".
fn transparent_border(sheet: &PixelBuffer) -> bool {
    let (w, h) = (sheet.width(), sheet.height());
    let (mut clear, mut total) = (0u32, 0u32);
    let mut check = |x: u32, y: u32| {
        total += 1;
        if sheet.pixel(x, y).a < 24 {
            clear += 1;
        }
    };
    for x in 0..w {
        check(x, 0);
        check(x, h - 1);
    }
    for y in 0..h {
        check(0, y);
        check(w - 1, y);
    }
    clear > total / 2
}

/// True for the pixels that belong to an item, and the background color found on the border (if any).
pub fn foreground_mask(sheet: &PixelBuffer, tolerance: i32) -> (Vec<bool>, Option<Rgb>) {
    let (w, h) = (sheet.width() as usize, sheet.height() as usize);
    let background = if transparent_border(sheet) { None } else { detect_border_colors(sheet).into_iter().next() };
    let limit = f64::from(tolerance) * f64::from(tolerance) * 3.0;
    let mut mask = vec![false; w * h];
    mask.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for (x, m) in row.iter_mut().enumerate() {
            let p = sheet.pixel(x as u32, y as u32);
            if p.a < 24 {
                continue;
            }
            if let Some((r, g, b)) = background {
                let (dr, dg, db) = (f64::from(p.r) - r, f64::from(p.g) - g, f64::from(p.b) - b);
                if dr * dr + dg * dg + db * db <= limit {
                    continue;
                }
            }
            *m = true;
        }
    });
    (mask, background)
}

/// Square dilation by running counts (fast for any radius).
fn dilate(mask: &[bool], w: usize, h: usize, r: usize) -> Vec<bool> {
    let mut horizontal = vec![false; mask.len()];
    horizontal.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let row = &mask[y * w..(y + 1) * w];
        let mut count = row.iter().take(r.min(w)).filter(|m| **m).count();
        for x in 0..w {
            if x + r < w && row[x + r] {
                count += 1;
            }
            if x > r && row[x - r - 1] {
                count -= 1;
            }
            out[x] = count > 0;
        }
    });
    let mut result = vec![false; mask.len()];
    let columns: Vec<Vec<bool>> = (0..w)
        .into_par_iter()
        .map(|x| {
            let mut col = vec![false; h];
            let mut count = (0..r.min(h)).filter(|y| horizontal[y * w + x]).count();
            for (y, c) in col.iter_mut().enumerate() {
                if y + r < h && horizontal[(y + r) * w + x] {
                    count += 1;
                }
                if y > r && horizontal[(y - r - 1) * w + x] {
                    count -= 1;
                }
                *c = count > 0;
            }
            col
        })
        .collect();
    for (x, col) in columns.iter().enumerate() {
        for (y, v) in col.iter().enumerate() {
            result[y * w + x] = *v;
        }
    }
    result
}

/// Bounding boxes (of the real item pixels) of each connected group of the joined mask; `labels` gets each pixel's group
/// number (1, 2, 3...; 0 = none).
fn find_boxes(joined: &[bool], real: &[bool], w: usize, h: usize, labels: &mut [u32]) -> Vec<(PixelRect, u32)> {
    let mut seen = vec![false; joined.len()];
    let mut boxes = Vec::new();
    let mut label = 0u32;
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..joined.len() {
        if !joined[start] || seen[start] {
            continue;
        }
        let (mut min_x, mut min_y, mut max_x, mut max_y) = (usize::MAX, usize::MAX, 0usize, 0usize);
        let mut any = false;
        label += 1;
        seen[start] = true;
        stack.push(start);
        while let Some(p) = stack.pop() {
            labels[p] = label;
            let (x, y) = (p % w, p / w);
            if real[p] {
                any = true;
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
            for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
                for nx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                    let q = ny * w + nx;
                    if joined[q] && !seen[q] {
                        seen[q] = true;
                        stack.push(q);
                    }
                }
            }
        }
        if any {
            boxes.push((
                PixelRect::new(min_x as i32, min_y as i32, (max_x - min_x + 1) as i32, (max_y - min_y + 1) as i32),
                label,
            ));
        }
    }
    boxes
}

/// Rows (pieces that overlap vertically) top to bottom, each row left to right.
type Found = (PixelRect, u32);

fn reading_order(mut boxes: Vec<Found>) -> Vec<Found> {
    boxes.sort_by_key(|(b, _)| b.y);
    let mut rows: Vec<(i32, i32, Vec<Found>)> = Vec::new();
    for item in boxes {
        let b = item.0;
        match rows.iter().position(|r| b.y < r.1 && b.y + b.height > r.0) {
            None => rows.push((b.y, b.y + b.height, vec![item])),
            Some(i) => {
                rows[i].0 = rows[i].0.min(b.y);
                rows[i].1 = rows[i].1.max(b.y + b.height);
                rows[i].2.push(item);
            }
        }
    }
    rows.sort_by_key(|r| r.0);
    rows.into_iter()
        .flat_map(|mut r| {
            r.2.sort_by_key(|(b, _)| b.x);
            r.2
        })
        .collect()
}

/// Background-colored pixels become transparent (with a soft edge for anti-aliasing).
fn clear_background(piece: &mut PixelBuffer, bg: Rgb, tolerance: i32) {
    let (inner, outer) = (f64::from(tolerance) * 0.6, f64::from(tolerance) * 1.4 + 1.0);
    for p in piece.data_mut().chunks_exact_mut(4) {
        let (dr, dg, db) = (f64::from(p[0]) - bg.0, f64::from(p[1]) - bg.1, f64::from(p[2]) - bg.2);
        let distance = ((dr * dr + dg * dg + db * db) / 3.0).sqrt();
        if distance >= outer {
            continue;
        }
        let keep = if distance <= inner { 0.0 } else { (distance - inner) / (outer - inner) };
        p[3] = (f64::from(p[3]) * keep).round().clamp(0.0, 255.0) as u8;
    }
}

/// Splits `sheet`. `selection_mask` (0..255 per pixel, canvas sized) makes its separate areas the pieces. `Err` when
/// cancelled.
pub fn split(
    sheet: &PixelBuffer,
    settings: &SplitSettings,
    selection_mask: Option<&[u8]>,
    cancel: &AtomicBool,
) -> Result<Vec<SheetPiece>, String> {
    let (w, h) = (sheet.width() as usize, sheet.height() as usize);
    let stop = || if cancel.load(Ordering::Relaxed) { Err("Cancelled.".to_owned()) } else { Ok(()) };
    let (is_piece, background) = match selection_mask {
        Some(mask) => {
            let is_piece: Vec<bool> = (0..w * h).map(|p| mask[p] >= 64 && sheet.data()[p * 4 + 3] >= 24).collect();
            let background = if settings.remove_background && !transparent_border(sheet) {
                detect_border_colors(sheet).into_iter().next()
            } else {
                None
            };
            (is_piece, background)
        }
        None => foreground_mask(sheet, settings.tolerance),
    };
    stop()?;
    // Every part grows by half the gap, so parts closer than the gap touch and become one piece.
    let joined =
        if settings.gap > 0 { dilate(&is_piece, w, h, ((settings.gap + 1) / 2) as usize) } else { is_piece.clone() };
    stop()?;
    let mut labels = vec![0u32; w * h];
    let boxes: Vec<(PixelRect, u32)> = find_boxes(&joined, &is_piece, w, h, &mut labels)
        .into_iter()
        .filter(|(b, _)| b.width >= settings.min_size && b.height >= settings.min_size)
        .collect();
    let mut pieces = Vec::with_capacity(boxes.len());
    for (bounds, label) in reading_order(boxes) {
        stop()?;
        let area = bounds.inflate(settings.padding);
        let mut pixels = sheet.crop(area); // outside the sheet becomes transparent
        if let Some(mask) = selection_mask {
            keep_selected_area(&mut pixels, area, mask, &labels, label, w, h);
        }
        if let (true, Some(bg)) = (settings.remove_background, background) {
            clear_background(&mut pixels, bg, settings.tolerance);
        }
        pieces.push(SheetPiece { bounds: area, pixels });
    }
    Ok(pieces)
}

/// Keeps only this piece's selected pixels (soft selection edges stay soft).
fn keep_selected_area(
    pixels: &mut PixelBuffer,
    area: PixelRect,
    mask: &[u8],
    labels: &[u32],
    label: u32,
    w: usize,
    h: usize,
) {
    for y in 0..area.height {
        let sy = area.y + y;
        for x in 0..area.width {
            let sx = area.x + x;
            let inside = sx >= 0
                && sy >= 0
                && (sx as usize) < w
                && (sy as usize) < h
                && labels[sy as usize * w + sx as usize] == label;
            let coverage = if inside { f64::from(mask[sy as usize * w + sx as usize]) / 255.0 } else { 0.0 };
            let i = pixels.index_of(x as u32, y as u32) + 3;
            pixels.data_mut()[i] = (f64::from(pixels.data()[i]) * coverage).round().clamp(0.0, 255.0) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    fn sheet() -> PixelBuffer {
        // A white sheet with three red squares: two on the top row, one below.
        let mut s = PixelBuffer::filled(120, 100, ColorRgba::WHITE);
        for (x0, y0) in [(10, 10), (60, 12), (30, 60)] {
            for y in y0..y0 + 20 {
                for x in x0..x0 + 20 {
                    s.set_pixel(x, y, ColorRgba::rgb(220, 20, 20));
                }
            }
        }
        s
    }

    #[test]
    fn items_on_a_plain_background_are_found_in_reading_order() {
        let pieces = split(&sheet(), &SplitSettings::default(), None, &AtomicBool::new(false)).expect("split");
        let positions: Vec<(i32, i32)> = pieces.iter().map(|p| (p.bounds.x, p.bounds.y)).collect();
        assert_eq!(positions, vec![(10, 10), (60, 12), (30, 60)]);
        assert!(pieces.iter().all(|p| p.bounds.width == 20 && p.bounds.height == 20));
    }

    #[test]
    fn the_sheet_background_becomes_transparent_inside_a_piece_when_asked() {
        let settings = SplitSettings { padding: 4, ..SplitSettings::default() };
        let pieces = split(&sheet(), &settings, None, &AtomicBool::new(false)).expect("split");
        assert_eq!(pieces[0].pixels.pixel(0, 0).a, 0, "the white padding is see-through");
        assert_eq!(pieces[0].pixels.pixel(10, 10).a, 255);
        let keep = SplitSettings { remove_background: false, ..settings };
        assert_eq!(split(&sheet(), &keep, None, &AtomicBool::new(false)).expect("split")[0].pixels.pixel(0, 0).a, 255);
    }

    #[test]
    fn close_parts_join_into_one_piece_and_specks_are_ignored() {
        let mut s = sheet();
        s.set_pixel(100, 90, ColorRgba::rgb(0, 0, 0)); // a speck
        for y in 10..30 {
            for x in 33..40 {
                s.set_pixel(x, y, ColorRgba::rgb(220, 20, 20)); // closer than the gap to the first square
            }
        }
        let pieces = split(&s, &SplitSettings::default(), None, &AtomicBool::new(false)).expect("split");
        assert_eq!(pieces.len(), 3);
        assert_eq!(pieces[0].bounds.width, 30);
    }

    #[test]
    fn a_selection_decides_the_pieces() {
        let s = sheet();
        let mut mask = vec![0u8; 120 * 100];
        for y in 60..80 {
            for x in 30..50 {
                mask[y * 120 + x] = 255;
            }
        }
        let pieces = split(&s, &SplitSettings::default(), Some(&mask), &AtomicBool::new(false)).expect("split");
        assert_eq!(pieces.len(), 1);
        assert_eq!((pieces[0].bounds.x, pieces[0].bounds.y), (30, 60));
    }

    #[test]
    fn a_cancelled_split_reports_it() {
        assert!(split(&sheet(), &SplitSettings::default(), None, &AtomicBool::new(true)).is_err());
    }
}
