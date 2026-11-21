//! Reordering layers by dragging in the Layers panel. The panel lists layers top first; gap 0 is above the
//! top row. Same rules as the C# `LayerOrder`.

/// The vertical span of one row that is on screen. `index` counts from the top of the whole list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowSpan {
    pub index: usize,
    pub top: f64,
    pub bottom: f64,
}

/// The gap (0 = above the first row) nearest to pointer height `y`.
pub fn gap_at(visible_rows: &[RowSpan], y: f64, row_count: usize) -> usize {
    let mut rows = visible_rows.to_vec();
    rows.sort_by_key(|r| r.index);
    match rows.iter().find(|r| y < r.top + (r.bottom - r.top) / 2.0) {
        Some(row) => row.index,
        None => rows.last().map_or(0, |r| row_count.min(r.index + 1)),
    }
}

/// The new bottom-to-top order after dropping `moved` items into `gap`, or `None` when nothing changes.
/// Moved items keep their order among themselves.
pub fn move_to_gap<T: PartialEq + Clone>(bottom_to_top: &[T], moved: &[T], gap: usize) -> Option<Vec<T>> {
    let top_down: Vec<T> = bottom_to_top.iter().rev().cloned().collect();
    let gap = gap.min(top_down.len());
    let insert_at = top_down[..gap].iter().filter(|item| !moved.contains(item)).count();
    let dragged: Vec<T> = top_down.iter().filter(|item| moved.contains(item)).cloned().collect();
    if dragged.is_empty() {
        return None;
    }
    let mut result: Vec<T> = top_down.iter().filter(|item| !moved.contains(item)).cloned().collect();
    result.splice(insert_at..insert_at, dragged);
    if result == top_down {
        return None;
    }
    result.reverse();
    Some(result)
}
