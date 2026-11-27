//! Settings kept between runs, in `settings\app.json` next to the exe (like the C# `AppSettings`), and the
//! list of known projects (`settings\projects.json`, like the C# `ProjectLibrary`).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::paths;

/// Everything in `app.json`. Unknown or damaged files fall back to the defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Settings {
    pub undo_steps: usize,
    pub icon_style: String,
    pub demo_created: bool,
    pub tour_seen: bool,
    pub recent_colors: Vec<String>,
    pub swatches: Vec<String>,
    pub brand_colors: Vec<String>,
    pub brand_fonts: Vec<String>,
    pub show_rulers: bool,
    pub show_grid: bool,
    pub show_guides: bool,
    pub snap_enabled: bool,
    pub grid_size: f64,
    pub snap_to_objects: bool,
    pub snap_to_canvas: bool,
    pub snap_to_spacing: bool,
    pub snap_to_grid: bool,
    pub snap_to_guides: bool,
    /// Where new projects are saved (empty = Documents\Flash Editor Projects).
    pub projects_folder: String,
    /// Projects opened or saved lately, newest first.
    pub recent: Vec<String>,
    /// Keyboard shortcuts the user changed ("cmd:Save project" = "Ctrl+S", "tool:Brush" = "B"; "" = none).
    pub shortcuts: std::collections::BTreeMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            undo_steps: fe_core::history::DEFAULT_STEPS,
            icon_style: "Plain".into(),
            demo_created: false,
            tour_seen: false,
            recent_colors: Vec::new(),
            swatches: Vec::new(),
            brand_colors: Vec::new(),
            brand_fonts: Vec::new(),
            show_rulers: false,
            show_grid: false,
            show_guides: true,
            snap_enabled: true,
            grid_size: 50.0,
            snap_to_objects: true,
            snap_to_canvas: true,
            snap_to_spacing: true,
            snap_to_grid: true,
            snap_to_guides: true,
            projects_folder: String::new(),
            recent: Vec::new(),
            shortcuts: std::collections::BTreeMap::new(),
        }
    }
}

/// How many projects the recent list keeps.
const MAX_RECENT: usize = 60;

impl Settings {
    fn file() -> PathBuf {
        paths::settings_dir().join("app.json")
    }

    /// Reads the settings (defaults when there is no file or it is damaged).
    pub fn load() -> Self {
        std::fs::read_to_string(Self::file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }

    /// Writes the settings. A folder that cannot be written only means the choice lasts until the program closes.
    pub fn save(&self) {
        let result = std::fs::create_dir_all(paths::settings_dir())
            .and_then(|()| std::fs::write(Self::file(), serde_json::to_string_pretty(self).unwrap_or_default()));
        if let Err(err) = result {
            log::warn!("Could not save the settings: {err}");
        }
    }

    /// The folder for new projects.
    pub fn projects_folder(&self) -> PathBuf {
        if self.projects_folder.trim().is_empty() {
            default_projects_folder()
        } else {
            PathBuf::from(&self.projects_folder)
        }
    }

    /// Puts a project at the top of the recent list.
    pub fn add_recent(&mut self, path: &std::path::Path) {
        let path = path.to_string_lossy().to_string();
        self.recent.retain(|p| !p.eq_ignore_ascii_case(&path));
        self.recent.insert(0, path);
        self.recent.truncate(MAX_RECENT);
    }

    /// Takes a project off the recent list.
    pub fn forget_recent(&mut self, path: &std::path::Path) {
        let path = path.to_string_lossy();
        self.recent.retain(|p| !p.eq_ignore_ascii_case(&path));
    }
}

/// Documents\Flash Editor Projects (the same folder the C# version uses).
pub fn default_projects_folder() -> PathBuf {
    dirs::document_dir().unwrap_or_else(paths::exe_dir).join("Flash Editor Projects")
}

/// A file name made safe and short ("Untitled project" when nothing is left), like the C# `CleanName`.
pub fn clean_name(name: &str) -> String {
    let bad = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let cleaned: String = name.chars().filter(|c| !bad.contains(c) && !c.is_control()).collect();
    let cleaned = cleaned.trim().trim_end_matches('.').to_owned();
    match cleaned.chars().count() {
        0 => "Untitled project".into(),
        n if n > 80 => cleaned.chars().take(80).collect::<String>().trim_end().to_owned(),
        _ => cleaned,
    }
}

/// A path for a new project in `folder` that does not exist yet ("Name.fe", "Name (2).fe"...).
pub fn new_project_path(folder: &std::path::Path, name: &str) -> PathBuf {
    let name = clean_name(name);
    let mut path = folder.join(format!("{name}.fe"));
    let mut i = 2;
    while path.exists() {
        path = folder.join(format!("{name} ({i}).fe"));
        i += 1;
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_cleaned_like_the_csharp_version() {
        assert_eq!("Myposter", clean_name("  My/poster?.  ")); // forbidden characters are dropped, like in C#
        assert_eq!("Untitled project", clean_name("..."));
        assert_eq!(80, clean_name(&"x".repeat(200)).len());
    }

    #[test]
    fn recent_projects_move_to_the_top_without_repeats() {
        let mut s = Settings::default();
        s.add_recent(std::path::Path::new(r"C:\a.flash"));
        s.add_recent(std::path::Path::new(r"C:\b.flash"));
        s.add_recent(std::path::Path::new(r"c:\A.flash"));
        assert_eq!(vec![r"c:\A.flash", r"C:\b.flash"], s.recent);
    }

    #[test]
    fn a_damaged_settings_file_gives_the_defaults() {
        let s: Settings = serde_json::from_str("{\"UndoSteps\": 12, \"Unknown\": 5}").unwrap_or_default();
        assert_eq!(12, s.undo_steps);
        assert!(s.show_guides);
    }
}
