//! Saves `.flash` projects in the format the C# version reads (version 2): every layer as a canvas-sized PNG,
//! an "extended" PNG when pixels lie outside the canvas, masks as grey pictures, objects kept as read, and a
//! small preview for the Project Manager. Pictures are encoded in parallel, streamed from the tiles.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use rayon::prelude::*;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use super::manifest::{AdjustmentEntry, AnimationEntry, LayerEntry, Manifest, PageEntry};
use super::png_stream::{encode_rows, encode_tiles, recompress_png, use_webp};
use super::{IMAGE_FOLDER, MANIFEST_ENTRY, PREVIEW_ENTRY, PREVIEW_SIZE};
use crate::document::{Layer, LayerMask, Page};
use crate::imaging::{PixelRect, TileMap};
use crate::io::IoError;
use crate::io::encode::encode_png;
use crate::paint::thumbnails::page_thumbnail;

/// A picture to put into the file.
enum Picture {
    Tiles(TileMap, PixelRect),
    Mask(LayerMask),
    Bytes(Arc<Vec<u8>>),
    /// An object picture stored again as lossless WebP (smaller).
    Packed(Vec<u8>),
}

impl Picture {
    /// `webp` for pictures stored losslessly as WebP, `png` for the very large ones and for object pictures.
    fn extension(&self) -> &'static str {
        match self {
            Self::Tiles(_, rect) if use_webp(rect.width as u32, rect.height as u32) => "webp",
            Self::Mask(mask) if use_webp(mask.width(), mask.height()) => "webp",
            _ => "png",
        }
    }
}

/// Gives every picture a name (`images/N.png`); the same object picture is stored once.
#[derive(Default)]
struct Pictures {
    list: Vec<(String, Picture)>,
    shared: HashMap<usize, String>,
}

impl Pictures {
    fn add(&mut self, picture: Picture) -> String {
        if let Picture::Bytes(b) = &picture
            && let Some(name) = self.shared.get(&(Arc::as_ptr(b) as usize))
        {
            return name.clone();
        }
        let picture = match picture {
            Picture::Bytes(b) => match recompress_png(&b) {
                Some(packed) => {
                    let name = format!("{IMAGE_FOLDER}{}.webp", self.list.len());
                    self.shared.insert(Arc::as_ptr(&b) as usize, name.clone());
                    self.list.push((name.clone(), Picture::Packed(packed)));
                    return name;
                }
                None => Picture::Bytes(b),
            },
            other => other,
        };
        let extension = picture.extension();
        let name = format!("{IMAGE_FOLDER}{}.{extension}", self.list.len());
        if let Picture::Bytes(b) = &picture {
            self.shared.insert(Arc::as_ptr(b) as usize, name.clone());
        }
        self.list.push((name.clone(), picture));
        name
    }
}

/// Saves pages to `path`. Writes a temporary file first, so a failed save never damages the old file.
pub fn save(pages: &[Page], active_page: usize, path: impl AsRef<Path>) -> Result<(), IoError> {
    save_as(pages, active_page, false, path)
}

/// Like `save`, for a project that is (or is not) a presentation.
pub fn save_as(pages: &[Page], active_page: usize, presentation: bool, path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    let bytes = to_bytes_as(pages, active_page, presentation)?;
    let mut temp = path.as_os_str().to_owned();
    temp.push(".saving");
    std::fs::write(&temp, bytes)?;
    std::fs::rename(&temp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&temp);
    })?;
    Ok(())
}

/// The whole project file as bytes.
pub fn to_bytes(pages: &[Page], active_page: usize) -> Result<Vec<u8>, IoError> {
    to_bytes_as(pages, active_page, false)
}

/// Like `to_bytes`, for a project that is (or is not) a presentation.
pub fn to_bytes_as(pages: &[Page], active_page: usize, presentation: bool) -> Result<Vec<u8>, IoError> {
    if pages.is_empty() {
        return Err(IoError::CannotSave("A project needs at least one page.".into()));
    }
    let active_page = active_page.min(pages.len() - 1);
    let mut pictures = Pictures::default();
    let manifest = Manifest {
        pages: pages.iter().map(|p| page_entry(p, &mut pictures)).collect(),
        active_page,
        project_type: if presentation { "presentation".into() } else { String::new() },
        ..Manifest::default()
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|e| IoError::CannotSave(e.to_string()))?;
    let encoded: Vec<(String, Vec<u8>)> = pictures
        .list
        .par_iter()
        .map(|(name, picture)| encode(picture).map(|data| (name.clone(), data)))
        .collect::<Result<_, _>>()?;
    let preview = encode_png(&page_thumbnail(&pages[active_page].doc, PREVIEW_SIZE, PREVIEW_SIZE)).ok();

    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let deflate = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored); // PNG is compressed
    let zip_err = |e: zip::result::ZipError| IoError::CannotSave(e.to_string());
    zip.start_file(MANIFEST_ENTRY, deflate).map_err(zip_err)?;
    zip.write_all(&json)?;
    if let Some(preview) = preview {
        zip.start_file(PREVIEW_ENTRY, stored).map_err(zip_err)?;
        zip.write_all(&preview)?;
    }
    for (name, data) in encoded {
        zip.start_file(name, stored).map_err(zip_err)?;
        zip.write_all(&data)?;
    }
    Ok(zip.finish().map_err(zip_err)?.into_inner())
}

fn page_entry(page: &Page, pictures: &mut Pictures) -> PageEntry {
    let doc = &page.doc;
    PageEntry {
        name: page.name.clone(),
        width: doc.width(),
        height: doc.height(),
        active_layer: doc.active_index().unwrap_or(0),
        vertical_guides: page.vertical_guides.clone(),
        horizontal_guides: page.horizontal_guides.clone(),
        transition: (!page.transition.is_default()).then(|| (&page.transition).into()),
        animations: page
            .valid_animations()
            .iter()
            .filter_map(|a| {
                Some(AnimationEntry {
                    layer: doc.index_of(a.layer)?,
                    effect: a.effect.key().into(),
                    start: a.start.name().into(),
                    direction: a.direction.key().into(),
                    duration: a.duration,
                    delay: a.delay,
                })
            })
            .collect(),
        layers: doc.layers().iter().map(|l| layer_entry(l, pictures)).collect(),
    }
}

fn layer_entry(layer: &Layer, pictures: &mut Pictures) -> LayerEntry {
    let pixels = layer.pixels();
    let mut entry = LayerEntry {
        name: layer.name.clone(),
        opacity: layer.opacity(),
        visible: layer.visible,
        locked: layer.locked,
        blend_mode: layer.blend.key().into(),
        adjustment: layer.adjustment.as_ref().map(AdjustmentEntry::from),
        mask_enabled: layer.mask_enabled,
        ..LayerEntry::default()
    };
    // Only the part with something on it is saved (and where it goes); the empty rest of the canvas costs nothing.
    let visible = pixels.visible_bounds();
    entry.pixels = (!visible.is_empty()).then(|| pictures.add(Picture::Tiles(pixels.clone(), visible)));
    (entry.pixels_x, entry.pixels_y) = (visible.x, visible.y);
    if let Some(restore) = &layer.restore {
        let used = restore.visible_bounds();
        entry.restore = (!used.is_empty()).then(|| pictures.add(Picture::Tiles(restore.clone(), used)));
        (entry.restore_x, entry.restore_y) = (used.x, used.y);
    }
    if let Some(mask) = layer.mask() {
        entry.mask_picture = Some(pictures.add(Picture::Mask(mask.clone())));
    }
    if let Some(object) = layer.object() {
        let renamed: HashMap<String, String> =
            object.images.iter().map(|(old, data)| (old.clone(), pictures.add(Picture::Bytes(data.clone())))).collect();
        entry.object = Some(object.renamed(&|s| renamed.get(s).cloned()));
    }
    entry
}

fn encode(picture: &Picture) -> Result<Vec<u8>, IoError> {
    match picture {
        Picture::Tiles(map, rect) => encode_tiles(map, *rect),
        Picture::Mask(mask) => encode_rows(mask.width(), mask.height(), |y, count| {
            let rect = PixelRect::new(0, y as i32, mask.width() as i32, count as i32);
            mask.copy_region(rect).into_iter().flat_map(|v| [v, v, v, 255]).collect()
        }),
        Picture::Bytes(data) => Ok(data.as_ref().clone()),
        Picture::Packed(data) => Ok(data.clone()),
    }
}
