//! Flash Editor image engine. No UI and no GPU code lives here.
//!
//! Mirrors `FlashEditor.Core` of the C# version. See `docs/plan/01-architecture.md`.

pub mod codes;
pub mod document;
pub mod effects;
pub mod history;
pub mod imaging;
pub mod io;
pub mod layout;
pub mod library;
pub mod objects;
pub mod paint;
pub mod render;
pub mod retouch;
pub mod selection;
pub mod text;
pub mod transform;
pub mod tutorial;
