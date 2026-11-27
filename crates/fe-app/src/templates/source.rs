//! Where the template files are and how to get them (C# `TemplateSource`): from the web (GitHub) or, for testing, from a
//! folder on this PC. Everything that was downloaded is kept in a cache folder, so templates already seen open at once and
//! the browser still shows them without internet.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// The public GitHub repository that holds the templates.
pub const DEFAULT_LOCATION: &str = "https://raw.githubusercontent.com/RahatAlamgir/Flash-Editor-Templates/master/";

#[derive(Clone, Debug)]
pub struct TemplateSource {
    location: String,
    cache: PathBuf,
}

impl TemplateSource {
    /// `location`: a web address or a folder (default: the GitHub repository, or the `FLASHEDITOR_TEMPLATES` environment
    /// variable, for testing). `cache`: where downloads are kept.
    pub fn new(location: Option<String>, cache: PathBuf) -> Self {
        let location = location
            .or_else(|| std::env::var("FLASHEDITOR_TEMPLATES").ok())
            .filter(|l| !l.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_LOCATION.to_owned());
        Self { location, cache }
    }

    fn is_folder(&self) -> bool {
        Path::new(&self.location).is_dir()
    }

    fn cache_path(&self, relative: &str) -> PathBuf {
        relative.split('/').fold(self.cache.clone(), |p, part| p.join(part))
    }

    /// The file, fresh from the source. The copy is kept in the cache. `Err` when the source cannot be reached.
    pub fn fetch(&self, relative: &str) -> Result<Vec<u8>, String> {
        let data = if self.is_folder() {
            let path = relative.split('/').fold(PathBuf::from(&self.location), |p, part| p.join(part));
            std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
        } else {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(20)))
                .user_agent("FlashEditor")
                .build()
                .into();
            let base = if self.location.ends_with('/') { self.location.clone() } else { format!("{}/", self.location) };
            let mut response = agent.get(format!("{base}{relative}")).call().map_err(|e| e.to_string())?;
            response.body_mut().with_config().limit(200 * 1024 * 1024).read_to_vec().map_err(|e| e.to_string())?
        };
        self.save(relative, &data);
        Ok(data)
    }

    /// The cached copy of a file, or `None` when it was never downloaded.
    pub fn cached(&self, relative: &str) -> Option<Vec<u8>> {
        std::fs::read(self.cache_path(relative)).ok()
    }

    /// The cached copy when there is one, otherwise a download (for pictures that never change).
    pub fn cached_or_fetch(&self, relative: &str) -> Result<Vec<u8>, String> {
        match self.cached(relative) {
            Some(data) => Ok(data),
            None => self.fetch(relative),
        }
    }

    fn save(&self, relative: &str, data: &[u8]) {
        let path = self.cache_path(relative);
        if let Some(folder) = path.parent() {
            // The cache is only a speed-up: a failure here is not a problem.
            let _ = std::fs::create_dir_all(folder).and_then(|()| std::fs::write(&path, data));
        }
    }
}
