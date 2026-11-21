//! Folders the app uses. Like the C# version, everything lives next to the exe.

use std::path::PathBuf;

/// Folder that holds `flash-editor.exe`.
pub fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from)).unwrap_or_else(|| PathBuf::from("."))
}

/// `settings\` next to the exe (window placement, app settings, log file).
pub fn settings_dir() -> PathBuf {
    exe_dir().join("settings")
}
