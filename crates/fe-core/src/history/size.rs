//! How much memory one undo step really holds: the tiles of the "before" copy that the "after" copy does not
//! share (and the other way round). Shared tiles cost nothing extra.

use std::collections::HashSet;
use std::sync::Arc;

use crate::document::Document;

/// Bytes held only by one side of the step.
pub fn step_bytes(before: &Document, after: &Document) -> usize {
    docs_bytes(std::iter::once(before), std::iter::once(after))
}

/// Bytes held only by one side, for steps that keep several documents (page list changes).
pub fn docs_bytes<'a>(before: impl Iterator<Item = &'a Document>, after: impl Iterator<Item = &'a Document>) -> usize {
    let mut a = HashSet::new();
    before.for_each(|d| a.extend(tile_pointers(d)));
    let mut b = HashSet::new();
    after.for_each(|d| b.extend(tile_pointers(d)));
    let only_a = a.difference(&b).count();
    let only_b = b.difference(&a).count();
    (only_a + only_b) * crate::imaging::TileMap::TILE_BYTES
}

fn tile_pointers(doc: &Document) -> HashSet<usize> {
    let mut set = HashSet::new();
    for layer in doc.layers() {
        let pixels = layer.pixels();
        for key in pixels.keys() {
            if let Some(tile) = pixels.tile_arc(key) {
                set.insert(Arc::as_ptr(tile) as usize);
            }
        }
    }
    set
}
