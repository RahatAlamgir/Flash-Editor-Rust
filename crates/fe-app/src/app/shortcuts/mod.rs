//! All keyboard shortcuts, with the changes the user made (C# `Shortcuts`). Commands need Ctrl or Alt (or an F key); tools
//! use a single key. The changes are kept in the settings file in the same form as the C# version ("cmd:Save project" =
//! "Ctrl+S"), so both programs read them.

mod keys;
pub mod run;

use egui::{Key, Modifiers};

pub use keys::{Gesture, pretty};
pub use run::run;

use crate::app::settings::Settings;
use crate::tools::ToolKind;

/// One thing that has a keyboard shortcut: a command (Ctrl+S) or a tool (V).
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub section: &'static str,
    pub title: &'static str,
    /// The shortcut the program starts with.
    pub default: String,
    pub tool: Option<ToolKind>,
}

/// Section, title and the normal shortcut of every command (the order of the list in the window).
pub const COMMANDS: [(&str, &str, &str); 43] = [
    ("File", "New", "Ctrl+N"),
    ("File", "Open", "Ctrl+O"),
    ("File", "Save project", "Ctrl+S"),
    ("File", "Save project as", "Ctrl+Shift+S"),
    ("File", "Export as picture", "Ctrl+Alt+S"),
    ("File", "Add page", "Ctrl+Alt+N"),
    ("File", "Close project", "Ctrl+W"),
    ("File", "Next picture", "Ctrl+Tab"),
    ("File", "Previous picture", "Ctrl+Shift+Tab"),
    ("Edit", "Undo", "Ctrl+Z"),
    ("Edit", "Redo", "Ctrl+Y"),
    ("Edit", "Copy", "Ctrl+C"),
    ("Edit", "Cut", "Ctrl+X"),
    ("Edit", "Paste", "Ctrl+V"),
    ("Edit", "Select all", "Ctrl+A"),
    ("Edit", "Deselect", "Ctrl+Shift+D"),
    ("Edit", "Invert selection", "Ctrl+Shift+I"),
    ("Image", "Image size", "Ctrl+Alt+I"),
    ("Image", "Canvas size", "Ctrl+Alt+C"),
    ("Image", "Remove background (AI)", "Ctrl+Alt+B"),
    ("Image", "Remove / fill selection (AI)", "Shift+F5"),
    ("Image", "Recolor", "Ctrl+Shift+R"),
    ("Layer", "New layer", "Ctrl+Shift+N"),
    ("Layer", "Duplicate", "Ctrl+D"),
    ("Layer", "Merge down", "Ctrl+E"),
    ("Layer", "Lock / unlock", "Ctrl+L"),
    ("Layer", "Group", "Ctrl+G"),
    ("Layer", "Ungroup", "Ctrl+Shift+G"),
    ("Layer", "Bring forward", "Ctrl+OemCloseBrackets"),
    ("Layer", "Send backward", "Ctrl+OemOpenBrackets"),
    ("Layer", "Bring to front", "Ctrl+Shift+OemCloseBrackets"),
    ("Layer", "Send to back", "Ctrl+Shift+OemOpenBrackets"),
    ("View", "Zoom in", "Ctrl+OemPlus"),
    ("View", "Zoom out", "Ctrl+OemMinus"),
    ("View", "Fit on screen", "Ctrl+D0"),
    ("View", "Actual size (100%)", "Ctrl+D1"),
    ("View", "Rulers", "Ctrl+R"),
    ("View", "Grid", "Ctrl+OemQuotes"),
    ("View", "Guides", "Ctrl+OemSemicolon"),
    ("View", "Snapping", "Ctrl+Shift+OemSemicolon"),
    ("View", "Keyboard shortcuts", "F1"),
    // Not in the C# list, but they have shortcuts in both programs.
    ("Edit", "Rename layer", "F2"),
    ("View", "Speed overlay", "F12"),
];

/// Keys that keep their job (the tools and the canvas use them), so they cannot be given to a tool.
const RESERVED_TOOL_KEYS: [Key; 14] = [
    Key::Space,
    Key::Escape,
    Key::Delete,
    Key::Backspace,
    Key::Enter,
    Key::Tab,
    Key::X,
    Key::D,
    Key::ArrowLeft,
    Key::ArrowRight,
    Key::ArrowUp,
    Key::ArrowDown,
    Key::OpenBracket,
    Key::CloseBracket,
];

/// Every command, then every tool.
pub fn entries() -> Vec<Entry> {
    let mut list: Vec<Entry> = COMMANDS
        .iter()
        .map(|(section, title, default)| Entry {
            id: format!("cmd:{title}"),
            section,
            title,
            default: (*default).to_owned(),
            tool: None,
        })
        .collect();
    list.extend(ToolKind::all().map(|t| Entry {
        id: format!("tool:{}", t.name()),
        section: "Tools",
        title: t.name(),
        default: t.shortcut().to_string(),
        tool: Some(t),
    }));
    list
}

/// The shortcut now in use as text the program can read back ("" = none).
pub fn current(settings: &Settings, entry: &Entry) -> String {
    settings.shortcuts.get(&entry.id).cloned().unwrap_or_else(|| entry.default.clone())
}

pub fn is_changed(settings: &Settings, entry: &Entry) -> bool {
    settings.shortcuts.contains_key(&entry.id)
}

/// The command a key press runs, if any (the title of its entry).
pub fn find(settings: &Settings, key: Key, mods: Modifiers) -> Option<&'static str> {
    COMMANDS.iter().find_map(|(_, title, default)| {
        let text = settings.shortcuts.get(&format!("cmd:{title}")).map_or(*default, String::as_str);
        Gesture::parse(text).filter(|g| g.matches(key, mods)).map(|_| *title)
    })
}

/// The shortcut of a command as people write it, for menus ("" = none).
pub fn text_for(settings: &Settings, title: &str) -> String {
    let default = COMMANDS.iter().find(|(_, t, _)| *t == title).map_or("", |(_, _, d)| *d);
    pretty(settings.shortcuts.get(&format!("cmd:{title}")).map_or(default, String::as_str))
}

/// Turns the keys that were pressed into the text kept in the settings, or says why they cannot be used.
pub fn from_keys(entry: &Entry, key: Key, mods: Modifiers) -> Result<String, String> {
    if entry.tool.is_some() {
        if mods.command || mods.alt || mods.shift {
            return Err("A tool shortcut is a single key, without Ctrl, Alt or Shift.".into());
        }
        if RESERVED_TOOL_KEYS.contains(&key) {
            let name = keys::wpf_name(key).unwrap_or_default();
            return Err(format!("{} is used by the program itself. Choose another key.", keys::pretty_key(&name)));
        }
        return keys::wpf_name(key).ok_or_else(|| "That key cannot be used.".to_owned());
    }
    let function_key = (Key::F1..=Key::F24).contains(&key);
    if !function_key && !mods.command && !mods.alt {
        return Err("Use Ctrl or Alt together with a key (or one of the F keys).".into());
    }
    Gesture { ctrl: mods.command, alt: mods.alt, shift: mods.shift, key }
        .text()
        .ok_or_else(|| "That key cannot be used.".to_owned())
}

/// Gives `entry` a new shortcut (text from [`from_keys`], or "" for none). Another command that had it loses it; its
/// title is returned.
pub fn set(settings: &mut Settings, entry: &Entry, text: &str) -> Option<&'static str> {
    let mut taken = None;
    if !text.is_empty() && entry.tool.is_none() {
        for (_, title, default) in COMMANDS {
            let id = format!("cmd:{title}");
            let used = settings.shortcuts.get(&id).map_or(default, String::as_str);
            if id != entry.id && used.eq_ignore_ascii_case(text) {
                settings.shortcuts.insert(id, String::new());
                taken = Some(title);
            }
        }
    }
    if text == entry.default {
        settings.shortcuts.remove(&entry.id);
    } else {
        settings.shortcuts.insert(entry.id.clone(), text.to_owned());
    }
    taken
}

pub fn reset(settings: &mut Settings, entry: &Entry) {
    settings.shortcuts.remove(&entry.id);
}

/// The letter or digit a tool answers to now (`None` = no shortcut).
pub fn tool_key(settings: &Settings, tool: ToolKind) -> Option<char> {
    let text = settings
        .shortcuts
        .get(&format!("tool:{}", tool.name()))
        .map_or_else(|| tool.shortcut().to_string(), Clone::clone);
    let key = keys::from_wpf(&text)?;
    let name = key.name();
    let mut chars = name.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) if c.is_ascii_alphanumeric() => Some(c.to_ascii_uppercase()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_is_a_readable_unique_shortcut() {
        let mut seen = std::collections::HashSet::new();
        for (_, title, default) in COMMANDS {
            let g = Gesture::parse(default).unwrap_or_else(|| panic!("{title}: {default}"));
            assert!(seen.insert(g.text()), "{title} shares {default}");
        }
    }

    #[test]
    fn a_new_shortcut_takes_the_old_owners_and_reset_brings_the_default_back() {
        let mut s = Settings::default();
        let entries = entries();
        let save = entries.iter().find(|e| e.title == "Save project").expect("entry");
        let taken = set(&mut s, save, "Ctrl+D");
        assert_eq!(taken, Some("Duplicate"));
        assert_eq!(find(&s, Key::D, Modifiers::COMMAND), Some("Save project"));
        assert_eq!(find(&s, Key::S, Modifiers::COMMAND), None);
        assert_eq!(text_for(&s, "Duplicate"), "");
        reset(&mut s, save);
        assert_eq!(find(&s, Key::S, Modifiers::COMMAND), Some("Save project"));
        assert!(!is_changed(&s, save));
    }

    #[test]
    fn rules_for_new_keys() {
        let entries = entries();
        let save = entries.iter().find(|e| e.title == "Save project").expect("entry");
        let brush = entries.iter().find(|e| e.tool == Some(ToolKind::Brush)).expect("brush");
        assert!(from_keys(save, Key::K, Modifiers::NONE).is_err());
        assert_eq!(from_keys(save, Key::K, Modifiers::COMMAND | Modifiers::SHIFT).as_deref(), Ok("Ctrl+Shift+K"));
        assert_eq!(from_keys(save, Key::F7, Modifiers::NONE).as_deref(), Ok("F7"));
        assert!(from_keys(brush, Key::X, Modifiers::NONE).is_err());
        assert!(from_keys(brush, Key::K, Modifiers::COMMAND).is_err());
        assert_eq!(from_keys(brush, Key::N, Modifiers::NONE).as_deref(), Ok("N"));
    }

    #[test]
    fn a_tool_answers_to_its_own_or_a_chosen_key() {
        let mut s = Settings::default();
        assert_eq!(tool_key(&s, ToolKind::Brush), Some('B'));
        let id = format!("tool:{}", ToolKind::Brush.name());
        s.shortcuts.insert(id.clone(), "N".into());
        assert_eq!(tool_key(&s, ToolKind::Brush), Some('N'));
        s.shortcuts.insert(id, String::new());
        assert_eq!(tool_key(&s, ToolKind::Brush), None);
    }
}
