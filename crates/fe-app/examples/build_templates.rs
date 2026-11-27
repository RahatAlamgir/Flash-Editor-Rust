//! Builds the online template repository: for every template of `catalog.json` it re-saves the project as
//! `template.fe`, writes a picture of every page (`page-1.jpg`, `page-2.jpg`...) and the small `thumb.jpg`, and updates
//! the size / SHA-256 / page count in the catalog.
//!
//! Usage: `cargo run -p fe-app --release --example build_templates -- "D:\AiBuild\Flash Editor Templates"`

use std::path::{Path, PathBuf};

use fe_core::imaging::PixelBuffer;
use fe_core::io::{ImageFileFormat, encode::encode, project};
use fe_core::paint::thumbnails::page_thumbnail;
use serde_json::Value;
use sha2::{Digest, Sha256};

/// Longest side of a page picture in the preview window.
const PAGE_SIZE: u32 = 960;
/// Longest side of the card picture.
const THUMB_SIZE: u32 = 480;

fn jpeg(image: &PixelBuffer) -> Vec<u8> {
    encode(image, ImageFileFormat::Jpeg, 86, None).expect("jpeg")
}

fn find_source(folder: &Path) -> Option<PathBuf> {
    ["template.fe", "template.flash"].iter().map(|n| folder.join(n)).find(|p| p.is_file())
}

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).expect("usage: build_templates <templates repository folder>"));
    let catalog_path = root.join("catalog.json");
    let mut catalog: Value =
        serde_json::from_slice(&std::fs::read(&catalog_path).expect("catalog.json")).expect("json");
    let list = catalog.get_mut("templates").and_then(Value::as_array_mut).expect("templates list");
    for entry in list {
        let text = |k: &str| entry[k].as_str().unwrap_or_default().to_owned();
        let (kind, id) = (text("type"), text("id"));
        let folder = root.join("templates").join(&kind).join(&id);
        let Some(source) = find_source(&folder) else {
            eprintln!("{kind}/{id}: no template file, skipped");
            continue;
        };
        let content = project::open(&source).unwrap_or_else(|e| panic!("{kind}/{id}: {e}"));
        let bytes = project::to_bytes(&content.pages, 0).expect("saves");
        std::fs::write(folder.join("template.fe"), &bytes).expect("write");
        if source.file_name().is_some_and(|n| n != "template.fe") {
            std::fs::remove_file(&source).expect("remove old file");
        }
        for (i, page) in content.pages.iter().enumerate() {
            let picture = page_thumbnail(&page.doc, PAGE_SIZE, PAGE_SIZE);
            std::fs::write(folder.join(format!("page-{}.jpg", i + 1)), jpeg(&picture)).expect("write page");
            if i == 0 {
                std::fs::write(folder.join("thumb.jpg"), jpeg(&page_thumbnail(&page.doc, THUMB_SIZE, THUMB_SIZE)))
                    .expect("write thumb");
            }
        }
        let first = &content.pages[0].doc;
        entry["pages"] = content.pages.len().into();
        entry["width"] = first.width().into();
        entry["height"] = first.height().into();
        entry["bytes"] = bytes.len().into();
        let hash: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
        entry["sha256"] = hash.into();
        println!("{kind}/{id}: {} pages, {} KB", content.pages.len(), bytes.len() / 1000);
    }
    catalog["version"] = 2.into();
    std::fs::write(&catalog_path, serde_json::to_string_pretty(&catalog).expect("json") + "\n").expect("write catalog");
}
