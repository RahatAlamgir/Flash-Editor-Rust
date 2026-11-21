//! Finds AI model files and downloads them once (C# `ModelStore`). Everything lives in ONE place: the "models" folder
//! next to the program, so the whole app with its models is a single folder that can be copied or removed in one go.
//! Downloads go to a ".part" file and are checked (size and SHA-256) before use, so a broken or half-finished download
//! is never loaded.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use sha2::{Digest, Sha256};

use super::info::ModelInfo;

/// The "models" folder next to the program.
pub fn default_folder() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default().join("models")
}

pub struct ModelStore {
    folder: PathBuf,
}

impl ModelStore {
    /// `folder`: another folder (tests); `None` = the program's own "models" folder.
    pub fn new(folder: Option<PathBuf>) -> Self {
        Self { folder: folder.unwrap_or_else(default_folder) }
    }

    pub fn folder(&self) -> &Path {
        &self.folder
    }

    /// The installed model file, or (when missing) where a download would be saved.
    pub fn path_of(&self, model: &ModelInfo) -> PathBuf {
        self.find_installed(model).unwrap_or_else(|| self.folder.join(model.folder.unwrap_or("")).join(model.file_name))
    }

    pub fn is_installed(&self, model: &ModelInfo) -> bool {
        self.find_installed(model).is_some()
    }

    /// A model that comes as a zip may be left in the folder by an installer: it is unpacked the first time it is looked for.
    fn pending_archive(&self, model: &ModelInfo) -> PathBuf {
        self.folder.join(format!("{}.zip", model.folder.unwrap_or(model.file_name)))
    }

    fn unpack_pending(&self, model: &ModelInfo) {
        let zip = self.pending_archive(model);
        if model.archive_files.is_empty() || !zip.exists() {
            return;
        }
        let _ = Self::verify(model, &zip).and_then(|()| self.unpack(model, &zip));
        let _ = fs::remove_file(&zip); // damaged or used up: the program offers to download the model again
    }

    fn find_installed(&self, model: &ModelInfo) -> Option<PathBuf> {
        if !model.archive_files.is_empty() {
            self.unpack_pending(model);
            let dir = self.folder.join(model.folder.unwrap_or(""));
            let complete = model
                .archive_files
                .iter()
                .all(|(name, size)| fs::metadata(dir.join(name)).is_ok_and(|m| m.len() == *size));
            return complete.then(|| dir.join(model.file_name));
        }
        let file = self.folder.join(model.file_name);
        fs::metadata(&file).is_ok_and(|m| m.len() == model.size_bytes).then_some(file)
    }

    /// How much space the model's files use (also a half-finished download); 0 when nothing is there.
    pub fn bytes_on_disk(&self, model: &ModelInfo) -> u64 {
        self.files_of(model).iter().filter_map(|p| fs::metadata(p).ok()).map(|m| m.len()).sum()
    }

    /// Removes the model (its files and any half-finished download). Fails when a file is still open.
    pub fn delete(&self, model: &ModelInfo) -> std::io::Result<()> {
        for path in self.files_of(model) {
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
        if let Some(f) = model.folder {
            let dir = self.folder.join(f);
            if dir.is_dir() && fs::read_dir(&dir)?.next().is_none() {
                fs::remove_dir(dir)?;
            }
        }
        Ok(())
    }

    /// Removes only a half-finished download.
    pub fn delete_partial(&self, model: &ModelInfo) {
        for path in self.partial_files(model) {
            let _ = fs::remove_file(path);
        }
    }

    fn files_of(&self, model: &ModelInfo) -> Vec<PathBuf> {
        let mut list: Vec<PathBuf> = if model.archive_files.is_empty() {
            vec![self.folder.join(model.file_name)]
        } else {
            let dir = self.folder.join(model.folder.unwrap_or(""));
            model.archive_files.iter().map(|(n, _)| dir.join(n)).collect()
        };
        list.extend(self.partial_files(model));
        list
    }

    fn partial_files(&self, model: &ModelInfo) -> Vec<PathBuf> {
        if model.archive_files.is_empty() {
            vec![self.folder.join(format!("{}.part", model.file_name))]
        } else {
            vec![self.folder.join(format!("{}.zip.part", model.folder.unwrap_or(""))), self.pending_archive(model)]
        }
    }

    /// Downloads the model into the folder. `progress` gets 0..1. Stops early (and removes the part file) when `cancel`
    /// is set. The error says in plain words what went wrong.
    pub fn download(&self, model: &ModelInfo, progress: &dyn Fn(f64), cancel: &AtomicBool) -> Result<(), String> {
        fs::create_dir_all(&self.folder).map_err(|e| self.cannot_save(&e))?;
        let probe = self.folder.join(".write-test");
        File::create(&probe).map_err(|e| self.cannot_save(&e))?;
        let _ = fs::remove_file(probe);
        let target = self.folder.join(model.file_name);
        let part = if model.archive_files.is_empty() {
            self.folder.join(format!("{}.part", model.file_name))
        } else {
            self.folder.join(format!("{}.zip.part", model.folder.unwrap_or("")))
        };
        if let Err(e) = self.fetch(model, &part, progress, cancel) {
            let _ = fs::remove_file(&part);
            return Err(e);
        }
        Self::verify(model, &part)?;
        if model.archive_files.is_empty() {
            fs::rename(&part, &target).map_err(|e| e.to_string())?;
        } else {
            self.unpack(model, &part)?;
            let _ = fs::remove_file(&part);
        }
        Ok(())
    }

    fn cannot_save(&self, e: &std::io::Error) -> String {
        format!(
            "Flash Editor cannot save the AI model in its folder:\n{}\n\n({e})\nCopy the program to a folder you may write to, or start it once as administrator.",
            self.folder.display()
        )
    }

    fn fetch(&self, model: &ModelInfo, part: &Path, progress: &dyn Fn(f64), cancel: &AtomicBool) -> Result<(), String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(30 * 60)))
            .user_agent("FlashEditor/1.0")
            .build()
            .into();
        let response = agent.get(model.download_url).call().map_err(|e| format!("The download failed: {e}"))?;
        let mut input = response.into_body().into_reader();
        let mut output = File::create(part).map_err(|e| e.to_string())?;
        let mut buffer = vec![0u8; 1 << 16];
        let mut done = 0u64;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err("The download was cancelled.".into());
            }
            let n = input.read(&mut buffer).map_err(|e| format!("The download failed: {e}"))?;
            if n == 0 {
                break;
            }
            output.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
            done += n as u64;
            progress((done as f64 / model.size_bytes as f64).min(1.0));
        }
        Ok(())
    }

    /// An error in plain words when the file is not the expected model (the file is deleted).
    pub fn verify(model: &ModelInfo, path: &Path) -> Result<(), String> {
        let size = fs::metadata(path).map_or(0, |m| m.len());
        if size != model.size_bytes {
            let _ = fs::remove_file(path);
            return Err(format!(
                "The downloaded model has the wrong size ({size} bytes, expected {}). Please try again.",
                model.size_bytes
            ));
        }
        if model.sha256.is_empty() {
            return Ok(());
        }
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 1 << 20];
        loop {
            let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let hash: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if !hash.eq_ignore_ascii_case(model.sha256) {
            drop(file);
            let _ = fs::remove_file(path);
            return Err("The downloaded model is damaged (checksum mismatch). Please try again.".into());
        }
        Ok(())
    }

    /// Takes only the listed model files out of the (already verified) zip.
    fn unpack(&self, model: &ModelInfo, zip_path: &Path) -> Result<(), String> {
        let dir = self.folder.join(model.folder.unwrap_or(""));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let mut zip =
            zip::ZipArchive::new(File::open(zip_path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        for (name, size) in model.archive_files {
            let mut found = None;
            for i in 0..zip.len() {
                let f = zip.by_index(i).map_err(|e| e.to_string())?;
                if f.name().ok().and_then(|n| n.rsplit('/').next().map(str::to_owned)).as_deref() == Some(*name) {
                    found = Some(i);
                    break;
                }
            }
            let i = found.ok_or_else(|| format!("The downloaded model is missing {name}."))?;
            let mut f = zip.by_index(i).map_err(|e| e.to_string())?;
            if f.size() != *size {
                return Err(format!("{name} in the download has an unexpected size."));
            }
            let mut out = File::create(dir.join(name)).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut out).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
