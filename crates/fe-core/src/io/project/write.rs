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

use super::manifest::{AdjustmentEntry, LayerEntry, Manifest, PageEntry};
use super::png_stream::{encode_rows, encode_tiles};
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
        let name = format!("{IMAGE_FOLDER}{}.png", self.list.len());
        if let Picture::Bytes(b) = &picture {
            self.shared.insert(Arc::as_ptr(b) as usize, name.clone());
        }
        self.list.push((name.clone(), picture));
        name
    }
}

/// Saves pages to `path`. Writes a temporary file first, so a failed save never damages the old file.
pub fn save(pages: &[Page], active_page: usize, path: impl AsRef<Path>) -> Result<(), IoError> {
    let path = path.as_ref();
    let bytes = to_bytes(pages, active_page)?;
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
    if pages.is_empty() {
        return Err(IoError::CannotSave("A project needs at least one page.".into()));
    }
    let active_page = active_page.min(pages.len() - 1);
    let mut pictures = Pictures::default();
    let manifest = Manifest {
        pages: pages.iter().map(|p| page_entry(p, &mut pictures)).collect(),
        active_page,
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
        layers: doc.layers().iter().map(|l| layer_entry(l, doc.bounds(), pictures)).collect(),
    }
}

fn layer_entry(layer: &Layer, canvas: PixelRect, pictures: &mut Pictures) -> LayerEntry {
    let pixels = layer.pixels();
    let mut entry = LayerEntry {
        name: layer.name.clone(),
        opacity: layer.opacity(),
        visible: layer.visible,
        locked: layer.locked,
        blend_mode: layer.blend.key().into(),
        adjustment: layer.adjustment.as_ref().map(AdjustmentEntry::from),
        pixels: Some(pictures.add(Picture::Tiles(pixels.clone(), canvas))),
        mask_enabled: layer.mask_enabled,
        ..LayerEntry::default()
    };
    let visible = pixels.visible_bounds();
    if !visible.is_empty() && visible.intersect(&canvas) != visible {
        let whole = canvas.union(&visible);
        entry.extended = Some(pictures.add(Picture::Tiles(pixels.clone(), whole)));
        entry.extended_x = whole.x;
        entry.extended_y = whole.y;
    }
    if let Some(restore) = &layer.restore {
        entry.restore = Some(pictures.add(Picture::Tiles(restore.clone(), canvas)));
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
    }
}
