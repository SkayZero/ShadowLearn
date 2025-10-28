use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

pub mod core;
pub mod stats;

use crate::validator::stats::ValidationStats;

/// Main validator for artefacts before learning
#[derive(Debug)]
pub struct ArtefactValidator {
    blender_path: Option<PathBuf>,
    python_path: Option<PathBuf>,
    timeout_duration: Duration,
    cache: HashMap<String, CachedValidation>,
    stats: ValidationStats,
}

#[derive(Debug, Clone)]
struct CachedValidation {
    result: ValidationResult,
    timestamp: Instant,
    file_hash: String,
}

impl ArtefactValidator {
    pub fn new() -> Self {
        Self {
            blender_path: Self::find_blender(),
            python_path: Self::find_python(),
            timeout_duration: Duration::from_secs(5),
            cache: HashMap::new(),
            stats: ValidationStats::new(),
        }
    }

    /// Validate an artefact with caching and stats
    pub async fn validate(&mut self, path: &Path, artefact_type: ArtefactType) -> ValidationResult {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached) = self.check_cache(path).await {
            self.stats.record_cache_hit();
            debug!("[VALIDATOR] Cache hit for {:?}", path);
            return cached;
        }

        // Perform validation
        let result = match artefact_type {
            ArtefactType::Blend => self.validate_blend(path).await,
            ArtefactType::Midi => self.validate_midi(path).await,
            ArtefactType::Python => self.validate_python(path).await,
            ArtefactType::Shader => self.validate_shader(path).await,
            ArtefactType::Json => self.validate_json(path).await,
            ArtefactType::Text => self.validate_text(path).await,
            ArtefactType::Unknown => ValidationResult::Skipped("Unknown artefact type".into()),
        };

        // Record stats
        let duration = start_time.elapsed();
        self.stats
            .record_validation(artefact_type, &result, duration);

        // Cache result
        if let Ok(hash) = self.compute_file_hash(path).await {
            let hash_clone = hash.clone();
            self.cache.insert(
                hash,
                CachedValidation {
                    result: result.clone(),
                    timestamp: Instant::now(),
                    file_hash: hash_clone,
                },
            );
        }

        debug!(
            "[VALIDATOR] Validated {:?} in {:?}: {:?}",
            path, duration, result
        );
        result
    }

    async fn check_cache(&self, path: &Path) -> Option<ValidationResult> {
        let hash = self.compute_file_hash(path).await.ok()?;
        let cached = self.cache.get(&hash)?;

        // Check if cache is still valid (1 hour TTL)
        if cached.timestamp.elapsed() < Duration::from_secs(3600) {
            Some(cached.result.clone())
        } else {
            None
        }
    }

    async fn compute_file_hash(&self, path: &Path) -> Result<String, String> {
        let contents = tokio::fs::read(path)
            .await
            .map_err(|e| format!("Failed to read file for hashing: {}", e))?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn validate_blend(&self, path: &Path) -> ValidationResult {
        let Some(blender) = &self.blender_path else {
            return ValidationResult::Skipped("Blender not found".into());
        };

        info!("[VALIDATOR] Validating Blender file: {:?}", path);

        let result = timeout(
            self.timeout_duration,
            tokio::process::Command::new(blender)
                .arg("--background")
                .arg("--no-window-focus")
                .arg(path)
                .arg("--python-expr")
                .arg("import bpy; exit(0 if bpy.data.objects else 1)")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .status(),
        )
        .await;

        match result {
            Ok(Ok(status)) if status.success() => {
                info!("[VALIDATOR] Blender file valid");
                ValidationResult::Valid
            }
            Ok(Ok(_)) => {
                warn!("[VALIDATOR] Blender file empty or corrupted");
                ValidationResult::Invalid("Blender file empty or corrupted".into())
            }
            Ok(Err(e)) => {
                error!("[VALIDATOR] Blender validation error: {}", e);
                ValidationResult::Error(e.to_string())
            }
            Err(_) => {
                warn!("[VALIDATOR] Blender validation timeout");
                ValidationResult::Invalid("Validation timeout".into())
            }
        }
    }

    async fn validate_midi(&self, path: &Path) -> ValidationResult {
        info!("[VALIDATOR] Validating MIDI file: {:?}", path);

        let bytes = match tokio::fs::read(path).await {
            Ok(b) => b,
            Err(e) => {
                error!("[VALIDATOR] Failed to read MIDI file: {}", e);
                return ValidationResult::Error(e.to_string());
            }
        };

        match midly::Smf::parse(&bytes) {
            Ok(smf) => {
                if smf.tracks.is_empty() {
                    warn!("[VALIDATOR] MIDI file has no tracks");
                    ValidationResult::Invalid("MIDI file has no tracks".into())
                } else {
                    info!(
                        "[VALIDATOR] MIDI file valid with {} tracks",
                        smf.tracks.len()
                    );
                    ValidationResult::Valid
                }
            }
            Err(e) => {
                warn!("[VALIDATOR] MIDI parse error: {}", e);
                ValidationResult::Invalid(format!("MIDI parse error: {}", e))
            }
        }
    }

    async fn validate_python(&self, path: &Path) -> ValidationResult {
        let Some(python) = &self.python_path else {
            return ValidationResult::Skipped("Python not found".into());
        };

        info!("[VALIDATOR] Validating Python file: {:?}", path);

        let result = timeout(
            self.timeout_duration,
            tokio::process::Command::new(python)
                .arg("-m")
                .arg("py_compile")
                .arg(path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) if output.status.success() => {
                info!("[VALIDATOR] Python file valid");
                ValidationResult::Valid
            }
            Ok(Ok(output)) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("[VALIDATOR] Python syntax error: {}", stderr);
                ValidationResult::Invalid(format!("Syntax error: {}", stderr))
            }
            Ok(Err(e)) => {
                error!("[VALIDATOR] Python validation error: {}", e);
                ValidationResult::Error(e.to_string())
            }
            Err(_) => {
                warn!("[VALIDATOR] Python validation timeout");
                ValidationResult::Invalid("Validation timeout".into())
            }
        }
    }

    async fn validate_shader(&self, path: &Path) -> ValidationResult {
        info!("[VALIDATOR] Validating Shader file: {:?}", path);

        let source = match tokio::fs::read_to_string(path).await {
            Ok(s) => s,
            Err(e) => {
                error!("[VALIDATOR] Failed to read shader file: {}", e);
                return ValidationResult::Error(e.to_string());
            }
        };

        // Check UTF-8 encoding
        if !source.is_char_boundary(source.len()) {
            warn!("[VALIDATOR] Shader file has invalid UTF-8 encoding");
            return ValidationResult::Invalid("Invalid UTF-8 encoding".into());
        }

        // Basic GLSL syntax check
        if !source.contains("void main()") {
            warn!("[VALIDATOR] Shader missing main() function");
            return ValidationResult::Invalid("Missing main() function".into());
        }

        // Check for common GLSL keywords
        let has_glsl = source.contains("gl_Position")
            || source.contains("gl_FragColor")
            || source.contains("varying")
            || source.contains("uniform")
            || source.contains("attribute")
            || source.contains("in")
            || source.contains("out");

        if !has_glsl {
            warn!("[VALIDATOR] Shader doesn't appear to be valid GLSL");
            return ValidationResult::Invalid("Doesn't appear to be valid GLSL".into());
        }

        info!("[VALIDATOR] Shader file valid");
        ValidationResult::Valid
    }

    async fn validate_json(&self, path: &Path) -> ValidationResult {
        info!("[VALIDATOR] Validating JSON file: {:?}", path);

        let contents = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                error!("[VALIDATOR] Failed to read JSON file: {}", e);
                return ValidationResult::Error(e.to_string());
            }
        };

        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(_) => {
                info!("[VALIDATOR] JSON file valid");
                ValidationResult::Valid
            }
            Err(e) => {
                warn!("[VALIDATOR] JSON parse error: {}", e);
                ValidationResult::Invalid(format!("JSON parse error: {}", e))
            }
        }
    }

    async fn validate_text(&self, path: &Path) -> ValidationResult {
        info!("[VALIDATOR] Validating Text file: {:?}", path);

        let contents = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) => {
                error!("[VALIDATOR] Failed to read text file: {}", e);
                return ValidationResult::Error(e.to_string());
            }
        };

        // Basic text validation
        if contents.trim().is_empty() {
            warn!("[VALIDATOR] Text file is empty");
            return ValidationResult::Invalid("Text file is empty".into());
        }

        // Check for reasonable length (not too short, not too long)
        if contents.len() < 10 {
            warn!("[VALIDATOR] Text file too short");
            return ValidationResult::Invalid("Text file too short".into());
        }

        if contents.len() > 1_000_000 {
            warn!("[VALIDATOR] Text file too long");
            return ValidationResult::Invalid("Text file too long".into());
        }

        info!("[VALIDATOR] Text file valid");
        ValidationResult::Valid
    }

    fn find_blender() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        let paths = vec![
            "/Applications/Blender.app/Contents/MacOS/Blender",
            "/usr/local/bin/blender",
            "/opt/homebrew/bin/blender",
        ];

        #[cfg(target_os = "windows")]
        let paths = vec![
            "C:\\Program Files\\Blender Foundation\\Blender\\blender.exe",
            "C:\\Program Files (x86)\\Blender Foundation\\Blender\\blender.exe",
            "C:\\Users\\%USERNAME%\\AppData\\Local\\Programs\\Blender Foundation\\Blender\\blender.exe",
        ];

        #[cfg(target_os = "linux")]
        let paths = vec![
            "/usr/bin/blender",
            "/usr/local/bin/blender",
            "/snap/bin/blender",
        ];

        for path in paths {
            let expanded_path = shellexpand::env(path).unwrap_or(path.into());
            let path_buf = PathBuf::from(expanded_path.as_ref());
            if path_buf.exists() {
                debug!("[VALIDATOR] Found Blender at: {:?}", path_buf);
                return Some(path_buf);
            }
        }

        debug!("[VALIDATOR] Blender not found");
        None
    }

    fn find_python() -> Option<PathBuf> {
        // Try python3 first, then python
        if let Ok(path) = which::which("python3") {
            debug!("[VALIDATOR] Found Python3 at: {:?}", path);
            return Some(path);
        }

        if let Ok(path) = which::which("python") {
            debug!("[VALIDATOR] Found Python at: {:?}", path);
            return Some(path);
        }

        debug!("[VALIDATOR] Python not found");
        None
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> &ValidationStats {
        &self.stats
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        info!("[VALIDATOR] Cache cleared");
    }

    /// Check if validators are available
    pub fn get_validator_status(&self) -> ValidatorStatus {
        ValidatorStatus {
            blender_available: self.blender_path.is_some(),
            python_available: self.python_path.is_some(),
            blender_path: self.blender_path.clone(),
            python_path: self.python_path.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
    Error(String),
    Skipped(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, ValidationResult::Skipped(_))
    }

    pub fn should_learn(&self) -> bool {
        matches!(self, ValidationResult::Valid | ValidationResult::Skipped(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtefactType {
    Blend,
    Midi,
    Python,
    Shader,
    Json,
    Text,
    Unknown,
}

impl ArtefactType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "blend" => Self::Blend,
            "mid" | "midi" => Self::Midi,
            "py" => Self::Python,
            "glsl" | "vert" | "frag" | "comp" => Self::Shader,
            "json" => Self::Json,
            "txt" | "md" | "rst" => Self::Text,
            _ => Self::Unknown,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::Unknown)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStatus {
    pub blender_available: bool,
    pub python_available: bool,
    pub blender_path: Option<PathBuf>,
    pub python_path: Option<PathBuf>,
}
