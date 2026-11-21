//! Decoded pictures of image objects and picture frames, kept while their PNG bytes are in use, so a picture is
//! decoded once however often its object is redrawn (resizing, turning).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use crate::imaging::PixelBuffer;

type Entry = (Weak<Vec<u8>>, Arc<PixelBuffer>);

fn cache() -> &'static Mutex<HashMap<usize, Entry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<usize, Entry>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Remembers the decoded pixels of `bytes` (when the pixels are already known).
pub fn remember(bytes: &Arc<Vec<u8>>, pixels: Arc<PixelBuffer>) {
    if let Ok(mut c) = cache().lock() {
        c.retain(|_, (w, _)| w.strong_count() > 0); // forget pictures nothing uses any more
        c.insert(Arc::as_ptr(bytes) as usize, (Arc::downgrade(bytes), pixels));
    }
}

/// The pixels of a picture (decoded on first use). `None` when the bytes are not a readable picture.
pub fn decoded(bytes: &Arc<Vec<u8>>) -> Option<Arc<PixelBuffer>> {
    let key = Arc::as_ptr(bytes) as usize;
    if let Ok(c) = cache().lock()
        && let Some((weak, pixels)) = c.get(&key)
        && weak.upgrade().is_some_and(|b| Arc::ptr_eq(&b, bytes))
    {
        return Some(pixels.clone());
    }
    let pixels = Arc::new(crate::io::decode::load_bytes(bytes, Some("png")).ok()?);
    remember(bytes, pixels.clone());
    Some(pixels)
}
