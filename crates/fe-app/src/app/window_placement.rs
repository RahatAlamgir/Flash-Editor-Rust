//! Remembers the window's size, position and maximized state in `settings\window.json` next to the exe,
//! like the C# `WindowPlacement`. The first start opens maximized.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::paths;

/// Where the window was when the program last closed.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WindowPlacement {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl WindowPlacement {
    /// The saved placement, or `None` on first start (or when the file is unreadable).
    pub fn load() -> Option<Self> {
        let text = std::fs::read_to_string(file()).ok()?;
        let placement: Self = serde_json::from_str(&text).ok()?;
        placement.is_sane().then_some(placement)
    }

    /// Writes the placement. Errors are only logged: losing it is harmless.
    pub fn save(&self) {
        let result = std::fs::create_dir_all(paths::settings_dir())
            .and_then(|()| std::fs::write(file(), serde_json::to_string_pretty(self).unwrap_or_default()));
        if let Err(err) = result {
            log::warn!("Could not save the window placement: {err}");
        }
    }

    /// Reads the current placement from egui (`None` while minimized or unknown).
    pub fn from_viewport(info: &egui::ViewportInfo) -> Option<Self> {
        if info.minimized == Some(true) {
            return None;
        }
        let rect = info.outer_rect?;
        // The position is the outer corner, the size is the inside (that is what the window is asked for when it opens);
        // saving the outer size made the window a little bigger every time the program was started.
        let inner = info.inner_rect.unwrap_or(rect);
        Some(Self {
            left: rect.left(),
            top: rect.top(),
            width: inner.width(),
            height: inner.height(),
            maximized: info.maximized.unwrap_or(false),
        })
    }

    /// Ignores nonsense such as a window of 0 width or far off every screen.
    fn is_sane(&self) -> bool {
        self.width >= 400.0
            && self.height >= 300.0
            && self.left.abs() < 20_000.0
            && self.top.abs() < 20_000.0
            && self.width < 20_000.0
            && self.height < 20_000.0
    }
}

fn file() -> PathBuf {
    paths::settings_dir().join("window.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_uses_the_csharp_style_names_and_reads_back() {
        let p = WindowPlacement { left: 10.0, top: 20.0, width: 1200.0, height: 800.0, maximized: true };
        let json = serde_json::to_string(&p).unwrap_or_default();
        assert!(json.contains("\"Maximized\":true"));
        assert_eq!(Some(p), serde_json::from_str(&json).ok());
    }

    #[test]
    fn broken_values_are_refused() {
        let tiny = WindowPlacement { left: 0.0, top: 0.0, width: 5.0, height: 5.0, maximized: false };
        assert!(!tiny.is_sane());
    }
}
