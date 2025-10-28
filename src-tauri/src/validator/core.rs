use serde::{Deserialize, Serialize};
use std::path::Path;

/// Core validation functionality
pub trait ValidatorCore {
    fn validate(&self, path: &Path) -> ValidationResult;
}

/// Validation result with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub status: ValidationStatus,
    pub message: Option<String>,
    pub duration_ms: u64,
    pub validator_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Invalid,
    Error,
    Skipped,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            status: ValidationStatus::Valid,
            message: None,
            duration_ms: 0,
            validator_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn invalid(message: String) -> Self {
        Self {
            status: ValidationStatus::Invalid,
            message: Some(message),
            duration_ms: 0,
            validator_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            status: ValidationStatus::Error,
            message: Some(message),
            duration_ms: 0,
            validator_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn skipped(reason: String) -> Self {
        Self {
            status: ValidationStatus::Skipped,
            message: Some(reason),
            duration_ms: 0,
            validator_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self.status, ValidationStatus::Valid)
    }

    pub fn should_learn(&self) -> bool {
        matches!(
            self.status,
            ValidationStatus::Valid | ValidationStatus::Skipped
        )
    }
}
