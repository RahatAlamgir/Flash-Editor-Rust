//! A document (one page's picture): canvas size and the stack of layers, bottom to top.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{Layer, LayerId};
use crate::imaging::{ColorRgba, PixelBuffer, PixelRect};
use crate::selection::Selection;

/// A picture made of layers. `layers[0]` is the bottom layer.
#[derive(Clone, Debug)]
pub struct Document {
    /// Identifies this document for caches (copies made for background work keep it).
    uid: u64,
    width: u32,
    height: u32,
    layers: Vec<Layer>,
    active: Option<LayerId>,
    /// Layers selected in the Layers panel (Ctrl / Shift click). Always holds the active layer.
    selected: Vec<LayerId>,
    layer_counter: u32,
    /// The selected area of the canvas (marching ants), if any. Shared: copies for undo cost nothing.
    area: Option<Arc<Selection>>,
}

impl Document {
    /// An empty document (no layers). Panics if a side is 0.
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0 && height > 0, "canvas size must be positive");
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            uid: NEXT.fetch_add(1, Ordering::Relaxed),
            width,
            height,
            layers: Vec::new(),
            active: None,
            selected: Vec::new(),
            layer_counter: 0,
            area: None,
        }
    }

    /// A document with one "Background" layer filled with `background` (transparent = empty layer).
    pub fn blank(width: u32, height: u32, background: ColorRgba) -> Self {
        let mut doc = Self::new(width, height);
        let mut layer = Layer::new("Background");
        if background.a > 0 {
            layer.pixels_mut().fill_rect(doc.bounds(), background);
        }
        doc.push_layer(layer);
        doc
    }

    /// A document showing one picture.
    pub fn from_image(image: &PixelBuffer, layer_name: &str) -> Self {
        let mut doc = Self::new(image.width(), image.height());
        doc.push_layer(Layer::from_buffer(layer_name, image));
        doc
    }

    /// A number that identifies this document (and snapshots of it) for caches such as the canvas textures.
    pub fn uid(&self) -> u64 {
        self.uid
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

    /// Layers bottom to top.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }

    pub fn layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id() == id)
    }

    pub fn layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id() == id)
    }

    pub fn index_of(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|l| l.id() == id)
    }

    pub fn active_id(&self) -> Option<LayerId> {
        self.active
    }

    pub fn active_layer(&self) -> Option<&Layer> {
        self.active.and_then(|id| self.layer(id))
    }

    pub fn active_layer_mut(&mut self) -> Option<&mut Layer> {
        self.active.and_then(|id| self.layer_mut(id))
    }

    /// Index of the active layer, if any.
    pub fn active_index(&self) -> Option<usize> {
        self.active.and_then(|id| self.index_of(id))
    }

    /// Makes a layer active (and the only selected one); ids that are not in the document are ignored.
    pub fn set_active(&mut self, id: Option<LayerId>) -> bool {
        if id.is_some_and(|id| self.index_of(id).is_none()) {
            return false;
        }
        self.active = id;
        self.selected = id.into_iter().collect();
        true
    }

    /// The selected layers, bottom to top (always includes the active layer).
    pub fn selected_ids(&self) -> Vec<LayerId> {
        self.layers.iter().map(Layer::id).filter(|id| self.is_selected(*id)).collect()
    }

    pub fn is_selected(&self, id: LayerId) -> bool {
        self.active == Some(id) || self.selected.contains(&id)
    }

    /// Selects these layers; `active` becomes the active one (it is added when missing). Unknown ids are dropped.
    pub fn set_selection(&mut self, ids: &[LayerId], active: Option<LayerId>) {
        let known: Vec<LayerId> = ids.iter().copied().filter(|id| self.index_of(*id).is_some()).collect();
        let active = active.filter(|id| self.index_of(*id).is_some()).or(known.last().copied());
        self.selected = known;
        if let Some(a) = active
            && !self.selected.contains(&a)
        {
            self.selected.push(a);
        }
        self.active = active;
    }

    /// "Layer 1", "Layer 2"... (or another prefix).
    pub fn next_layer_name(&mut self, prefix: &str) -> String {
        self.layer_counter += 1;
        format!("{prefix} {}", self.layer_counter)
    }

    /// Adds a layer on top and makes it active.
    pub fn push_layer(&mut self, layer: Layer) -> LayerId {
        let id = layer.id();
        self.layers.push(layer);
        self.set_active(Some(id));
        id
    }

    /// Inserts a layer at `index` (0 = bottom) and makes it active.
    pub fn insert_layer(&mut self, index: usize, layer: Layer) -> LayerId {
        let id = layer.id();
        self.layers.insert(index.min(self.layers.len()), layer);
        self.set_active(Some(id));
        id
    }

    /// Removes a layer. The active layer moves to the one below (or the new bottom).
    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer> {
        let index = self.index_of(id)?;
        let layer = self.layers.remove(index);
        self.selected.retain(|s| *s != id);
        if self.active == Some(id) {
            let next = self.layers.get(index.saturating_sub(1)).or(self.layers.last()).map(Layer::id);
            self.set_active(next);
        }
        Some(layer)
    }

    /// Replaces the whole stack (bottom to top). Keeps `active` when it is in the new stack, else the top layer.
    pub fn set_layers(&mut self, layers: Vec<Layer>, active: Option<LayerId>) {
        self.layers = layers;
        let valid = active.filter(|id| self.index_of(*id).is_some());
        self.set_active(valid.or(self.layers.last().map(Layer::id)));
    }

    /// Changes the canvas size only (layers keep their pixels where they are). Drops the selected area.
    pub fn set_canvas_size(&mut self, width: u32, height: u32) {
        assert!(width > 0 && height > 0, "canvas size must be positive");
        self.width = width;
        self.height = height;
        self.area = None;
    }

    /// The selected area of the canvas (`None` = nothing selected, so everything can be edited).
    pub fn selected_area(&self) -> Option<&Selection> {
        self.area.as_deref()
    }

    /// The selected area as a shared handle (to notice whether it changed: compare with `Arc::ptr_eq`).
    pub fn selected_area_arc(&self) -> Option<&Arc<Selection>> {
        self.area.as_ref()
    }

    /// Sets or clears the selected area. A selection made for another canvas size is ignored.
    pub fn set_selected_area(&mut self, area: Option<Selection>) {
        self.area = area.filter(|s| s.canvas_width() == self.width && s.canvas_height() == self.height).map(Arc::new);
    }

    /// Sets the selected area from a shared handle.
    pub fn set_selected_area_arc(&mut self, area: Option<Arc<Selection>>) {
        self.area = area.filter(|s| s.canvas_width() == self.width && s.canvas_height() == self.height);
    }

    /// Memory held by all layer tiles and masks.
    pub fn memory_bytes(&self) -> usize {
        self.layers.iter().map(|l| l.pixels().memory_bytes() + l.mask().map_or(0, |m| m.memory_bytes())).sum()
    }
}
