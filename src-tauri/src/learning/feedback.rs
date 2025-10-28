#![allow(unused_imports)]
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::persistence::database::DatabaseManager;
use crate::learning::reward::Outcome;
use crate::learning::trust::TrustManager;

/// Feedback collector for learning loop
pub struct FeedbackCollector {
    trust_manager: TrustManager,
    metrics: FeedbackMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackOutcome {
    Positive,  // Artefact was useful/used
    Negative,  // Artefact was not useful
    Neutral,   // No explicit feedback
}

impl FeedbackCollector {
    pub fn new(trust_manager: TrustManager) -> Self {
        Self {
            trust_manager,
            metrics: FeedbackMetrics::new(),
        }
    }

    /// Record user feedback on an artifact
    pub async fn record_feedback(
        &mut self,
        suggestion_id: &str,
        outcome: FeedbackOutcome,
    ) -> Result<f32, String> {
        info!("[FEEDBACK] Recording {:?} feedback for suggestion {}", outcome, suggestion_id);

        // Update metrics
        self.metrics.record_feedback(&outcome);

        // Simulate trust update (will be properly implemented later)
        let updated_trust = 0.5; // Placeholder
        
        info!("[FEEDBACK] Updated trust to {:.2} for suggestion {}", updated_trust, suggestion_id);
        
        Ok(updated_trust)
    }

    /// Get feedback metrics
    pub fn get_metrics(&self) -> &FeedbackMetrics {
        &self.metrics
    }
}

/// Metrics for feedback tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackMetrics {
    total_feedback: u64,
    positive_count: u64,
    negative_count: u64,
    neutral_count: u64,
    success_rate: f32,
}

impl FeedbackMetrics {
    pub fn new() -> Self {
        Self {
            total_feedback: 0,
            positive_count: 0,
            negative_count: 0,
            neutral_count: 0,
            success_rate: 0.0,
        }
    }

    fn record_feedback(&mut self, outcome: &FeedbackOutcome) {
        self.total_feedback += 1;
        
        match outcome {
            FeedbackOutcome::Positive => self.positive_count += 1,
            FeedbackOutcome::Negative => self.negative_count += 1,
            FeedbackOutcome::Neutral => self.neutral_count += 1,
        }

        // Calculate success rate
        if self.total_feedback > 0 {
            self.success_rate = self.positive_count as f32 / self.total_feedback as f32;
        }
    }
}

