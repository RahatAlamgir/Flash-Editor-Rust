//! Flash Editor AI features (ONNX Runtime). Mirrors `FlashEditor.AI` of the C# version. Nothing here loads the AI
//! runtime until a feature is used, and the models are downloaded only after the user agrees.

#![allow(clippy::chunks_exact_to_as_chunks)] // plain 4-byte pixel chunks read best here

pub mod effects;
pub mod inpainting;
pub mod models;
pub mod runtime;
pub mod segmentation;
pub mod upscaling;
