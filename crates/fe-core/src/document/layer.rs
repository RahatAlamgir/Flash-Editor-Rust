//! One layer: its pixels, look (opacity, blend mode, visibility) and optional mask or adjustment.
//!
//! Unlike the C# version there is no canvas-sized buffer and no separate "extended" buffer: the sparse
//! [`TileMap`] holds pixels anywhere, also past the canvas edge, so nothing is ever cut off.

use std::sync::atomic::{AtomicU64, Ordering};

use super::{AdjustmentStyle, LayerMask};
use crate::imaging::{BlendMode, PixelBuffer, PixelRect, TileMap};
use crate::objects::StoredObject;

/// A unique id for a layer, stable for the whole run of the program.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(u64);

impl LayerId {
    /// A new id that no other layer has.
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// Why a mask could not be set.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("the mask must have the size of the canvas ({expected_w}x{expected_h}), not {got_w}x{got_h}")]
pub struct MaskSizeError {
    pub expected_w: u32,
    pub expected_h: u32,
    pub got_w: u32,
    pub got_h: u32,
}

/// A layer of a document.
#[derive(Clone, Debug)]
pub struct Layer {
    id: LayerId,
    pub name: String,
    opacity: f64,
    pub visible: bool,
    pub locked: bool,
    pub blend: BlendMode,
    /// Set for adjustment layers: they change everything below them and have no pixels of their own.
    pub adjustment: Option<AdjustmentStyle>,
    pixels: TileMap,
    mask: Option<LayerMask>,
    pub mask_enabled: bool,
    /// An editable object read from a project file (shown with its saved drawing; see [`StoredObject`]).
    /// Changing the pixels removes it: the layer becomes plain pixels.
    object: Option<StoredObject>,
    /// The original picture of a cut-out, for the Refine brush ("restore").
    pub restore: Option<TileMap>,
    version: u64,
}

impl Layer {
    /// An empty (transparent) layer.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::next(),
            name: name.into(),
            opacity: 1.0,
            visible: true,
            locked: false,
            blend: BlendMode::Normal,
            adjustment: None,
            pixels: TileMap::empty(),
            mask: None,
            mask_enabled: true,
            object: None,
            restore: None,
            version: 0,
        }
    }

    /// A layer holding a picture with its top-left at the canvas origin.
    pub fn from_buffer(name: impl Into<String>, picture: &PixelBuffer) -> Self {
        let mut layer = Self::new(name);
        layer.pixels = TileMap::from_buffer(picture, 0, 0);
        layer
    }

    /// An adjustment layer.
    pub fn adjustment(name: impl Into<String>, style: AdjustmentStyle) -> Self {
        let mut layer = Self::new(name);
        layer.adjustment = Some(style);
        layer
    }

    pub fn id(&self) -> LayerId {
        self.id
    }

    /// Goes up on every change of pixels or mask (for caches like thumbnails and GPU textures).
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn opacity(&self) -> f64 {
        self.opacity
    }

    /// Sets the opacity (clamped to 0..1).
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    pub fn is_adjustment(&self) -> bool {
        self.adjustment.is_some()
    }

    pub fn pixels(&self) -> &TileMap {
        &self.pixels
    }

    /// Pixels to change. Marks the layer as changed; an object layer becomes plain pixels.
    pub fn pixels_mut(&mut self) -> &mut TileMap {
        self.version += 1;
        self.object = None;
        &mut self.pixels
    }

    /// Replaces all pixels (cheap: tiles are shared, not copied); an object layer becomes plain pixels.
    pub fn replace_pixels(&mut self, pixels: TileMap) {
        self.version += 1;
        self.object = None;
        self.pixels = pixels;
    }

    /// Sets the pixels and the object they show together (when reading a project file).
    pub fn set_pixels_with_object(&mut self, pixels: TileMap, object: Option<StoredObject>) {
        self.version += 1;
        self.pixels = pixels;
        self.object = object;
    }

    /// The editable object of this layer, if it has one.
    pub fn object(&self) -> Option<&StoredObject> {
        self.object.as_ref()
    }

    pub fn mask(&self) -> Option<&LayerMask> {
        self.mask.as_ref()
    }

    /// The mask to change. Marks the layer as changed.
    pub fn mask_mut(&mut self) -> Option<&mut LayerMask> {
        self.version += 1;
        self.mask.as_mut()
    }

    /// The mask that is used for drawing (none while switched off).
    pub fn effective_mask(&self) -> Option<&LayerMask> {
        self.mask.as_ref().filter(|_| self.mask_enabled)
    }

    /// Adds, replaces or removes the mask. It must match the canvas size.
    pub fn set_mask(&mut self, mask: Option<LayerMask>, canvas_w: u32, canvas_h: u32) -> Result<(), MaskSizeError> {
        if let Some(m) = &mask
            && (m.width() != canvas_w || m.height() != canvas_h)
        {
            return Err(MaskSizeError {
                expected_w: canvas_w,
                expected_h: canvas_h,
                got_w: m.width(),
                got_h: m.height(),
            });
        }
        self.mask = mask;
        self.version += 1;
        Ok(())
    }

    /// The smallest rectangle around all visible pixels (may reach past the canvas).
    pub fn visible_bounds(&self) -> PixelRect {
        self.pixels.visible_bounds()
    }

    /// True when some pixels lie outside the canvas.
    pub fn has_hidden_pixels(&self, canvas: PixelRect) -> bool {
        let tiles = self.pixels.tile_bounds();
        tiles.intersect(&canvas) != tiles && {
            let visible = self.visible_bounds();
            visible.intersect(&canvas) != visible
        }
    }

    /// A copy with a new id and name. Pixels and mask are shared until one of the two layers changes them.
    pub fn duplicate(&self, name: impl Into<String>) -> Self {
        Self { id: LayerId::next(), name: name.into(), version: 0, ..self.clone() }
    }
}
