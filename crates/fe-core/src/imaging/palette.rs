//! Finds the main colors of a picture (for "Add colors from this picture"): plain k-means on a small sample, no AI
//! model (C# `PaletteExtractor`).

use super::{ColorRgba, PixelBuffer};

type Rgb = (f32, f32, f32);

fn distance(a: Rgb, b: Rgb) -> f64 {
    let (dr, dg, db) = (f64::from(a.0 - b.0), f64::from(a.1 - b.1), f64::from(a.2 - b.2));
    dr * dr + dg * dg + db * db
}

/// The `count` most typical colors, the most common first. Nearly transparent pixels are ignored. Colors that are
/// almost the same are merged, so fewer than `count` may come back for a plain picture.
pub fn extract(image: &PixelBuffer, count: usize) -> Vec<ColorRgba> {
    let count = count.clamp(1, 16);
    // A sample of at most ~4000 pixels, spread over the whole picture.
    let total = (image.width() as usize) * (image.height() as usize);
    let step = ((total as f64 / 4000.0).sqrt() as usize).max(1);
    let mut samples: Vec<Rgb> = Vec::new();
    for y in (0..image.height()).step_by(step) {
        for x in (0..image.width()).step_by(step) {
            let p = image.pixel(x, y);
            if p.a >= 128 {
                samples.push((f32::from(p.r), f32::from(p.g), f32::from(p.b)));
            }
        }
    }
    if samples.is_empty() {
        return Vec::new();
    }
    // Start from colors spread through the (sorted by brightness) samples, then let them settle.
    let mut ordered = samples.clone();
    ordered.sort_by(|a, b| (a.0 + a.1 + a.2).total_cmp(&(b.0 + b.1 + b.2)));
    let k = count.min(ordered.len());
    let mut centers: Vec<Rgb> =
        (0..k).map(|c| ordered[((c as f64 + 0.5) * ordered.len() as f64 / k as f64) as usize]).collect();
    let mut assignment = vec![0usize; samples.len()];
    for round in 0..12 {
        let mut changed = false;
        for (s, sample) in samples.iter().enumerate() {
            let best = (0..k)
                .min_by(|a, b| distance(*sample, centers[*a]).total_cmp(&distance(*sample, centers[*b])))
                .unwrap_or(0);
            if assignment[s] != best || round == 0 {
                assignment[s] = best;
                changed = true;
            }
        }
        let mut sums = vec![(0.0f64, 0.0f64, 0.0f64, 0usize); k];
        for (s, sample) in samples.iter().enumerate() {
            let sum = &mut sums[assignment[s]];
            sum.0 += f64::from(sample.0);
            sum.1 += f64::from(sample.1);
            sum.2 += f64::from(sample.2);
            sum.3 += 1;
        }
        for (c, sum) in sums.iter().enumerate() {
            if sum.3 > 0 {
                let n = sum.3 as f64;
                centers[c] = ((sum.0 / n) as f32, (sum.1 / n) as f32, (sum.2 / n) as f32);
            }
        }
        if !changed {
            break;
        }
    }
    let mut sizes = vec![0usize; k];
    for a in &assignment {
        sizes[*a] += 1;
    }
    let mut order: Vec<usize> = (0..k).filter(|c| sizes[*c] > 0).collect();
    order.sort_by_key(|c| std::cmp::Reverse(sizes[*c]));
    let mut result: Vec<ColorRgba> = Vec::new();
    for c in order {
        let color = ColorRgba::rgb(centers[c].0.round() as u8, centers[c].1.round() as u8, centers[c].2.round() as u8);
        // Skip near-duplicates.
        if result.iter().all(|r| distance((f32::from(r.r), f32::from(r.g), f32::from(r.b)), centers[c]) > 18.0 * 18.0) {
            result.push(color);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_main_colors_come_back_most_common_first() {
        let mut p = PixelBuffer::new(100, 100);
        p.fill(ColorRgba::rgb(255, 0, 0));
        for y in 0..100 {
            for x in 0..30 {
                p.set_pixel(x, y, ColorRgba::rgb(0, 0, 255));
            }
        }
        let colors = extract(&p, 3);
        assert_eq!(colors.len(), 2);
        assert!(colors[0].r > 200 && colors[0].b < 60);
        assert!(colors[1].b > 200 && colors[1].r < 60);
    }

    #[test]
    fn a_plain_picture_gives_one_color() {
        let mut p = PixelBuffer::new(20, 20);
        p.fill(ColorRgba::rgb(10, 200, 90));
        assert_eq!(extract(&p, 5), vec![ColorRgba::rgb(10, 200, 90)]);
    }

    #[test]
    fn see_through_pixels_are_ignored() {
        assert!(extract(&PixelBuffer::new(10, 10), 4).is_empty());
    }
}
