//! The installed fonts. Found once, the first time text is drawn or the font list is shown.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use rustybuzz::Face;

fn database() -> &'static fontdb::Database {
    static DB: OnceLock<fontdb::Database> = OnceLock::new();
    DB.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        db
    })
}

/// Every installed font family, sorted, without duplicates.
pub fn families() -> &'static [String] {
    static LIST: OnceLock<Vec<String>> = OnceLock::new();
    LIST.get_or_init(|| {
        let mut names: Vec<String> =
            database().faces().filter_map(|f| f.families.first().map(|(name, _)| name.clone())).collect();
        names.sort_by_key(|n| n.to_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        names
    })
}

/// The face of a family with the wanted weight and slant. A missing family falls back to Segoe UI, Arial, then any font
/// (like WPF's default font).
fn find(family: &str, bold: bool, italic: bool) -> Option<fontdb::ID> {
    type Cache = Mutex<HashMap<(String, bool, bool), Option<fontdb::ID>>>;
    static CACHE: OnceLock<Cache> = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let key = (family.to_lowercase(), bold, italic);
    if let Some(found) = cache.lock().ok()?.get(&key) {
        return *found;
    }
    let db = database();
    let query = |name: &str, weight: u16| {
        db.query(&fontdb::Query {
            families: &[fontdb::Family::Name(name)],
            weight: fontdb::Weight(weight),
            stretch: fontdb::Stretch::Normal,
            style: if italic { fontdb::Style::Italic } else { fontdb::Style::Normal },
        })
    };
    let weight = if bold { 700 } else { 400 };
    let has_family = |name: &str| db.faces().any(|f| f.families.iter().any(|(n, _)| n.eq_ignore_ascii_case(name)));
    // A list like "Segoe Fluent Icons, Segoe MDL2 Assets" (WPF font fallback): the first installed one counts.
    let first = family.split(',').next().unwrap_or(family).trim();
    let family = family.split(',').map(str::trim).find(|n| has_family(n)).unwrap_or(first);
    // Names like "Segoe UI Black" are a weight of "Segoe UI" in the font files: split the weight word off.
    let by_weight_word = || {
        const WORDS: [(&str, u16); 8] = [
            (" extralight", 200),
            (" semilight", 350),
            (" semibold", 600),
            (" extrabold", 800),
            (" black", 900),
            (" heavy", 900),
            (" light", 300),
            (" medium", 500),
        ];
        let lower = family.to_lowercase();
        WORDS.iter().find_map(|(word, w)| {
            let base = lower.strip_suffix(word)?;
            let base = family.get(..base.len())?;
            query(base, if bold { (*w).max(700) } else { *w })
        })
    };
    let id = (if has_family(family) { query(family, weight) } else { by_weight_word() })
        .or_else(|| ["Segoe UI", "Arial", "Tahoma"].iter().find_map(|n| query(n, weight)))
        .or_else(|| db.faces().next().map(|f| f.id));
    cache.lock().ok()?.insert(key, id);
    id
}

/// Runs `f` with the font's face (the font file stays in the system's memory map: nothing is copied).
pub fn with_face<R>(family: &str, bold: bool, italic: bool, f: impl FnOnce(&Face) -> R) -> Option<R> {
    let id = find(family, bold, italic)?;
    database().with_face_data(id, |data, index| Face::from_slice(data, index).map(|face| f(&face)))?
}
