//! Text (C# `TextObject` drawing, `SpacedText`, `ArcText`): installed fonts are found with `fontdb`, letters are shaped
//! with `rustybuzz` (kerning) and drawn as outlines, so fill, outline, gradients and curved text all use one path.

mod arc;
mod data;
mod editing;
pub mod fonts;
mod layout;
mod outline;
mod presets;

pub use data::{TextAlign, TextData};
pub use editing::TextCursor;
pub use layout::{CaretPos, Laid, lay_out};
pub use presets::{TEXT_PRESETS, TextPreset};

use crate::objects::{ObjectBox, StoredObject};

/// Makes the box as big as the text (C# `MeasureText`), keeping the top-left corner of the box.
pub fn refit(o: &mut StoredObject, t: &TextData) {
    let (w, h) = crate::render::measure_text(t);
    if let Some(b) = o.box_() {
        o.set_box(ObjectBox { width: w, height: h, ..b });
    }
}
