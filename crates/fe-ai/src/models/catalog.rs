//! Every AI model Flash Editor can use, so the AI Models window can list them (C# `ModelCatalog`).

use super::info::{ISNET, LAMA, MOBILE_SAM_DECODER, MOBILE_SAM_ENCODER, ModelInfo, REAL_ESRGAN};

/// All models, in the order they are shown.
pub fn all() -> [&'static ModelInfo; 5] {
    [&ISNET, &REAL_ESRGAN, &LAMA, &MOBILE_SAM_ENCODER, &MOBILE_SAM_DECODER]
}

/// The name and the features that need the model.
pub struct Description {
    pub title: &'static str,
    pub used_for: &'static str,
}

pub fn describe(model: &ModelInfo) -> Description {
    let (title, used_for) = if model == &ISNET {
        ("Background removal and subject selection", "Image > Remove Background (AI), Select > Select Subject (AI)")
    } else if model == &REAL_ESRGAN {
        ("Upscaling and clean-up", "Image > AI Upscale / Clean Up")
    } else if model == &LAMA {
        ("Object removal (fill)", "Select > Fill (AI), Remove Brush")
    } else if model == &MOBILE_SAM_ENCODER {
        ("Click to select (part 1 of 2)", "Click Select tool: studies the picture")
    } else if model == &MOBILE_SAM_DECODER {
        ("Click to select (part 2 of 2)", "Click Select tool: outlines the object you click")
    } else {
        (model.display_name, "")
    };
    Description { title, used_for }
}
