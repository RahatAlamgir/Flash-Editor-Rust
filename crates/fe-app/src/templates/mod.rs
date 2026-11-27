//! Online templates for the Project Manager (C# `Services/Templates`): the list comes from a public GitHub repository, so new
//! templates appear without updating the program. The window is in `dialogs/project_manager/templates.rs`.

pub mod catalog;
pub mod source;

pub use catalog::{Catalog, OnlineTemplate, download};
pub use source::TemplateSource;

/// The source the program uses: the GitHub repository (or `FLASHEDITOR_TEMPLATES`), cached in `settings\online-templates`.
pub fn default_source() -> TemplateSource {
    TemplateSource::new(None, crate::app::paths::settings_dir().join("online-templates"))
}
