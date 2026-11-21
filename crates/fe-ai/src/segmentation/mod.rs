//! Subject finder (background removal, select subject) and click select (MobileSAM).

mod click;
pub mod mask_ops;
mod subject;

pub use click::{ClickPrompt, ClickSegmenter};
pub use subject::{SubjectSegmenter, content_hash};
