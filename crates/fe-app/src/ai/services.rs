//! The loaded AI models. A model is opened the first time it is used and kept in memory; `release_all` lets go of all of
//! them (before a model file is deleted, or to give memory back).

use std::sync::Arc;

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
}

impl Services {
    pub fn subject(&mut self, store: &ModelStore) -> Arc<SubjectSegmenter> {
        self.subject.get_or_insert_with(|| Arc::new(SubjectSegmenter::new(&ISNET, store.path_of(&ISNET)))).clone()
    }

    pub fn click(&mut self, store: &ModelStore) -> Arc<ClickSegmenter> {
        self.click
            .get_or_insert_with(|| {
                Arc::new(ClickSegmenter::new(store.path_of(&MOBILE_SAM_ENCODER), store.path_of(&MOBILE_SAM_DECODER)))
            })
            .clone()
    }

    pub fn inpaint(&mut self, store: &ModelStore) -> Arc<Inpainter> {
        self.inpaint.get_or_insert_with(|| Arc::new(Inpainter::new(&LAMA, store.path_of(&LAMA)))).clone()
    }

    pub fn upscale(&mut self, store: &ModelStore) -> Arc<AiUpscaler> {
        self.upscale.get_or_insert_with(|| Arc::new(AiUpscaler::new(&REAL_ESRGAN, store.path_of(&REAL_ESRGAN)))).clone()
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
