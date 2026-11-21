//! Frames (C# `PictureFill`, `CollageLayouts`): a picture shown inside a shape, and ready-made arrangements of
//! frames on a grid (Photo Grid).

use std::sync::Arc;

use super::foreign::new_picture_name;
use super::{PictureFill, StoredObject};

/// One frame of a collage: where it sits and how many grid cells it covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollageCell {
    pub column: usize,
    pub row: usize,
    pub column_span: usize,
    pub row_span: usize,
}

const fn cell(column: usize, row: usize, column_span: usize, row_span: usize) -> CollageCell {
    CollageCell { column, row, column_span, row_span }
}

/// A ready-made arrangement of frames on a grid (like the layouts of a collage maker).
pub struct CollageLayout {
    pub name: &'static str,
    pub columns: usize,
    pub rows: usize,
    pub cells: &'static [CollageCell],
}

impl CollageLayout {
    /// The frames as rectangles (x, y, width, height) on a canvas: an even border (`padding`) and the same gap
    /// between the frames.
    pub fn place(&self, canvas_w: f64, canvas_h: f64, padding: f64, gap: f64) -> Vec<(f64, f64, f64, f64)> {
        let cw = ((canvas_w - 2.0 * padding - gap * (self.columns - 1) as f64) / self.columns as f64).max(1.0);
        let ch = ((canvas_h - 2.0 * padding - gap * (self.rows - 1) as f64) / self.rows as f64).max(1.0);
        self.cells
            .iter()
            .map(|c| {
                (
                    padding + c.column as f64 * (cw + gap),
                    padding + c.row as f64 * (ch + gap),
                    c.column_span as f64 * cw + (c.column_span - 1) as f64 * gap,
                    c.row_span as f64 * ch + (c.row_span - 1) as f64 * gap,
                )
            })
            .collect()
    }
}

/// Every cell of a plain `columns` wide grid (row by row).
const fn grid_cells<const N: usize>(columns: usize) -> [CollageCell; N] {
    let mut cells = [cell(0, 0, 1, 1); N];
    let mut i = 0;
    while i < N {
        cells[i] = cell(i % columns, i / columns, 1, 1);
        i += 1;
    }
    cells
}

pub const COLLAGE_LAYOUTS: [CollageLayout; 10] = [
    CollageLayout { name: "Two side by side", columns: 2, rows: 1, cells: &grid_cells::<2>(2) },
    CollageLayout { name: "Two stacked", columns: 1, rows: 2, cells: &grid_cells::<2>(1) },
    CollageLayout { name: "Three columns", columns: 3, rows: 1, cells: &grid_cells::<3>(3) },
    CollageLayout { name: "Three rows", columns: 1, rows: 3, cells: &grid_cells::<3>(1) },
    CollageLayout { name: "2 x 2 grid", columns: 2, rows: 2, cells: &grid_cells::<4>(2) },
    CollageLayout { name: "3 x 3 grid", columns: 3, rows: 3, cells: &grid_cells::<9>(3) },
    CollageLayout {
        name: "One large, two small",
        columns: 2,
        rows: 2,
        cells: &[cell(0, 0, 1, 2), cell(1, 0, 1, 1), cell(1, 1, 1, 1)],
    },
    CollageLayout {
        name: "Large on top, three small",
        columns: 3,
        rows: 3,
        cells: &[cell(0, 0, 3, 2), cell(0, 2, 1, 1), cell(1, 2, 1, 1), cell(2, 2, 1, 1)],
    },
    CollageLayout {
        name: "Large on the left, three small",
        columns: 3,
        rows: 3,
        cells: &[cell(0, 0, 2, 3), cell(2, 0, 1, 1), cell(2, 1, 1, 1), cell(2, 2, 1, 1)],
    },
    CollageLayout {
        name: "Wide band and four",
        columns: 2,
        rows: 3,
        cells: &[cell(0, 0, 2, 1), cell(0, 1, 1, 1), cell(1, 1, 1, 1), cell(0, 2, 1, 1), cell(1, 2, 1, 1)],
    },
];

impl StoredObject {
    /// The picture inside a shape (a frame) with its zoom and position.
    pub fn picture_fill(&self) -> Option<PictureFill> {
        self.shape()?.picture
    }

    /// Puts a picture (PNG bytes) into a shape, or takes it out with `None`. Zoom and position start centered.
    pub fn set_picture_fill(&mut self, picture: Option<(Arc<Vec<u8>>, &str)>) {
        let Some(mut shape) = self.shape() else { return };
        match picture {
            Some((bytes, name)) => {
                let file = new_picture_name();
                self.images.insert(file.clone(), bytes);
                shape.picture =
                    Some(PictureFill { image: file, name: name.to_owned(), zoom: 1.0, pan_x: 0.0, pan_y: 0.0 });
            }
            None => shape.picture = None,
        }
        self.set_shape(&shape);
    }

    /// Changes the zoom / position of the picture inside a frame.
    pub fn set_picture_view(&mut self, zoom: f64, pan_x: f64, pan_y: f64) {
        let Some(mut shape) = self.shape() else { return };
        if let Some(p) = &mut shape.picture {
            p.zoom = zoom.clamp(0.25, 5.0);
            p.pan_x = pan_x.clamp(-1.0, 1.0);
            p.pan_y = pan_y.clamp(-1.0, 1.0);
            self.set_shape(&shape);
        }
    }
}
