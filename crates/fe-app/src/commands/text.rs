//! Text commands (C# `TextTool.PlaceText / EditExisting`, `ObjectPropertiesViewModel.ChangeText`): new text,
//! editing the words, and Properties changes that keep the box fitted to the text.

use fe_core::text::TextData;

use super::objects::change_object;
use crate::app::FlashApp;

/// The name of a text layer: its first line (C# `SuggestedLayerName`).
pub fn layer_name(text: &str) -> String {
    let first = text.split('\n').next().unwrap_or("").trim();
    if first.is_empty() {
        "Text".into()
    } else if first.chars().count() > 24 {
        format!("{}...", first.chars().take(24).collect::<String>())
    } else {
        first.to_owned()
    }
}

/// A Properties change of the selected text. The box follows the size of the text, its top-left stays where it is.
pub fn change_text(app: &mut FlashApp, what: &'static str, mut change: impl FnMut(&mut TextData)) {
    change_object(app, what, |o| {
        let Some(mut t) = o.text() else { return };
        change(&mut t);
        o.set_text(&t);
        fe_core::text::refit(o, &t);
    });
}
