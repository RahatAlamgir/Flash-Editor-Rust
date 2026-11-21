//! Opens `.flash` projects (version 1 and 2), including files saved by the C# version.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};
use std::path::Path;
use std::sync::Arc;

use rayon::prelude::*;

use super::manifest::{CURRENT_VERSION, FORMAT_NAME, LayerEntry, Manifest, PageEntry};
use super::png_stream::{decode_bands, decode_to_tiles};
use super::{MANIFEST_ENTRY, ProjectContent};
use crate::document::{AdjustmentStyle, Document, Layer, LayerMask, Page};
use crate::imaging::{BlendMode, PixelRect};
use crate::io::IoError;
use crate::io::decode::check_size;
use crate::objects::StoredObject;

fn not_a_project() -> IoError {
    IoError::CannotOpen("This is not a Flash Editor project.".into())
}

fn damaged(why: impl std::fmt::Display) -> IoError {
    IoError::CannotOpen(format!("The project file is damaged and could not be read ({why})."))
}

/// Opens a project file.
pub fn open(path: impl AsRef<Path>) -> Result<ProjectContent, IoError> {
    let file = std::fs::File::open(path)?;
    open_from(std::io::BufReader::new(file))
}

/// Opens a project from any readable source (used by tests and autosave recovery).
pub fn open_from(reader: impl Read + Seek) -> Result<ProjectContent, IoError> {
    let mut zip = zip::ZipArchive::new(reader).map_err(|_| not_a_project())?;
    let manifest: Manifest = {
        let mut entry = zip.by_name(MANIFEST_ENTRY).map_err(|_| not_a_project())?;
        let mut text = Vec::new();
        entry.read_to_end(&mut text)?;
        serde_json::from_slice(&text).map_err(damaged)?
    };
    if manifest.format != FORMAT_NAME {
        return Err(not_a_project());
    }
    if manifest.version > CURRENT_VERSION {
        return Err(IoError::CannotOpen(
            "This project was saved by a newer version of Flash Editor. Update the program to open it.".into(),
        ));
    }
    let active_page = manifest.active_page;
    let entries = if manifest.pages.is_empty() {
        vec![PageEntry {
            name: "Page 1".into(),
            width: manifest.width,
            height: manifest.height,
            active_layer: manifest.active_layer,
            layers: manifest.layers,
            ..PageEntry::default()
        }]
    } else {
        manifest.pages
    };

    // Read every picture the manifest names (zip reading is one at a time; decoding is parallel below).
    let in_zip: HashSet<String> = zip.file_names().filter_map(Result::ok).map(|n| n.into_owned()).collect();
    let is_picture = |s: &str| s.starts_with("images/") && in_zip.contains(s);
    let mut wanted: Vec<String> = Vec::new();
    for page in &entries {
        for layer in &page.layers {
            wanted.extend(
                [&layer.pixels, &layer.extended, &layer.restore, &layer.mask_picture].into_iter().flatten().cloned(),
            );
            if let Some(object) = &layer.object {
                StoredObject::picture_names(object, &is_picture, &mut wanted);
            }
        }
    }
    let mut bytes: HashMap<String, Arc<Vec<u8>>> = HashMap::new();
    for name in wanted {
        if bytes.contains_key(&name) {
            continue;
        }
        let mut entry = zip.by_name(&name).map_err(|_| damaged(format!("{name} is missing")))?;
        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data)?;
        bytes.insert(name, Arc::new(data));
    }

    let mut pages = Vec::with_capacity(entries.len());
    for entry in &entries {
        pages.push(build_page(entry, &bytes, &is_picture)?);
    }
    if pages.is_empty() {
        return Err(damaged("no pages"));
    }
    let active_page = active_page.min(pages.len() - 1);
    Ok(ProjectContent { pages, active_page })
}

fn build_page(
    entry: &PageEntry,
    bytes: &HashMap<String, Arc<Vec<u8>>>,
    is_picture: &(dyn Fn(&str) -> bool + Sync),
) -> Result<Page, IoError> {
    if entry.layers.is_empty() {
        return Err(damaged("a page has no layers"));
    }
    check_size(entry.width, entry.height)?;
    let layers: Vec<Layer> = entry
        .layers
        .par_iter()
        .map(|l| build_layer(l, entry.width, entry.height, bytes, is_picture))
        .collect::<Result<_, _>>()?;
    let mut doc = Document::new(entry.width, entry.height);
    let active = layers.get(entry.active_layer.min(layers.len() - 1)).map(Layer::id);
    doc.set_layers(layers, active);
    let mut page = Page::new(entry.name.clone(), doc);
    page.vertical_guides = entry.vertical_guides.clone();
    page.horizontal_guides = entry.horizontal_guides.clone();
    Ok(page)
}

fn build_layer(
    entry: &LayerEntry,
    width: u32,
    height: u32,
    bytes: &HashMap<String, Arc<Vec<u8>>>,
    is_picture: &(dyn Fn(&str) -> bool + Sync),
) -> Result<Layer, IoError> {
    let get = |name: &String| bytes.get(name).ok_or_else(|| damaged(format!("{name} is missing")));
    let mut layer = Layer::new(entry.name.clone());
    // Pixels: the "extended" picture when there is one (it holds pixels outside the canvas too).
    let pixels = match (&entry.extended, &entry.pixels) {
        (Some(ext), _) => decode_to_tiles(get(ext)?, entry.extended_x, entry.extended_y)?.0,
        (None, Some(px)) => {
            let (map, w, h) = decode_to_tiles(get(px)?, 0, 0)?;
            if (w, h) != (width, height) {
                return Err(damaged("a layer does not match the size of the picture"));
            }
            map
        }
        (None, None) => return Err(damaged("a layer has no picture")),
    };
    let object = match &entry.object {
        Some(json) => {
            let mut names = Vec::new();
            StoredObject::picture_names(json, is_picture, &mut names);
            let images = names.into_iter().filter_map(|n| bytes.get(&n).map(|b| (n, b.clone()))).collect();
            Some(StoredObject { json: json.clone(), images })
        }
        None => None,
    };
    layer.set_pixels_with_object(pixels, object);
    layer.set_opacity(entry.opacity);
    layer.visible = entry.visible;
    layer.locked = entry.locked;
    layer.blend = BlendMode::from_key(&entry.blend_mode).unwrap_or_default();
    layer.adjustment = entry.adjustment.as_ref().map(AdjustmentStyle::from);
    if let Some(restore) = &entry.restore {
        layer.restore = Some(decode_to_tiles(get(restore)?, 0, 0)?.0);
    }
    if let Some(mask_name) = &entry.mask_picture {
        let mut mask = LayerMask::new(width, height, 255);
        let (w, h) = decode_bands(get(mask_name)?, |y, rows, w| {
            let values: Vec<u8> = rows.as_chunks::<4>().0.iter().map(|p| p[1]).collect();
            let count = (values.len() / w.max(1) as usize) as i32;
            mask.paste_region(PixelRect::new(0, y as i32, w as i32, count), &values);
        })?;
        if (w, h) == (width, height) {
            let _ = layer.set_mask(Some(mask), width, height);
            layer.mask_enabled = entry.mask_enabled;
        }
    }
    Ok(layer)
}
