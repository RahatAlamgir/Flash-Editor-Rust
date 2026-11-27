//! The list of online templates (C# `TemplateCatalog`, `OnlineTemplate`, `TemplateType`): reads `catalog.json` and
//! `types.json`, finds templates by words, and checks a downloaded template against the catalog (size and SHA-256). When the
//! internet cannot be reached, the copy saved last time is used.

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::source::TemplateSource;

/// A project type (a category button) such as "Poster" with its usual size.
#[derive(Clone, Debug, PartialEq)]
pub struct TemplateType {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
}

/// One ready-made template of the catalog, as shown on a card.
#[derive(Clone, Debug, PartialEq)]
pub struct OnlineTemplate {
    pub id: String,
    pub kind: String,
    pub type_name: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub pages: u32,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub sha256: String,
    /// "2026-11-23" style text; newest first.
    pub added: String,
}

impl OnlineTemplate {
    /// Where the project file is, relative to the catalog (for example "templates/poster/mega-sale/template.flash").
    pub fn file_path(&self) -> String {
        format!("templates/{}/{}/template.flash", self.kind, self.id)
    }

    pub fn thumbnail_path(&self) -> String {
        format!("templates/{}/{}/thumb.jpg", self.kind, self.id)
    }

    /// True when the word is in the name, the type or a keyword.
    pub fn matches(&self, word: &str) -> bool {
        let w = word.to_lowercase();
        self.name.to_lowercase().contains(&w)
            || self.type_name.to_lowercase().contains(&w)
            || self.kind.to_lowercase().contains(&w)
            || self.keywords.iter().any(|k| k.to_lowercase().contains(&w))
    }
}

/// A category button: "All" (no id), then every type of the catalog.
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub id: Option<String>,
    pub name: String,
    pub count: usize,
}

#[derive(Clone, Debug, Default)]
pub struct Catalog {
    pub types: Vec<TemplateType>,
    pub templates: Vec<OnlineTemplate>,
    /// True when the list came from the saved copy because the source could not be reached.
    pub offline: bool,
}

fn text(v: &Value, key: &str) -> Option<String> {
    v.get(key)?.as_str().map(str::to_owned)
}

fn number(v: &Value, key: &str) -> Option<u64> {
    v.get(key)?.as_u64()
}

fn parse_types(json: &[u8]) -> Vec<TemplateType> {
    let Ok(v) = serde_json::from_slice::<Value>(json) else { return Vec::new() };
    let Some(list) = v.get("types").and_then(Value::as_array) else { return Vec::new() };
    list.iter()
        .filter_map(|t| {
            Some(TemplateType {
                id: text(t, "id")?,
                name: text(t, "name")?,
                width: number(t, "width")? as u32,
                height: number(t, "height")? as u32,
            })
        })
        .collect()
}

fn parse_templates(json: &[u8], types: &[TemplateType]) -> Result<Vec<OnlineTemplate>, String> {
    let v: Value = serde_json::from_slice(json).map_err(|e| e.to_string())?;
    let list = v.get("templates").and_then(Value::as_array).ok_or("The template list is damaged.")?;
    let mut out: Vec<OnlineTemplate> = list
        .iter()
        .filter_map(|t| {
            let kind = text(t, "type")?;
            let type_name = types.iter().find(|x| x.id == kind).map_or_else(|| kind.clone(), |x| x.name.clone());
            let keywords = t
                .get("keywords")?
                .as_array()?
                .iter()
                .filter_map(|k| k.as_str().filter(|s| !s.is_empty()).map(str::to_owned))
                .collect();
            Some(OnlineTemplate {
                id: text(t, "id")?,
                kind,
                type_name,
                name: text(t, "name")?,
                keywords,
                pages: number(t, "pages")? as u32,
                width: number(t, "width")? as u32,
                height: number(t, "height")? as u32,
                bytes: number(t, "bytes")?,
                sha256: text(t, "sha256")?,
                added: text(t, "added").unwrap_or_default(),
            })
        })
        .collect();
    // Newest first, then A to Z.
    out.sort_by(|a, b| b.added.cmp(&a.added).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    Ok(out)
}

impl Catalog {
    /// Loads the catalog. `Err` when there is no connection and nothing was saved before.
    pub fn load(source: &TemplateSource) -> Result<Self, String> {
        let (catalog, types, offline) = match (source.fetch("catalog.json"), source.fetch("types.json")) {
            (Ok(c), Ok(t)) => (c, t, false),
            _ => {
                let c = source.cached("catalog.json").ok_or("The templates could not be loaded.")?;
                (c, source.cached("types.json").unwrap_or_default(), true)
            }
        };
        let types = parse_types(&types);
        let templates = parse_templates(&catalog, &types)?;
        Ok(Self { types, templates, offline })
    }

    /// The category buttons: "All", then every type of the catalog (also the ones without templates yet).
    pub fn categories(&self) -> Vec<Category> {
        let mut list = vec![Category { id: None, name: "All".into(), count: self.templates.len() }];
        for t in &self.types {
            list.push(Category {
                id: Some(t.id.clone()),
                name: t.name.clone(),
                count: self.templates.iter().filter(|x| x.kind == t.id).count(),
            });
        }
        // A type the list does not know.
        let mut unknown: Vec<&str> =
            self.templates.iter().map(|t| t.kind.as_str()).filter(|k| !self.types.iter().any(|t| t.id == *k)).collect();
        unknown.sort_unstable();
        unknown.dedup();
        for id in unknown {
            list.push(Category {
                id: Some(id.to_owned()),
                name: id.to_owned(),
                count: self.templates.iter().filter(|x| x.kind == id).count(),
            });
        }
        list
    }

    /// The templates of one category (`None` = all) that match every word typed in `text`.
    pub fn find(&self, kind: Option<&str>, text: &str) -> Vec<&OnlineTemplate> {
        let words: Vec<&str> = text.split_whitespace().collect();
        self.templates
            .iter()
            .filter(|t| kind.is_none_or(|k| t.kind == k) && words.iter().all(|w| t.matches(w)))
            .collect()
    }
}

fn matches_catalog(data: &[u8], t: &OnlineTemplate) -> bool {
    let hash: String = Sha256::digest(data).iter().map(|b| format!("{b:02x}")).collect();
    data.len() as u64 == t.bytes && hash.eq_ignore_ascii_case(&t.sha256)
}

/// Gets the template's project file and checks it against the catalog (size and SHA-256), so a damaged or swapped file is
/// never opened.
pub fn download(source: &TemplateSource, t: &OnlineTemplate) -> Result<Vec<u8>, String> {
    if let Some(saved) = source.cached(&t.file_path()).filter(|d| matches_catalog(d, t)) {
        return Ok(saved);
    }
    let data = source.fetch(&t.file_path()).map_err(|_| {
        format!("\"{}\" could not be downloaded. Check your internet connection and try again.", t.name)
    })?;
    if matches_catalog(&data, t) {
        Ok(data)
    } else {
        Err(format!("The file of \"{}\" is damaged. Try again later.", t.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_source(tag: &str) -> (TemplateSource, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("fe-templates-{tag}-{}", std::process::id()));
        let repo = root.join("repo");
        let file = repo.join("templates").join("poster").join("mega-sale");
        std::fs::create_dir_all(&file).expect("folder");
        let flash = b"fake project bytes".to_vec();
        std::fs::write(file.join("template.flash"), &flash).expect("file");
        let hash: String = Sha256::digest(&flash).iter().map(|b| format!("{b:02x}")).collect();
        std::fs::write(repo.join("types.json"), r#"{"types":[{"id":"poster","name":"Poster","width":1080,"height":1350},{"id":"card","name":"Card","width":1050,"height":600}]}"#).expect("types");
        std::fs::write(
            repo.join("catalog.json"),
            format!(
                r#"{{"templates":[
                {{"id":"mega-sale","type":"poster","name":"Mega Sale","keywords":["sale","business"],"pages":1,"width":1080,"height":1350,"bytes":{},"sha256":"{hash}","added":"2026-11-20"}},
                {{"id":"party","type":"poster","name":"Party Night","keywords":["party"],"pages":2,"width":1080,"height":1350,"bytes":5,"sha256":"00","added":"2026-11-22"}},
                {{"id":"old","type":"other","name":"Old One","keywords":[],"pages":1,"width":10,"height":10,"bytes":1,"sha256":"00","added":"2025-01-01"}},
                {{"broken": true}}]}}"#,
                flash.len()
            ),
        )
        .expect("catalog");
        (TemplateSource::new(Some(repo.to_string_lossy().into_owned()), root.join("cache")), root)
    }

    #[test]
    fn the_catalog_is_read_sorted_newest_first_and_broken_entries_are_skipped() {
        let (source, root) = sample_source("read");
        let catalog = Catalog::load(&source).expect("loads");
        let names: Vec<&str> = catalog.templates.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Party Night", "Mega Sale", "Old One"]);
        assert_eq!(catalog.templates[0].type_name, "Poster");
        assert_eq!(catalog.templates[2].type_name, "other", "a type the list does not know keeps its id");
        let cats: Vec<(String, usize)> = catalog.categories().into_iter().map(|c| (c.name, c.count)).collect();
        assert_eq!(cats, [("All".into(), 3), ("Poster".into(), 2), ("Card".into(), 0), ("other".into(), 1)]);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn words_must_all_match_the_name_the_type_or_a_keyword() {
        let (source, root) = sample_source("find");
        let catalog = Catalog::load(&source).expect("loads");
        assert_eq!(catalog.find(None, "sale").len(), 1);
        assert_eq!(catalog.find(None, "poster business").len(), 1);
        assert_eq!(catalog.find(Some("poster"), "").len(), 2);
        assert_eq!(catalog.find(Some("card"), "").len(), 0);
        assert!(catalog.find(None, "sale party").is_empty());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn a_download_is_checked_and_the_saved_copy_works_without_the_source() {
        let (source, root) = sample_source("download");
        let catalog = Catalog::load(&source).expect("loads");
        let mega = catalog.templates.iter().find(|t| t.id == "mega-sale").expect("template").clone();
        assert_eq!(download(&source, &mega).expect("checked"), b"fake project bytes");
        let party = catalog.templates.iter().find(|t| t.id == "party").expect("template");
        assert!(download(&source, party).is_err(), "a file that is not there");
        // The source goes away: the catalog and the template still open from the saved copies.
        let offline = TemplateSource::new(Some(root.join("gone").to_string_lossy().into_owned()), root.join("cache"));
        let again = Catalog::load(&offline).expect("from the saved copy");
        assert!(again.offline && again.templates.len() == 3);
        assert_eq!(download(&offline, &mega).expect("saved"), b"fake project bytes");
        // A swapped file is refused.
        std::fs::write(
            root.join("cache").join("templates").join("poster").join("mega-sale").join("template.flash"),
            b"swapped",
        )
        .expect("write");
        let bad = TemplateSource::new(Some(root.join("gone").to_string_lossy().into_owned()), root.join("cache"));
        assert!(download(&bad, &mega).is_err());
        let _ = std::fs::remove_dir_all(root);
    }
}
