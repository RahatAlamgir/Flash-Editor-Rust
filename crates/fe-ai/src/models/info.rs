//! Describes an AI model: where to download it, how to check it, and how the model wants its input (C# `ModelInfo`).
//! Most models are one .onnx file. Some come as a zip holding the .onnx file plus a separate weights file; then
//! `size_bytes` / `sha256` describe the zip, and the listed files are unpacked into a folder named `folder`.

#[derive(Debug, PartialEq)]
pub struct ModelInfo {
    pub display_name: &'static str,
    pub file_name: &'static str,
    pub download_url: &'static str,
    pub size_bytes: u64,
    pub sha256: &'static str,
    pub input_size: usize,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    /// Files inside the zip and their sizes (empty = the download is the model file itself).
    pub archive_files: &'static [(&'static str, u64)],
    pub folder: Option<&'static str>,
}

impl ModelInfo {
    pub fn size_text(&self) -> String {
        format!("{:.0} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
    }
}

/// IS-Net "general use" (DIS, Apache 2.0), as published by the rembg project. Finds the main subject of a photo.
pub static ISNET: ModelInfo = ModelInfo {
    display_name: "IS-Net background removal model",
    file_name: "isnet-general-use.onnx",
    download_url: "https://github.com/danielgatis/rembg/releases/download/v0.0.0/isnet-general-use.onnx",
    size_bytes: 178_648_008,
    sha256: "60920e99c45464f2ba57bee2ad08c919a52bbf852739e96947fbb4358c0d964a",
    input_size: 1024,
    mean: [0.5, 0.5, 0.5],
    std: [1.0, 1.0, 1.0],
    archive_files: &[],
    folder: None,
};

/// Real-ESRGAN x4plus (BSD-3) as converted to ONNX by Qualcomm AI Hub: 4 times larger and sharper, on 128 x 128 tiles.
pub static REAL_ESRGAN: ModelInfo = ModelInfo {
    display_name: "Real-ESRGAN x4plus upscaling model",
    file_name: "real_esrgan_x4plus.onnx",
    download_url: "https://qaihub-public-assets.s3.us-west-2.amazonaws.com/qai-hub-models/models/real_esrgan_x4plus/releases/v0.66.0/real_esrgan_x4plus-onnx-float.zip",
    size_bytes: 62_174_027,
    sha256: "05aa31323c6a7e831493171cac970e398697709cae7e7ae8e0760e6da4d64ca4",
    input_size: 128,
    mean: [0.0; 3],
    std: [1.0; 3],
    archive_files: &[("real_esrgan_x4plus.onnx", 3_158_313), ("real_esrgan_x4plus.data", 66_737_664)],
    folder: Some("real_esrgan_x4plus"),
};

/// LaMa "big-lama" (Apache 2.0) as converted to ONNX by Carve: fills a hidden area with what would plausibly be there.
pub static LAMA: ModelInfo = ModelInfo {
    display_name: "LaMa object removal model",
    file_name: "lama_fp32.onnx",
    download_url: "https://huggingface.co/Carve/LaMa-ONNX/resolve/main/lama_fp32.onnx",
    size_bytes: 208_044_816,
    sha256: "1faef5301d78db7dda502fe59966957ec4b79dd64e16f03ed96913c7a4eb68d6",
    input_size: 512,
    mean: [0.0; 3],
    std: [1.0; 3],
    archive_files: &[],
    folder: None,
};

/// MobileSAM (Apache 2.0) image encoder as converted to ONNX by Acly: studies a picture so a click can select an object.
pub static MOBILE_SAM_ENCODER: ModelInfo = ModelInfo {
    display_name: "MobileSAM click-to-select model (picture part)",
    file_name: "mobile_sam_image_encoder.onnx",
    download_url: "https://huggingface.co/Acly/MobileSAM/resolve/main/mobile_sam_image_encoder.onnx",
    size_bytes: 28_157_093,
    sha256: "580f5fb648ea1062c0aabc26217aed56921985f03f0cbbd852bba81d760cc749",
    input_size: 1024,
    mean: [0.0; 3],
    std: [1.0; 3],
    archive_files: &[],
    folder: None,
};

/// MobileSAM mask decoder (the small second part): turns a click or box into the outline of the object.
pub static MOBILE_SAM_DECODER: ModelInfo = ModelInfo {
    display_name: "MobileSAM click-to-select model (outline part)",
    file_name: "sam_mask_decoder_single.onnx",
    download_url: "https://huggingface.co/Acly/MobileSAM/resolve/main/sam_mask_decoder_single.onnx",
    size_bytes: 16_501_323,
    sha256: "93915fc7c993ab9d59ab8c9ccd3bce37f7509c81ab4150a74abd4d2abbd8570d",
    input_size: 256,
    mean: [0.0; 3],
    std: [1.0; 3],
    archive_files: &[],
    folder: None,
};
