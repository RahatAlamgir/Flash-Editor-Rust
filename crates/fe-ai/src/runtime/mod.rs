//! Opens AI models on the best device available: the graphics card through DirectML (NVIDIA, AMD and Intel GPUs on
//! Windows 10 / 11), or the processor when there is no usable GPU or the GPU fails to load the model (C#
//! `OnnxSessionFactory`). The ONNX Runtime DLL is loaded only when the first model is opened.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ort::execution_providers::DirectMLExecutionProvider;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;

/// A session and the name of what it runs on ("GPU (DirectML)" or "CPU").
pub struct Opened {
    pub session: Session,
    pub device: &'static str,
}

/// Where `onnxruntime.dll` is looked for: next to the program, then in a "runtime" folder there.
fn dll_candidates() -> Vec<PathBuf> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)).unwrap_or_default();
    vec![exe_dir.join("onnxruntime.dll"), exe_dir.join("runtime").join("onnxruntime.dll")]
}

/// The path of the runtime DLL, if it is there.
pub fn runtime_dll() -> Option<PathBuf> {
    std::env::var_os("FLASH_ORT_DLL")
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .or_else(|| dll_candidates().into_iter().find(|p| p.exists()))
}

/// Loads the runtime (once). The error says in plain words what is missing.
pub fn ensure_loaded() -> Result<(), String> {
    static LOADED: OnceLock<Result<(), String>> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let Some(dll) = runtime_dll() else {
                return Err("The AI runtime (onnxruntime.dll) was not found next to Flash Editor. Reinstall the program or copy the file there.".into());
            };
            ort::init_from(dll.to_string_lossy().to_string())
                .commit()
                .map(|_| ())
                .map_err(|e| format!("The AI runtime could not be started: {e}"))
        })
        .clone()
}

fn prefer_gpu() -> bool {
    std::env::var_os("FLASH_AI_CPU").is_none()
}

/// Opens the model. `allow_gpu` is false for models that do not run correctly on the graphics card.
pub fn open(model_path: &Path, allow_gpu: bool) -> Result<Opened, String> {
    ensure_loaded()?;
    if allow_gpu && prefer_gpu() {
        let gpu = Session::builder()
            .and_then(|b| b.with_optimization_level(GraphOptimizationLevel::Level3))
            .and_then(|b| b.with_memory_pattern(false)) // required by DirectML
            .and_then(|b| b.with_parallel_execution(false)) // required by DirectML
            .and_then(|b| b.with_execution_providers([DirectMLExecutionProvider::default().build().error_on_failure()]))
            .and_then(|b| b.commit_from_file(model_path));
        if let Ok(session) = gpu {
            return Ok(Opened { session, device: "GPU (DirectML)" });
        }
        // No DirectX 12 GPU, an old driver, or the model does not run on it: use the processor.
    }
    let session = Session::builder()
        .and_then(|b| b.with_optimization_level(GraphOptimizationLevel::Level3))
        .and_then(|b| b.commit_from_file(model_path))
        .map_err(|e| format!("The AI model could not be opened: {e}"))?;
    Ok(Opened { session, device: "CPU" })
}
