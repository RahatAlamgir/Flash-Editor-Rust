//! File > Batch Convert (C# `BatchConverter`): turns every picture in a folder into another format and / or a smaller size.
//! The originals are never changed and existing files are never overwritten (a number is added to the new name).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::export::{ExportSettings, save_with};
use super::format::{ImageFileFormat, can_open};

/// What to do with every picture in a folder.
#[derive(Clone, Debug)]
pub struct BatchOptions {
    pub input: PathBuf,
    pub output: PathBuf,
    pub format: ImageFileFormat,
    pub quality: u8,
    /// Pictures wider than this are made smaller (0 = no limit). Pictures are never enlarged.
    pub max_width: u32,
    pub max_height: u32,
    pub include_subfolders: bool,
}

/// The outcome of a batch run.
#[derive(Debug, Default)]
pub struct BatchResult {
    pub converted: usize,
    pub failed: Vec<String>,
}

fn collect(folder: &Path, deep: bool, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if deep {
                collect(&path, deep, out);
            }
        } else if can_open(&path) {
            out.push(path);
        }
    }
}

/// The pictures that would be converted (never the ones inside the output folder).
pub fn find_pictures(options: &BatchOptions) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(&options.input, options.include_subfolders, &mut files);
    let output = std::fs::canonicalize(&options.output).unwrap_or_else(|_| options.output.clone());
    files.retain(|f| !std::fs::canonicalize(f).is_ok_and(|c| c.starts_with(&output)));
    files.sort_by_key(|f| f.to_string_lossy().to_lowercase());
    files
}

/// The size a picture gets: it fits inside the limits, keeps its proportions and is never enlarged.
pub fn fit_size(width: u32, height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let mut scale: f64 = 1.0;
    if max_width > 0 {
        scale = scale.min(f64::from(max_width) / f64::from(width));
    }
    if max_height > 0 {
        scale = scale.min(f64::from(max_height) / f64::from(height));
    }
    (
        ((f64::from(width) * scale).round_ties_even() as u32).max(1),
        ((f64::from(height) * scale).round_ties_even() as u32).max(1),
    )
}

/// A file name that does not exist yet: "name.png", then "name (2).png", "name (3).png"...
pub fn unique_path(folder: &Path, name: &str, extension: &str) -> PathBuf {
    let mut path = folder.join(format!("{name}{extension}"));
    let mut i = 2;
    while path.exists() {
        path = folder.join(format!("{name} ({i}){extension}"));
        i += 1;
    }
    path
}

/// Converts the pictures. `progress` gets (done, total); `cancel` stops after the picture being converted.
pub fn run(options: &BatchOptions, progress: &dyn Fn(usize, usize), cancel: &AtomicBool) -> BatchResult {
    let files = find_pictures(options);
    let mut result = BatchResult::default();
    let _ = std::fs::create_dir_all(&options.output);
    for (i, file) in files.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        progress(i, files.len());
        let name = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let converted = (|| -> Result<(), String> {
            let image = super::decode::load(file).map_err(|e| e.to_string())?;
            let (w, h) = fit_size(image.width(), image.height(), options.max_width, options.max_height);
            // The folder layout is kept when subfolders are included.
            let relative = file.parent().and_then(|p| p.strip_prefix(&options.input).ok()).unwrap_or(Path::new(""));
            let folder = options.output.join(relative);
            std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            let stem = file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let target = unique_path(&folder, &stem, options.format.extension());
            save_with(&image, target, &ExportSettings::new(options.format, options.quality, w, h))
                .map_err(|e| e.to_string())
        })();
        match converted {
            Ok(()) => result.converted += 1,
            Err(e) => result.failed.push(format!("{name}: {e}")),
        }
    }
    progress(files.len(), files.len());
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::{ColorRgba, PixelBuffer};

    #[test]
    fn pictures_fit_inside_the_limits_and_are_never_enlarged() {
        assert_eq!(fit_size(4000, 3000, 1000, 0), (1000, 750));
        assert_eq!(fit_size(4000, 3000, 1000, 500), (667, 500));
        assert_eq!(fit_size(400, 300, 1000, 1000), (400, 300));
        assert_eq!(fit_size(400, 300, 0, 0), (400, 300));
    }

    #[test]
    fn a_folder_is_converted_without_touching_originals_or_overwriting() {
        let root = std::env::temp_dir().join(format!("fe-batch-{}", std::process::id()));
        let (input, output) = (root.join("in"), root.join("out"));
        std::fs::create_dir_all(&input).expect("folder");
        let image = PixelBuffer::filled(200, 100, ColorRgba::rgb(10, 200, 10));
        crate::io::encode::save(&image, input.join("a.png"), ImageFileFormat::Png, 90, None).expect("saved");
        std::fs::write(input.join("notes.txt"), "x").expect("file");
        let options = BatchOptions {
            input: input.clone(),
            output: output.clone(),
            format: ImageFileFormat::Jpeg,
            quality: 80,
            max_width: 100,
            max_height: 0,
            include_subfolders: false,
        };
        assert_eq!(find_pictures(&options).len(), 1);
        let first = run(&options, &|_, _| {}, &AtomicBool::new(false));
        let second = run(&options, &|_, _| {}, &AtomicBool::new(false));
        assert_eq!((first.converted, second.converted), (1, 1));
        assert!(output.join("a.jpg").exists() && output.join("a (2).jpg").exists());
        let small = crate::io::decode::load(output.join("a.jpg")).expect("readable");
        assert_eq!((small.width(), small.height()), (100, 50));
        assert!(input.join("a.png").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
