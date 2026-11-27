//! Key names as the C# version stores them in the settings file (WPF names such as "OemPlus", "D1", "F5"), and the
//! text people read ("Ctrl+Shift+S", "Ctrl++").

use egui::{Key, Modifiers};

/// The WPF names that differ from egui's, with the egui key.
const SPECIAL: [(&str, Key); 17] = [
    ("OemPlus", Key::Equals),
    ("OemMinus", Key::Minus),
    ("OemQuotes", Key::Quote),
    ("OemSemicolon", Key::Semicolon),
    ("OemOpenBrackets", Key::OpenBracket),
    ("OemCloseBrackets", Key::CloseBracket),
    ("OemComma", Key::Comma),
    ("OemPeriod", Key::Period),
    ("OemQuestion", Key::Slash),
    ("OemTilde", Key::Backtick),
    ("OemBackslash", Key::Backslash),
    ("Back", Key::Backspace),
    ("Return", Key::Enter),
    ("Next", Key::PageDown),
    ("Prior", Key::PageUp),
    ("Escape", Key::Escape),
    ("Space", Key::Space),
];

/// The WPF name of a key, or `None` for keys that cannot be a shortcut.
pub fn wpf_name(key: Key) -> Option<String> {
    if key == Key::Plus {
        return Some("OemPlus".into()); // the same key as =
    }
    if let Some((name, _)) = SPECIAL.iter().find(|(_, k)| *k == key) {
        return Some((*name).to_owned());
    }
    let name = key.name();
    if name.len() == 1 && name.as_bytes()[0].is_ascii_digit() {
        return Some(format!("D{name}"));
    }
    Some(name.to_owned())
}

/// The key for a WPF name.
pub fn from_wpf(name: &str) -> Option<Key> {
    if let Some((_, key)) = SPECIAL.iter().find(|(n, _)| *n == name) {
        return Some(*key);
    }
    if name.len() == 2 && name.starts_with('D') && name.as_bytes()[1].is_ascii_digit() {
        return Key::from_name(&name[1..]);
    }
    Key::from_name(name)
}

/// The key as people write it.
pub fn pretty_key(name: &str) -> String {
    match name {
        "OemPlus" => "+".into(),
        "OemMinus" => "-".into(),
        "OemQuotes" => "'".into(),
        "OemSemicolon" => ";".into(),
        "OemOpenBrackets" => "[".into(),
        "OemCloseBrackets" => "]".into(),
        "OemComma" => ",".into(),
        "OemPeriod" => ".".into(),
        "OemQuestion" => "/".into(),
        "OemTilde" => "`".into(),
        "OemBackslash" => "\\".into(),
        d if d.len() == 2 && d.starts_with('D') && d.as_bytes()[1].is_ascii_digit() => d[1..].to_owned(),
        other => other.to_owned(),
    }
}

/// "Ctrl+Shift+OemCloseBrackets" as "Ctrl+Shift+]".
pub fn pretty(text: &str) -> String {
    text.split('+').filter(|p| !p.is_empty()).map(pretty_key).collect::<Vec<_>>().join("+")
}

/// A shortcut as text: the modifiers and the key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gesture {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: Key,
}

impl Gesture {
    pub fn parse(text: &str) -> Option<Self> {
        let parts: Vec<&str> = text.split('+').collect();
        let (key, mods) = parts.split_last()?;
        Some(Self {
            ctrl: mods.contains(&"Ctrl"),
            alt: mods.contains(&"Alt"),
            shift: mods.contains(&"Shift"),
            key: from_wpf(key)?,
        })
    }

    pub fn text(&self) -> Option<String> {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl".to_owned());
        }
        if self.alt {
            parts.push("Alt".to_owned());
        }
        if self.shift {
            parts.push("Shift".to_owned());
        }
        parts.push(wpf_name(self.key)?);
        Some(parts.join("+"))
    }

    /// True when this press is the shortcut.
    pub fn matches(&self, key: Key, mods: Modifiers) -> bool {
        let same_key = self.key == key || (self.key == Key::Equals && key == Key::Plus);
        same_key && mods.command == self.ctrl && mods.alt == self.alt && mods.shift == self.shift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_with_the_csharp_settings_text() {
        for text in [
            "Ctrl+S",
            "Ctrl+Shift+S",
            "Ctrl+Alt+N",
            "F1",
            "Shift+F5",
            "Ctrl+OemCloseBrackets",
            "Ctrl+D0",
            "Ctrl+Shift+OemSemicolon",
            "Ctrl+Tab",
            "Ctrl+OemPlus",
        ] {
            let g = Gesture::parse(text).expect(text);
            assert_eq!(g.text().as_deref(), Some(text), "{text}");
        }
        assert_eq!(pretty("Ctrl+Shift+OemCloseBrackets"), "Ctrl+Shift+]");
        assert_eq!(pretty("Ctrl+D1"), "Ctrl+1");
    }

    #[test]
    fn a_press_matches_only_its_own_modifiers() {
        let g = Gesture::parse("Ctrl+Shift+S").expect("parsed");
        assert!(g.matches(Key::S, Modifiers::COMMAND | Modifiers::SHIFT));
        assert!(!g.matches(Key::S, Modifiers::COMMAND));
        assert!(!g.matches(Key::S, Modifiers::COMMAND | Modifiers::SHIFT | Modifiers::ALT));
        assert!(Gesture::parse("Ctrl+OemPlus").expect("parsed").matches(Key::Plus, Modifiers::COMMAND));
    }
}
