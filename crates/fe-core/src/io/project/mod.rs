//! `.fe` project files: a zip with `project.json`, a 480 px `preview.png` and `images/N.png`.
//! Compatible with the C# version both ways (see `docs/plan/01-architecture.md`, ".flash compatibility").

mod manifest;
pub(crate) use manifest::AdjustmentEntry;
mod png_stream;
mod read;
mod write;

use crate::document::Page;

pub use read::{open, open_from};
pub use write::{save, save_as, to_bytes, to_bytes_as};

/// File extension of projects.
pub const EXTENSION: &str = "fe";
pub(crate) const MANIFEST_ENTRY: &str = "project.json";
pub(crate) const PREVIEW_ENTRY: &str = "preview.png";
pub(crate) const IMAGE_FOLDER: &str = "images/";
/// Longest side of the preview picture.
pub const PREVIEW_SIZE: u32 = 480;

/// Everything in a project file.
#[derive(Debug)]
pub struct ProjectContent {
    pub pages: Vec<Page>,
    pub active_page: usize,
    /// The project was saved as a presentation.
    pub presentation: bool,
}

/// True when the path ends in `.fe` (any case).
pub fn is_project(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(EXTENSION))
}

/// What the Project Manager shows about a project without opening it.
#[derive(Clone, Debug, Default)]
pub struct ProjectInfo {
    pub pages: usize,
    /// Size of the first page.
    pub width: u32,
    pub height: u32,
    /// Saved as a presentation.
    pub presentation: bool,
}

/// Reads only the page list of a project (fast: no pictures are decoded).
pub fn read_info(path: impl AsRef<std::path::Path>) -> Result<ProjectInfo, crate::io::IoError> {
    use std::io::Read;
    let not_project = || crate::io::IoError::CannotOpen("This is not a Flash Editor project.".into());
    let file = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(|_| not_project())?;
    let mut text = Vec::new();
    zip.by_name(MANIFEST_ENTRY).map_err(|_| not_project())?.read_to_end(&mut text)?;
    let json: serde_json::Value = serde_json::from_slice(&text).map_err(|_| not_project())?;
    let field = |v: &serde_json::Value, name: &str| v.get(name).and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    let presentation = json.get("ProjectType").and_then(|t| t.as_str()) == Some("presentation");
    match json.get("Pages").and_then(|p| p.as_array()).filter(|p| !p.is_empty()) {
        Some(pages) => Ok(ProjectInfo {
            pages: pages.len(),
            width: field(&pages[0], "Width"),
            height: field(&pages[0], "Height"),
            presentation,
        }),
        None => {
            Ok(ProjectInfo { pages: 1, width: field(&json, "Width"), height: field(&json, "Height"), presentation })
        }
    }
}

/// Just the preview picture of a project (for the Project Manager cards), as PNG bytes.
pub fn read_preview(path: impl AsRef<std::path::Path>) -> Option<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).ok()?;
    let mut entry = zip.by_name(PREVIEW_ENTRY).ok()?;
    let mut data = Vec::new();
    entry.read_to_end(&mut data).ok()?;
    Some(data)
}
