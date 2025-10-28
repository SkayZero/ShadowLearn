use thiserror::Error;

/// Erreurs liées à la détection d'applications
#[derive(Debug, Error)]
pub enum AppDetectionError {
    #[error("No active window found")]
    NoActiveWindow,
}

/// Erreurs liées au clipboard
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("Failed to access clipboard: {0}")]
    AccessError(String),
}

/// Erreurs liées à l'agrégation de contexte
#[derive(Debug, Error)]
pub enum ContextError {
    #[error("App detection failed: {0}")]
    AppDetection(#[from] AppDetectionError),

    #[error("Clipboard monitoring failed: {0}")]
    Clipboard(#[from] ClipboardError),
}

/// Status des permissions TCC (macOS)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TCCStatus {
    Granted,
    Denied,
    Unknown,
}

impl std::fmt::Display for TCCStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TCCStatus::Granted => write!(f, "granted"),
            TCCStatus::Denied => write!(f, "denied"),
            TCCStatus::Unknown => write!(f, "unknown"),
        }
    }
}
