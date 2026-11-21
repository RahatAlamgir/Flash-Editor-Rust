//! Runs an effect on a layer's pixels, limited to the selection when there is one (C# `EffectApplier`).

use rayon::prelude::*;

use super::{Effect, Values};
use crate::imaging::PixelBuffer;
use crate::selection::Selection;

/// How many pixels around the selection the effect needs to read.
pub fn padding_for(effect: &Effect, values: &Values) -> i32 {
    (effect.padding)(values)
}

/// The effect on `source`; with a selection only the selected area changes (soft edges blend).
pub fn apply_to_selection(
    source: &PixelBuffer,
    effect: &Effect,
    values: &Values,
    selection: Option<&Selection>,
) -> PixelBuffer {
    let Some(sel) = selection else { return effect.apply(source, values) };
    let bounds = sel.bounds().intersect(&source.bounds());
    if bounds.is_empty() {
        return source.clone();
    }
    // Only the selected area plus the margin the effect reads is processed.
    let region = bounds.inflate(padding_for(effect, values)).intersect(&source.bounds());
    let processed = effect.apply(&source.crop(region), values);
    let mut out = source.clone();
    let stride = out.stride();
    out.data_mut().par_chunks_mut(stride).enumerate().for_each(|(y, row)| {
        let y = y as i32;
        if y < bounds.y || y >= bounds.y + bounds.height {
            return;
        }
        for x in bounds.x..bounds.x + bounds.width {
            let coverage = sel.coverage(x, y);
            if coverage <= 0.0 {
                continue;
            }
            let p = processed.pixel((x - region.x) as u32, (y - region.y) as u32);
            let d = &mut row[x as usize * 4..x as usize * 4 + 4];
            for (c, new) in d.iter_mut().zip([p.r, p.g, p.b, p.a]) {
                *c = (f64::from(*c) + (f64::from(new) - f64::from(*c)) * coverage).round() as u8;
            }
        }
    });
    out
}
