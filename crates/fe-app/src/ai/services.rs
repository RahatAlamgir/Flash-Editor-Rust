//! The loaded AI models. A model is opened the first time it is used and kept in memory; `release_all` lets go of all of
//! them (before a model file is deleted, or to give memory back).

use std::sync::Arc;
use std::time::{Duration, Instant};

use fe_ai::inpainting::Inpainter;
use fe_ai::models::{ISNET, LAMA, MOBILE_SAM_DECODER, MOBILE_SAM_ENCODER, ModelStore, REAL_ESRGAN};
use fe_ai::segmentation::{ClickSegmenter, SubjectSegmenter};
use fe_ai::upscaling::AiUpscaler;

#[derive(Default)]
pub struct Services {
    subject: Option<Arc<SubjectSegmenter>>,
    click: Option<Arc<ClickSegmenter>>,
    inpaint: Option<Arc<Inpainter>>,
    upscale: Option<Arc<AiUpscaler>>,
    /// When a model was last asked for.
    last_used: Option<Instant>,
}

/// How long the loaded models stay in memory when no AI tool is used.
pub const IDLE_RELEASE: Duration = Duration::from_secs(300);

impl Services {
    pub fn subject(&mut self, store: &ModelStore) -> Arc<SubjectSegmenter> {
        self.last_used = Some(Instant::now());
        self.subject.get_or_insert_with(|| Arc::new(SubjectSegmenter::new(&ISNET, store.path_of(&ISNET)))).clone()
    }

    pub fn click(&mut self, store: &ModelStore) -> Arc<ClickSegmenter> {
        self.last_used = Some(Instant::now());
        self.click
            .get_or_insert_with(|| {
                Arc::new(ClickSegmenter::new(store.path_of(&MOBILE_SAM_ENCODER), store.path_of(&MOBILE_SAM_DECODER)))
            })
            .clone()
    }

    pub fn inpaint(&mut self, store: &ModelStore) -> Arc<Inpainter> {
        self.last_used = Some(Instant::now());
        self.inpaint.get_or_insert_with(|| Arc::new(Inpainter::new(&LAMA, store.path_of(&LAMA)))).clone()
    }

    pub fn upscale(&mut self, store: &ModelStore) -> Arc<AiUpscaler> {
        self.last_used = Some(Instant::now());
        self.upscale.get_or_insert_with(|| Arc::new(AiUpscaler::new(&REAL_ESRGAN, store.path_of(&REAL_ESRGAN)))).clone()
    }

    /// True when some model is loaded.
    pub fn any_loaded(&self) -> bool {
        self.subject.is_some() || self.click.is_some() || self.inpaint.is_some() || self.upscale.is_some()
    }

    /// Lets go of the models when none was used for `IDLE_RELEASE` and no job holds one. Returns true when it did.
    pub fn release_if_idle(&mut self) -> bool {
        let idle = self.last_used.is_some_and(|t| t.elapsed() >= IDLE_RELEASE);
        let free = |a: Option<usize>| a.is_none_or(|n| n <= 1);
        let unused = free(self.subject.as_ref().map(Arc::strong_count))
            && free(self.click.as_ref().map(Arc::strong_count))
            && free(self.inpaint.as_ref().map(Arc::strong_count))
            && free(self.upscale.as_ref().map(Arc::strong_count));
        if self.any_loaded() && idle && unused {
            self.release_all();
            return true;
        }
        false
    }

    /// Lets go of every loaded model.
    pub fn release_all(&mut self) {
        if let Some(s) = self.subject.take() {
            s.release();
        }
        if let Some(s) = self.click.take() {
            s.release();
        }
        if let Some(s) = self.inpaint.take() {
            s.release();
        }
        if let Some(s) = self.upscale.take() {
            s.release();
        }
    }
}
