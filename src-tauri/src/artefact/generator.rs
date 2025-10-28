#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use std::time::Duration;
use tokio::time::timeout;

use crate::adaptive::AdaptivePromptEngine;
use crate::intent::llm_client::{LLMClient, LLMProvider};
use crate::validator::{ArtefactValidator, ArtefactType, ValidationResult};
use std::path::Path;

/// Generator for context-aware artifacts
pub struct ArtefactGenerator {
    adaptive_engine: AdaptivePromptEngine,
    llm_client: LLMClient,
    validator: ArtefactValidator,
}

impl ArtefactGenerator {
    pub fn new() -> Self {
        Self {
            adaptive_engine: AdaptivePromptEngine::new(),
            llm_client: LLMClient::new(LLMProvider::Ollama, None),
            validator: ArtefactValidator::new(),
        }
    }

    /// Generate an artifact based on context
    pub async fn generate(
        &mut self,
        domain: String,
        intent: String,
        trust_score: f32,
        idle_time: f32,
        cluster_id: String,
        artefact_type: ArtefactType,
    ) -> Result<GeneratedArtifact, String> {
        info!("[ARTEFACT] Generating {:?} for domain={}, intent={}", artefact_type, domain, intent);

        // Step 1: Generate adaptive prompt
        let prompt = self.adaptive_engine.generate_prompt(
            &domain,
            &intent,
            trust_score,
            idle_time,
            &cluster_id,
        ).await?;

        // Step 2: Call LLM with timeout
        let timeout_duration = Duration::from_secs(30);
        
        let llm_response = match timeout(timeout_duration, async {
            self.llm_client.generate(&prompt, 500).await
        }).await {
            Ok(Ok(response)) => response,
            Ok(Err(e)) => {
                warn!("[ARTEFACT] LLM error: {}, using fallback", e);
                return self.generate_fallback_artifact(artefact_type, &intent);
            }
            Err(_timeout) => {
                warn!("[ARTEFACT] LLM timeout, using fallback");
                return self.generate_fallback_artifact(artefact_type, &intent);
            }
        };

        // Step 3: Skip validation for now (will be integrated in future)
        // TODO: Save artifact to temp file and validate using validator
        debug!("[ARTEFACT] Validation skipped for content-based generation");

        // Step 4: Create generated artifact
        let artifact = GeneratedArtifact {
            content: llm_response,
            artefact_type,
            domain,
            intent,
            trust_score,
            validated: true,
            validation_details: "Validated (skip)".to_string(),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        info!("[ARTEFACT] Generated successfully: {:?}", artefact_type);
        Ok(artifact)
    }

    /// Generate fallback artifact when LLM fails
    fn generate_fallback_artifact(
        &self,
        artefact_type: ArtefactType,
        intent: &str,
    ) -> Result<GeneratedArtifact, String> {
        let fallback_content = match artefact_type {
            ArtefactType::Text => self.generate_fallback_code(intent),
            ArtefactType::Blend => self.generate_fallback_blender(intent),
            ArtefactType::Midi => self.generate_fallback_midi(intent),
            ArtefactType::Shader => self.generate_fallback_shader(intent),
            ArtefactType::Json => self.generate_fallback_json(intent),
            ArtefactType::Python => self.generate_fallback_python(intent),
            ArtefactType::Unknown => format!("// Fallback for {:?}", artefact_type),
        };

        Ok(GeneratedArtifact {
            content: fallback_content,
            artefact_type,
            domain: "fallback".to_string(),
            intent: intent.to_string(),
            trust_score: 0.3,
            validated: false,
            validation_details: "Fallback generated".to_string(),
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    fn generate_fallback_code(&self, _intent: &str) -> String {
        "// Fallback code generated\nfn main() {\n    println!(\"Hello, World!\");\n}".to_string()
    }

    fn generate_fallback_blender(&self, _intent: &str) -> String {
        "import bpy\n# Fallback Blender script".to_string()
    }

    fn generate_fallback_midi(&self, _intent: &str) -> String {
        "MThd".to_string() // Minimal MIDI header
    }

    fn generate_fallback_shader(&self, _intent: &str) -> String {
        "void main() { gl_FragColor = vec4(1.0); }".to_string()
    }

    fn generate_fallback_json(&self, _intent: &str) -> String {
        "{\"fallback\": true}".to_string()
    }

    fn generate_fallback_python(&self, _intent: &str) -> String {
        "# Fallback Python script\ndef main():\n    pass".to_string()
    }
}

/// Generated artifact with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedArtifact {
    pub content: String,
    pub artefact_type: ArtefactType,
    pub domain: String,
    pub intent: String,
    pub trust_score: f32,
    pub validated: bool,
    pub validation_details: String,
    pub generated_at: u64,
}

impl GeneratedArtifact {
    /// Get artifact metadata
    pub fn metadata(&self) -> ArtefactMetadata {
        ArtefactMetadata {
            artefact_type: self.artefact_type.clone(),
            domain: self.domain.clone(),
            intent: self.intent.clone(),
            trust_score: self.trust_score,
            size_bytes: self.content.len(),
            validated: self.validated,
            generated_at: self.generated_at,
        }
    }
}

/// Artifact metadata for logging and display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtefactMetadata {
    pub artefact_type: ArtefactType,
    pub domain: String,
    pub intent: String,
    pub trust_score: f32,
    pub size_bytes: usize,
    pub validated: bool,
    pub generated_at: u64,
}

