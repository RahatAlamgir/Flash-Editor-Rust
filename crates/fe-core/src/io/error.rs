//! Errors from reading and writing files, with messages meant for the user.

/// Something went wrong reading or writing a file.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    /// The file could not be read or written (missing, no permission, disk full...).
    #[error("{0}")]
    File(#[from] std::io::Error),
    /// The bytes are not a picture we can read; the text says what to do.
    #[error("{0}")]
    CannotOpen(String),
    /// The picture is bigger than the editor allows.
    #[error("The image is too large ({width}x{height}). Maximum is {max}px per side.")]
    TooLarge { width: u32, height: u32, max: u32 },
    /// Encoding failed or the format cannot be saved yet.
    #[error("{0}")]
    CannotSave(String),
}
