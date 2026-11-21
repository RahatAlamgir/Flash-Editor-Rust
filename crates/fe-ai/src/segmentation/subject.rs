//! Runs the background-removal model: given a picture, how much each pixel belongs to the main subject (0 = background,
//! 1 = subject) (C# `SubjectSegmenter`). The model is loaded on first use and kept in memory; the last result is cached,
//! so moving a slider does not run the model again.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use fe_core::imaging::PixelBuffer;
use ort::value::Tensor;

use super::mask_ops;
use crate::models::ModelInfo;
use crate::runtime::{self, Opened};

/// A fast fingerprint of the pixels (to know whether a cached result still fits the picture).
pub fn content_hash(image: &PixelBuffer) -> u64 {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    let mut words = image.data().chunks_exact(8);
    for w in words.by_ref() {
        hash = (hash ^ u64::from_le_bytes(w.try_into().unwrap_or([0; 8]))).wrapping_mul(1_099_511_628_211);
    }
    for b in words.remainder() {
        hash = (hash ^ u64::from(*b)).wrapping_mul(1_099_511_628_211);
    }
    hash
}

#[derive(Default)]
struct State {
    session: Option<Opened>,
    key: Option<(u32, u32, u64)>,
    mask: Option<Vec<f32>>,
}

pub struct SubjectSegmenter {
    model: &'static ModelInfo,
    path: PathBuf,
    state: Mutex<State>,
}

impl SubjectSegmenter {
    pub fn new(model: &'static ModelInfo, path: PathBuf) -> Self {
        Self { model, path, state: Mutex::new(State::default()) }
    }

    /// What the model runs on ("GPU (DirectML)" or "CPU"; empty until first used).
    pub fn device_name(&self) -> String {
        self.state.lock().ok().and_then(|s| s.session.as_ref().map(|o| o.device.to_owned())).unwrap_or_default()
    }

    /// The subject mask at the picture's own size. Can take a few seconds the first time.
    pub fn predict_mask(&self, image: &PixelBuffer, cancel: &AtomicBool) -> Result<Vec<f32>, String> {
        let key = (image.width(), image.height(), content_hash(image));
        let mut state = self.state.lock().map_err(|_| "The AI model is busy.".to_owned())?; // one run at a time
        if state.key == Some(key)
            && let Some(m) = &state.mask
        {
            return Ok(m.clone());
        }
        let size = self.model.input_size;
        let input = mask_ops::to_input_tensor(image, size, self.model.mean, self.model.std);
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled.".into());
        }
        if state.session.is_none() {
            state.session = Some(runtime::open(&self.path, true)?);
        }
        let opened = state.session.as_mut().ok_or("The AI model is not loaded.")?;
        let tensor = Tensor::from_array(([1usize, 3, size, size], input)).map_err(|e| e.to_string())?;
        let name = opened.session.inputs[0].name.clone();
        let outputs =
            opened.session.run(ort::inputs![name => tensor]).map_err(|e| format!("The AI model failed: {e}"))?;
        let (_, raw) = outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        let raw = raw.to_vec();
        drop(outputs);
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled.".into());
        }
        let side = (raw.len() as f64).sqrt().round() as usize; // the output is side x side (usually the input size)
        let mask = mask_ops::resize_mask(
            &mask_ops::normalize(&raw),
            side,
            side,
            image.width() as usize,
            image.height() as usize,
        );
        state.key = Some(key);
        state.mask = Some(mask.clone());
        Ok(mask)
    }

    /// Drops the model from memory (it is loaded again when needed).
    pub fn release(&self) {
        if let Ok(mut s) = self.state.lock() {
            *s = State::default();
        }
    }
}
