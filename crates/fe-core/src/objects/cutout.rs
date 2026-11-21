//! Cut-out pictures (C# `ImageObject.Original`): a placed picture that remembers the picture as it was before its
//! background was removed, so the Refine brush can paint removed parts back. The original is stored in the project
//! like the picture itself (`"Original": "images/N.png"`).

use std::sync::Arc;

use serde_json::Value;

use super::{ObjectBox, StoredObject, new_picture_name, picture_bytes};
use crate::imaging::PixelBuffer;

impl StoredObject {
    /// A new placed picture that remembers `original` (same size as `source`).
    pub fn new_cutout(b: ObjectBox, source: &PixelBuffer, original: &PixelBuffer, name: &str) -> Self {
        let mut object = Self::new_image(b, picture_bytes(source), name);
        object.set_original(Some(original));
        object
    }

    /// The remembered original picture of a placed picture (PNG bytes).
    pub fn original_picture(&self) -> Option<&Arc<Vec<u8>>> {
        let name = self.json.get("Original")?.as_str()?;
        self.images.get(name)
    }

    /// Remembers (or forgets) the original picture.
    pub fn set_original(&mut self, original: Option<&PixelBuffer>) {
        if let Some(old) = self.json.get("Original").and_then(Value::as_str).map(str::to_owned) {
            self.images.remove(&old);
        }
        let Some(map) = self.json.as_object_mut() else { return };
        match original {
            Some(pixels) => {
                let file = new_picture_name();
                map.insert("Original".into(), file.clone().into());
                self.images.insert(file, picture_bytes(pixels));
            }
            None => {
                map.shift_remove("Original");
            }
        }
    }

    /// The same picture object with other pixels (the original stays); counts as a new revision.
    pub fn with_source(&self, source: &PixelBuffer) -> Self {
        let mut next = self.clone();
        let file = new_picture_name();
        let old = next.json.get("Source").and_then(Value::as_str).map(str::to_owned);
        if let Some(map) = next.json.as_object_mut() {
            map.insert("Source".into(), file.clone().into());
            let revision = map.get("Revision").and_then(Value::as_i64).unwrap_or(0) + 1;
            map.insert("Revision".into(), revision.into());
        }
        if let Some(old) = old {
            next.images.remove(&old);
        }
        next.images.insert(file, picture_bytes(source));
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::imaging::ColorRgba;

    #[test]
    fn a_cutout_remembers_its_original_and_keeps_it_when_the_pixels_change() {
        let original = PixelBuffer::filled(4, 4, ColorRgba::rgb(10, 20, 30));
        let source = PixelBuffer::filled(4, 4, ColorRgba::rgb(1, 2, 3));
        let object = StoredObject::new_cutout(ObjectBox::new(0.0, 0.0, 4.0, 4.0, 0.0), &source, &original, "Photo");
        assert!(object.original_picture().is_some());
        let edited = object.with_source(&PixelBuffer::filled(4, 4, ColorRgba::rgb(9, 9, 9)));
        assert!(edited.original_picture().is_some());
        assert_ne!(edited.picture().map(|p| p.as_ptr()), object.picture().map(|p| p.as_ptr()));
        assert_eq!(edited.json.get("Revision").and_then(|v| v.as_i64()), Some(1));
    }
}
