#![allow(unused_imports)]
#![allow(dead_code)]

pub mod generator;

// Re-export types for external use
pub use generator::{ArtefactGenerator, GeneratedArtifact, ArtefactMetadata};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, error};

/// Main artifact management system
pub struct ArtefactSystem {
    generator: generator::ArtefactGenerator,
    stats: ArtefactStats,
}

impl ArtefactSystem {
    pub fn new() -> Self {
        Self {
            generator: generator::ArtefactGenerator::new(),
            stats: ArtefactStats::new(),
        }
    }

    /// Generate an artifact
    pub async fn generate_artifact(
        &mut self,
        domain: String,
        intent: String,
        trust_score: f32,
        idle_time: f32,
        cluster_id: String,
        artefact_type: crate::validator::ArtefactType,
    ) -> Result<GeneratedArtifact, String> {
        let start_time = std::time::Instant::now();
        
        info!("[ARTEFACT SYSTEM] Generating {:?} artifact", artefact_type);
        
        let result = self.generator.generate(
            domain,
            intent,
            trust_score,
            idle_time,
            cluster_id,
            artefact_type,
        ).await;
        
        let duration = start_time.elapsed();
        
        match &result {
            Ok(artifact) => {
                self.stats.record_success(duration);
                info!("[ARTEFACT SYSTEM] Generated successfully in {:?}", duration);
            }
            Err(e) => {
                self.stats.record_failure(e.clone());
                error!("[ARTEFACT SYSTEM] Failed: {}", e);
            }
        }
        
        result
    }

    /// Get statistics
    pub fn get_stats(&self) -> &ArtefactStats {
        &self.stats
    }
}

/// Statistics for artifact generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtefactStats {
    pub total_generated: u64,
    pub successful: u64,
    pub failed: u64,
    pub average_generation_time_ms: f64,
    pub total_generation_time_ms: u64,
    pub by_type: std::collections::HashMap<String, u64>,
}

impl ArtefactStats {
    pub fn new() -> Self {
        Self {
            total_generated: 0,
            successful: 0,
            failed: 0,
            average_generation_time_ms: 0.0,
            total_generation_time_ms: 0,
            by_type: std::collections::HashMap::new(),
        }
    }

    fn record_success(&mut self, duration: std::time::Duration) {
        self.total_generated += 1;
        self.successful += 1;
        self.total_generation_time_ms += duration.as_millis() as u64;
        self.average_generation_time_ms = 
            self.total_generation_time_ms as f64 / self.total_generated as f64;
    }

    fn record_failure(&mut self, _error: String) {
        self.total_generated += 1;
        self.failed += 1;
    }
}

