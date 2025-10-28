use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScreenshotError {
    #[error("Failed to initialize screen capture: {0}")]
    InitFailed(String),

    #[error("No screens detected")]
    NoScreens,

    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    #[error("Image processing failed: {0}")]
    ProcessingFailed(String),

    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Image encoding error: {0}")]
    ImageError(#[from] image::ImageError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Unknown,
}

impl std::fmt::Display for PermissionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionStatus::Granted => write!(f, "granted"),
            PermissionStatus::Denied => write!(f, "denied"),
            PermissionStatus::Unknown => write!(f, "unknown"),
        }
    }
}
