#![allow(unused_imports)]
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info};

pub mod detector;
pub mod llm_client;

use detector::*;
use llm_client::*;

/// Intent detection system with LLM integration
#[derive(Debug)]
#[allow(dead_code)]
pub struct IntentSystem {
    detector: IntentDetector,
    stats: IntentStats,
}

#[allow(dead_code)]
impl IntentSystem {
    pub fn new(llm_client: Arc<Mutex<LLMClient>>) -> Self {
        Self {
            detector: IntentDetector::new(llm_client),
            stats: IntentStats::new(),
        }
    }

    /// Detect intent from context
    pub async fn detect_intent(
        &mut self,
        ctx: &crate::context::aggregator::Context,
    ) -> Result<Intent, String> {
        let start_time = Instant::now();

        let intent = self.detector.detect_intent(ctx).await?;

        // Record stats
        let duration = start_time.elapsed();
        self.stats.record_detection(&intent, duration);

        debug!(
            "[INTENT] Detected: {:?} (confidence: {:.2}, duration: {:?})",
            intent.intent_type, intent.confidence, duration
        );

        Ok(intent)
    }

    /// Check if intent should proceed (confidence threshold)
    pub fn should_proceed(&self, intent: &Intent) -> bool {
        self.detector.should_proceed(intent)
    }

    /// Get intent statistics
    pub fn get_stats(&self) -> &IntentStats {
        &self.stats
    }

    /// Clear intent cache
    pub fn clear_cache(&mut self) {
        self.detector.clear_cache();
        info!("[INTENT] Cache cleared");
    }

    /// Get cache hit rate
    pub fn get_cache_hit_rate(&self) -> f64 {
        self.detector.get_cache_hit_rate()
    }

    /// Get cache size
    pub fn get_cache_size(&self) -> usize {
        self.detector.get_cache_size()
    }
}

/// Intent detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Intent {
    pub intent_type: IntentType,
    pub confidence: f32,
    pub reason: String,
    pub detected_at_ms: u64,
}

/// Types of user intents
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentType {
    Debugging,
    Learning,
    Creating,
    Researching,
    Stuck,
    Unknown,
}

#[allow(dead_code)]
impl IntentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentType::Debugging => "debugging",
            IntentType::Learning => "learning",
            IntentType::Creating => "creating",
            IntentType::Researching => "researching",
            IntentType::Stuck => "stuck",
            IntentType::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "debugging" => IntentType::Debugging,
            "learning" => IntentType::Learning,
            "creating" => IntentType::Creating,
            "researching" => IntentType::Researching,
            "stuck" => IntentType::Stuck,
            _ => IntentType::Unknown,
        }
    }
}

/// Intent detection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IntentStats {
    pub total_detections: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub high_confidence_count: u64,
    pub low_confidence_count: u64,
    pub average_confidence: f64,
    pub average_detection_time_ms: f64,
    pub total_detection_time_ms: u64,
    pub intent_distribution: HashMap<String, u64>,
    pub confidence_distribution: HashMap<String, u64>,
}

#[allow(dead_code)]
impl IntentStats {
    fn new() -> Self {
        Self {
            total_detections: 0,
            cache_hits: 0,
            cache_misses: 0,
            high_confidence_count: 0,
            low_confidence_count: 0,
            average_confidence: 0.0,
            average_detection_time_ms: 0.0,
            total_detection_time_ms: 0,
            intent_distribution: HashMap::new(),
            confidence_distribution: HashMap::new(),
        }
    }

    fn record_detection(&mut self, intent: &Intent, duration: Duration) {
        self.total_detections += 1;
        self.total_detection_time_ms += duration.as_millis() as u64;
        self.average_detection_time_ms =
            self.total_detection_time_ms as f64 / self.total_detections as f64;

        // Update confidence stats
        self.average_confidence = (self.average_confidence * (self.total_detections - 1) as f64
            + intent.confidence as f64)
            / self.total_detections as f64;

        if intent.confidence >= 0.7 {
            self.high_confidence_count += 1;
        } else {
            self.low_confidence_count += 1;
        }

        // Update distributions
        let intent_key = intent.intent_type.as_str().to_string();
        *self.intent_distribution.entry(intent_key).or_insert(0) += 1;

        let conf_bucket = if intent.confidence >= 0.9 {
            "0.9-1.0"
        } else if intent.confidence >= 0.8 {
            "0.8-0.9"
        } else if intent.confidence >= 0.7 {
            "0.7-0.8"
        } else if intent.confidence >= 0.6 {
            "0.6-0.7"
        } else if intent.confidence >= 0.5 {
            "0.5-0.6"
        } else {
            "0.0-0.5"
        };

        *self
            .confidence_distribution
            .entry(conf_bucket.to_string())
            .or_insert(0) += 1;
    }

    pub fn get_cache_hit_rate(&self) -> f64 {
        let total_cache_operations = self.cache_hits + self.cache_misses;
        if total_cache_operations == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total_cache_operations as f64
    }

    pub fn get_high_confidence_rate(&self) -> f64 {
        if self.total_detections == 0 {
            return 0.0;
        }
        self.high_confidence_count as f64 / self.total_detections as f64
    }
}
