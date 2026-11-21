//! QR code and barcode encoders (written from the standards, no library; ports of the C# `QrEncoder` and
//! `BarcodeEncoder`).

pub mod barcode;
mod code128;
mod code39;
pub mod qr;
mod qr_grid;
mod qr_tables;

pub use barcode::{BarcodeData, BarcodeKind};
pub use qr::{QrLevel, encode as encode_qr, max_bytes};
