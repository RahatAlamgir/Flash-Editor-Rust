//! Which models exist, where they come from; download once and verify (SHA-256).

mod catalog;
mod info;
mod store;

pub use catalog::{Description, all, describe};
pub use info::{ISNET, LAMA, MOBILE_SAM_DECODER, MOBILE_SAM_ENCODER, ModelInfo, REAL_ESRGAN};
pub use store::{ModelStore, default_folder};
