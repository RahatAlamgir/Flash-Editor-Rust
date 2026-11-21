//! Small helpers between picture-sized masks and selections.

use fe_core::imaging::PixelRect;
use fe_core::selection::Selection;

/// A selection from a canvas-sized mask (0 = not selected). `None` when nothing is set.
pub fn selection_from_canvas_mask(mask: &[u8], width: u32, height: u32) -> Option<Selection> {
    let area = fe_ai::inpainting::bounds_of(mask, width as usize, height as usize);
    if area.is_empty() {
        return None;
    }
    let mut piece = Vec::with_capacity(area.width as usize * area.height as usize);
    for y in area.y..area.y + area.height {
        let from = y as usize * width as usize + area.x as usize;
        piece.extend_from_slice(&mask[from..from + area.width as usize]);
    }
    Selection::from_mask(piece, area, width, height)
}

/// The selection as a canvas-sized mask.
pub fn canvas_mask(selection: &Selection, width: u32, height: u32) -> Vec<u8> {
    selection.mask_over(PixelRect::new(0, 0, width as i32, height as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mask_becomes_a_selection_around_what_is_set() {
        let mut mask = vec![0u8; 10 * 8];
        assert!(selection_from_canvas_mask(&mask, 10, 8).is_none());
        for y in 2..5 {
            for x in 3..7 {
                mask[y * 10 + x] = 255;
            }
        }
        let selection = selection_from_canvas_mask(&mask, 10, 8).expect("a selection");
        assert_eq!(selection.bounds(), PixelRect::new(3, 2, 4, 3));
        assert_eq!(canvas_mask(&selection, 10, 8), mask);
    }
}
