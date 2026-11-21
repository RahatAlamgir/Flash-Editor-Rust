//! Copy / paste of pictures and layers (C# `ClipboardService`). Copies keep full transparency inside the program
//! and also go to the Windows clipboard, so other programs can paste them; pictures copied in other programs can
//! be pasted here too.

use fe_core::document::Layer;
use fe_core::imaging::PixelBuffer;

/// A picture on the clipboard and where it was copied from (when it came from this program).
#[derive(Clone)]
pub struct ClipImage {
    pub pixels: PixelBuffer,
    pub origin: Option<(i32, i32)>,
}

/// The program's own clipboard.
#[derive(Default)]
pub struct Clipboard {
    image: Option<ClipImage>,
    /// Layers copied as they are (objects stay editable when pasted).
    layers: Option<Vec<Layer>>,
    /// The Windows clipboard's change counter right after our copy: while it is the same, the clipboard still
    /// holds our picture, so the full-quality copy kept here is used.
    sequence: Option<u32>,
}

/// The Windows clipboard's change counter.
fn sequence() -> Option<u32> {
    #[cfg(windows)]
    {
        clipboard_win::raw::seq_num().map(|n| n.get())
    }
    #[cfg(not(windows))]
    {
        None
    }
}

impl Clipboard {
    /// Copies a picture (from canvas position `origin`).
    pub fn copy_image(&mut self, pixels: PixelBuffer, origin: (i32, i32)) {
        let to_windows = arboard::Clipboard::new().and_then(|mut c| {
            c.set_image(arboard::ImageData {
                width: pixels.width() as usize,
                height: pixels.height() as usize,
                bytes: std::borrow::Cow::Borrowed(pixels.data()),
            })
        });
        if let Err(err) = to_windows {
            // Another program may hold the clipboard; the copy inside the program still works.
            log::warn!("Could not put the picture on the Windows clipboard: {err}");
        }
        self.image = Some(ClipImage { pixels, origin: Some(origin) });
        self.layers = None;
        self.sequence = sequence();
    }

    /// Copies layers as they are, plus a picture of them for other programs.
    pub fn copy_layers(&mut self, layers: Vec<Layer>, picture: PixelBuffer, origin: (i32, i32)) {
        self.copy_image(picture, origin);
        self.layers = Some(layers);
    }

    /// True while our own copy is still what the Windows clipboard holds.
    fn ours(&self) -> bool {
        match (self.sequence, sequence()) {
            (Some(ours), Some(now)) => ours == now,
            _ => true, // cannot tell: trust the program's own copy
        }
    }

    /// The layers copied last, if nothing else was copied after them.
    pub fn layers(&self) -> Option<&[Layer]> {
        self.layers.as_deref().filter(|_| self.ours())
    }

    /// The picture to paste: our own copy while it is still on the clipboard, else a picture from another
    /// program, else our last copy.
    pub fn image(&self) -> Option<ClipImage> {
        if self.image.is_some() && self.ours() {
            return self.image.clone();
        }
        let from_windows = arboard::Clipboard::new()
            .and_then(|mut c| c.get_image())
            .ok()
            .and_then(|img| PixelBuffer::from_raw(img.width as u32, img.height as u32, img.bytes.into_owned()));
        match from_windows {
            Some(pixels) => Some(ClipImage { pixels, origin: None }),
            None => self.image.clone(),
        }
    }
}
