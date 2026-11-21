//! Click to select (MobileSAM) (C# `ClickSegmenter`): the picture is studied once (the slow part, a second or two), then
//! every click or box gives an outline of the object under it almost at once. The study result is kept.

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use fe_core::imaging::PixelBuffer;
use fe_core::transform::{ResampleMode, resize};
use ort::value::Tensor;

use super::subject::content_hash;
use crate::runtime::{self, Opened};

const ENCODER_SIZE: usize = 1024;

/// What the user told the AI: clicks on the object (include), beside it (exclude), or a box around it.
#[derive(Clone, Debug, Default)]
pub struct ClickPrompt {
    /// (x, y, include) in picture pixels.
    pub points: Vec<(f64, f64, bool)>,
    /// (x1, y1, x2, y2).
    pub boxed: Option<(f64, f64, f64, f64)>,
}

#[derive(Default)]
struct State {
    encoder: Option<Opened>,
    decoder: Option<Opened>,
    key: Option<(u32, u32, u64)>,
    embeddings: Option<Vec<f32>>,
}

pub struct ClickSegmenter {
    encoder_path: PathBuf,
    decoder_path: PathBuf,
    state: Mutex<State>,
}

impl ClickSegmenter {
    pub fn new(encoder_path: PathBuf, decoder_path: PathBuf) -> Self {
        Self { encoder_path, decoder_path, state: Mutex::new(State::default()) }
    }

    /// The decoder's point list (in the 1024 pixel space the model works in) and labels: 1 = on the object, 0 = not the
    /// object, 2 / 3 = top-left / bottom-right of a box. Without a box the model wants one extra "padding" point (-1).
    pub fn build_prompt(prompt: &ClickPrompt, width: usize, height: usize) -> (Vec<f32>, Vec<f32>) {
        let scale = ENCODER_SIZE as f64 / width.max(height) as f64;
        let (mut coords, mut labels) = (Vec::new(), Vec::new());
        let mut add = |x: f64, y: f64, label: f32| {
            coords.push((x * scale) as f32);
            coords.push((y * scale) as f32);
            labels.push(label);
        };
        for (x, y, include) in &prompt.points {
            add(*x, *y, if *include { 1.0 } else { 0.0 });
        }
        if let Some((x1, y1, x2, y2)) = prompt.boxed {
            add(x1.min(x2), y1.min(y2), 2.0);
            add(x1.max(x2), y1.max(y2), 3.0);
        } else {
            add(0.0, 0.0, -1.0);
        }
        (coords, labels)
    }

    /// The selected area as 0 / 255 per pixel at the picture's size (all zero when the AI found nothing).
    pub fn predict(&self, image: &PixelBuffer, prompt: &ClickPrompt, cancel: &AtomicBool) -> Result<Vec<u8>, String> {
        let (w, h) = (image.width() as usize, image.height() as usize);
        let mut state = self.state.lock().map_err(|_| "The AI model is busy.".to_owned())?;
        let key = (image.width(), image.height(), content_hash(image));
        if state.key != Some(key) || state.embeddings.is_none() {
            // Studies the picture (the result is kept for the next click).
            let scale = ENCODER_SIZE as f64 / w.max(h) as f64;
            let (sw, sh) = (((w as f64 * scale).round() as usize).max(1), ((h as f64 * scale).round() as usize).max(1));
            let small = if sw == w && sh == h {
                image.clone()
            } else {
                resize(image, sw as u32, sh as u32, ResampleMode::Smooth)
            };
            // height x width x 3, RGB, 0..255 (the model normalizes and pads itself)
            let input: Vec<f32> = small
                .data()
                .chunks_exact(4)
                .flat_map(|p| [f32::from(p[0]), f32::from(p[1]), f32::from(p[2])])
                .collect();
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled.".into());
            }
            if state.encoder.is_none() {
                // On the graphics card this model gives wrong outlines: the processor is used.
                state.encoder = Some(runtime::open(&self.encoder_path, false)?);
            }
            let enc = state.encoder.as_mut().ok_or("The AI model is not loaded.")?;
            let tensor = Tensor::from_array(([sh, sw, 3usize], input)).map_err(|e| e.to_string())?;
            let outputs = enc
                .session
                .run(ort::inputs!["input_image" => tensor])
                .map_err(|e| format!("The AI model failed: {e}"))?;
            let (_, data) = outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
            let data = data.to_vec();
            drop(outputs);
            state.key = Some(key);
            state.embeddings = Some(data);
        }
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled.".into());
        }
        let (coords, labels) = Self::build_prompt(prompt, w, h);
        let n = labels.len();
        if state.decoder.is_none() {
            state.decoder = Some(runtime::open(&self.decoder_path, false)?);
        }
        let embeddings = state.embeddings.clone().ok_or("No picture was studied.")?;
        let dec = state.decoder.as_mut().ok_or("The AI model is not loaded.")?;
        let t = |shape: Vec<usize>, data: Vec<f32>| Tensor::from_array((shape, data)).map_err(|e| e.to_string());
        let outputs = dec
            .session
            .run(ort::inputs![
                "image_embeddings" => t(vec![1, 256, 64, 64], embeddings)?,
                "point_coords" => t(vec![1, n, 2], coords)?,
                "point_labels" => t(vec![1, n], labels)?,
                "mask_input" => t(vec![1, 1, 256, 256], vec![0.0; 256 * 256])?,
                "has_mask_input" => t(vec![1], vec![0.0])?,
                "orig_im_size" => t(vec![2], vec![h as f32, w as f32])?
            ])
            .map_err(|e| format!("The AI model failed: {e}"))?;
        let (_, logits) = outputs["masks"].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        let mut mask = vec![0u8; w * h];
        if logits.len() >= mask.len() {
            for (m, l) in mask.iter_mut().zip(logits) {
                *m = if *l > 0.0 { 255 } else { 0 };
            }
        }
        Ok(mask)
    }

    pub fn release(&self) {
        if let Ok(mut s) = self.state.lock() {
            *s = State::default();
        }
    }
}
