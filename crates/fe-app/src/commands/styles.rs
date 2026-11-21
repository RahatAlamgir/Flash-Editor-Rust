//! Saved styles (C# `StyleLibrary`, "Save this look..."): the looks the user saved are kept in `settings\styles.json`
//! next to the program and are there in every project.

use fe_core::objects::style::{ObjectStyle, SavedStyle, styles_from_json, styles_to_json};

use crate::app::{FlashApp, paths};

fn file() -> std::path::PathBuf {
    paths::settings_dir().join("styles.json")
}

/// The saved looks (read from the file the first time).
pub fn list(app: &mut FlashApp) -> &Vec<SavedStyle> {
    app.styles.get_or_insert_with(|| std::fs::read_to_string(file()).map(|t| styles_from_json(&t)).unwrap_or_default())
}

fn save(app: &mut FlashApp) {
    let text = styles_to_json(list(app));
    let path = file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text); // saving the list is a convenience; never disturb the user
}

/// Saves the look of the selected object. A name that already exists is replaced.
pub fn add(app: &mut FlashApp, name: &str) {
    let name = name.trim().to_owned();
    let Some(object) = super::parts::target_object(app) else { return };
    if name.is_empty() {
        return;
    }
    let style = ObjectStyle::from(&object);
    let _ = list(app);
    if let Some(styles) = &mut app.styles {
        match styles.iter_mut().find(|s| s.name.eq_ignore_ascii_case(&name)) {
            Some(existing) => existing.style = style,
            None => styles.push(SavedStyle { name, style }),
        }
    }
    save(app);
    app.status = "Look saved. It is in Properties > Styles in every project.".into();
}

pub fn remove(app: &mut FlashApp, index: usize) {
    let _ = list(app);
    if let Some(styles) = &mut app.styles
        && index < styles.len()
    {
        styles.remove(index);
    }
    save(app);
}

/// Puts a saved look on the selected object.
pub fn apply(app: &mut FlashApp, index: usize) {
    let Some(style) = list(app).get(index).map(|s| s.style.clone()) else { return };
    super::objects::change_object(app, "Apply Style", |o| style.apply_to(o));
}
