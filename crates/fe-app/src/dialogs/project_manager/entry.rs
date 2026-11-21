//! One project in the Project Manager, and reading the list in the background.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use egui::ColorImage;
use fe_core::io::project::{self, ProjectInfo};

/// What a card shows.
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub modified: Option<SystemTime>,
    pub size: u64,
    pub info: Option<ProjectInfo>,
    /// The project's preview picture, decoded.
    pub preview: Option<ColorImage>,
}

impl Entry {
    /// Reads one project's card (never fails: a damaged file shows "Cannot be read").
    pub fn read(path: &Path) -> Self {
        let meta = std::fs::metadata(path).ok();
        let preview = project::read_preview(path).and_then(|png| image::load_from_memory(&png).ok()).map(|img| {
            let rgba = img.into_rgba8();
            ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw())
        });
        Self {
            path: path.to_path_buf(),
            name: path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            modified: meta.as_ref().and_then(|m| m.modified().ok()),
            size: meta.map_or(0, |m| m.len()),
            info: project::read_info(path).ok(),
            preview,
        }
    }

    /// "3 pages  •  1280 × 720", or "Cannot be read".
    pub fn details(&self) -> String {
        match &self.info {
            Some(i) => {
                format!("{} page{}  •  {} × {}", i.pages, if i.pages == 1 { "" } else { "s" }, i.width, i.height)
            }
            None => "Cannot be read".into(),
        }
    }

    /// "Edited today 14:05", "Edited yesterday 09:12" or "Edited 3 Nov 2026" (local time).
    pub fn modified_text(&self) -> String {
        let Some(modified) = self.modified else { return String::new() };
        let when: chrono::DateTime<chrono::Local> = modified.into();
        let today = chrono::Local::now().date_naive();
        if when.date_naive() == today {
            format!("Edited today {}", when.format("%H:%M"))
        } else if Some(when.date_naive()) == today.pred_opt() {
            format!("Edited yesterday {}", when.format("%H:%M"))
        } else {
            format!("Edited {}", when.format("%-d %b %Y"))
        }
    }

    /// "120 KB" or "8.4 MB".
    pub fn size_text(&self) -> String {
        if self.size < 1024 * 1024 {
            format!("{} KB", (self.size / 1024).max(1))
        } else {
            format!("{:.1} MB", self.size as f64 / 1048576.0)
        }
    }
}

/// Every known project: the recent list plus the `.flash` files in the projects folder, newest first.
pub fn read_all(recent: &[String], folder: &Path) -> Vec<Entry> {
    let mut paths: Vec<PathBuf> = recent.iter().map(PathBuf::from).filter(|p| p.is_file()).collect();
    if let Ok(dir) = std::fs::read_dir(folder) {
        for path in dir.filter_map(Result::ok).map(|e| e.path()).filter(|p| project::is_project(p)) {
            let key = path.to_string_lossy().to_lowercase();
            if !paths.iter().any(|p| p.to_string_lossy().to_lowercase() == key) {
                paths.push(path);
            }
        }
    }
    let mut entries: Vec<Entry> = paths.iter().map(|p| Entry::read(p)).collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.modified));
    entries
}
