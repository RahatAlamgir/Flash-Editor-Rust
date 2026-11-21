//! The AI features inside the program (C# `EffectCommands.EnsureModel`, `InpaintCommands`, `UpscaleCommands`, the AI
//! Models window...). The AI itself is in the `fe-ai` crate; this folder is what the window does with it: asking before a
//! model is downloaded, running the AI on a background thread with progress and Cancel, and putting the result on the
//! picture. One AI job runs at a time.

pub mod click;
pub mod fill;
pub mod job;
pub mod masks;
pub mod models_dialog;
pub mod outcome;
pub mod pending;
pub mod remove_bg;
pub mod services;
pub mod subject;
pub mod upscale;

use std::sync::Arc;

use fe_ai::models::ModelStore;

pub use job::AiJob;
pub use outcome::AiDone;
pub use pending::Pending;
pub use services::Services;

/// What the program knows about the AI.
pub struct AiState {
    /// The folder with the downloaded models.
    pub store: Arc<ModelStore>,
    /// The loaded models (kept while in use, released when idle or before a model file is deleted).
    pub services: Services,
    /// The AI job that is running, if any.
    pub job: Option<AiJob>,
    /// The window, so tools can start jobs without it being passed around.
    pub ctx: Option<egui::Context>,
}

impl Default for AiState {
    fn default() -> Self {
        Self { store: Arc::new(ModelStore::new(None)), services: Services::default(), job: None, ctx: None }
    }
}
