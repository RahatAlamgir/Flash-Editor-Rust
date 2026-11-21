//! Pictures ready for drawing: premultiplied tiny-skia pixmaps with smaller copies (each half the size of the one
//! before), so a big photo drawn small stays smooth. Kept while the picture is in use.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use tiny_skia::{IntSize, Pixmap};

use crate::imaging::PixelBuffer;
use crate::objects::pictures;

/// A picture and its smaller copies (`levels[0]` = full size).
pub struct Mips {
    pub levels: Vec<Pixmap>,
}

impl Mips {
    /// The copy to draw at `scale` (drawn size / picture size) and how much it is smaller than the original.
    pub fn for_scale(&self, scale: f64) -> (&Pixmap, f64) {
        let mut level = 0;
        let mut factor = 1.0;
        while level + 1 < self.levels.len() && scale * factor * 2.0 <= 1.0 {
            level += 1;
            factor *= 2.0;
        }
        (&self.levels[level], factor)
    }
}

/// Straight RGBA to a premultiplied pixmap.
pub fn to_pixmap(picture: &PixelBuffer) -> Option<Pixmap> {
    let mut data = picture.data().to_vec();
    for p in data.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if a < 255 {
            for c in &mut p[..3] {
                *c = ((*c as u32 * a + 127) / 255) as u8;
            }
        }
    }
    Pixmap::from_vec(data, IntSize::from_wh(picture.width(), picture.height())?)
}

/// Half the size (2 x 2 box filter on premultiplied pixels).
fn half(p: &Pixmap) -> Option<Pixmap> {
    let (w, h) = ((p.width() / 2).max(1), (p.height() / 2).max(1));
    let mut out = Pixmap::new(w, h)?;
    let (src, sw) = (p.data(), p.width() as usize);
    let dst = out.data_mut();
    for y in 0..h as usize {
        for x in 0..w as usize {
            for c in 0..4 {
                let at = |dx: usize, dy: usize| {
                    let (sx, sy) = ((x * 2 + dx).min(sw - 1), (y * 2 + dy).min(p.height() as usize - 1));
                    src[(sy * sw + sx) * 4 + c] as u32
                };
                dst[(y * w as usize + x) * 4 + c] = ((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1) + 2) / 4) as u8;
            }
        }
    }
    Some(out)
}

type Entry = (Weak<Vec<u8>>, Arc<Mips>);

fn cache() -> &'static Mutex<HashMap<usize, Entry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<usize, Entry>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The drawable copies of a picture (made on first use).
pub fn mips(bytes: &Arc<Vec<u8>>) -> Option<Arc<Mips>> {
    let key = Arc::as_ptr(bytes) as usize;
    if let Ok(c) = cache().lock()
        && let Some((weak, mips)) = c.get(&key)
        && weak.upgrade().is_some_and(|b| Arc::ptr_eq(&b, bytes))
    {
        return Some(mips.clone());
    }
    let picture = pictures::decoded(bytes)?;
    let mut levels = vec![to_pixmap(&picture)?];
    while levels.last().is_some_and(|l| l.width() > 64 && l.height() > 64) {
        let next = half(levels.last()?)?;
        levels.push(next);
    }
    let mips = Arc::new(Mips { levels });
    if let Ok(mut c) = cache().lock() {
        c.retain(|_, (w, _)| w.strong_count() > 0);
        c.insert(key, (Arc::downgrade(bytes), mips.clone()));
    }
    Some(mips)
}
