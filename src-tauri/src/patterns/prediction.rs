/**
 * Action Prediction System
 * Predicts next user actions based on learned patterns
 */

use super::learning::{ActionSignature, UserAction, WorkflowPattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub predicted_action: ActionSignature,
    pub confidence: f64,
    pub reasoning: String,
    pub pattern_id: Option<String>,
    pub alternative_predictions: Vec<AlternativePrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativePrediction {
    pub action: ActionSignature,
    pub confidence: f64,
}

pub struct ActionPredictor {
    patterns: Vec<WorkflowPattern>,
    recent_actions: Vec<UserAction>,
    prediction_cache: HashMap<String, Prediction>,
}

impl ActionPredictor {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            recent_actions: Vec::new(),
            prediction_cache: HashMap::new(),
        }
    }

    /// Update known patterns
    pub fn update_patterns(&mut self, patterns: Vec<WorkflowPattern>) {
        self.patterns = patterns;
        // Clear cache when patterns update
        self.prediction_cache.clear();
    }

    /// Update recent action history
    pub fn update_recent_actions(&mut self, actions: Vec<UserAction>) {
        self.recent_actions = actions;
    }

    /// Predict next action based on current context
    pub fn predict_next_action(&mut self) -> Option<Prediction> {
        if self.recent_actions.is_empty() {
            return None;
        }

        // Create context key for caching
        let context_key = self.create_context_key();

        // Check cache
        if let Some(cached) = self.prediction_cache.get(&context_key) {
            debug!("📊 Using cached prediction");
            return Some(cached.clone());
        }

        // Find matching patterns
        let matching_patterns = self.find_matching_patterns();

        if matching_patterns.is_empty() {
            debug!("📊 No matching patterns found for prediction");
            return None;
        }

        // Generate predictions from each matching pattern
        let mut predictions = Vec::new();

        for (pattern, match_position) in matching_patterns {
            if match_position + 1 < pattern.sequence.len() {
                let next_action = pattern.sequence[match_position + 1].clone();
                let confidence = self.calculate_prediction_confidence(&pattern, match_position);

                predictions.push((next_action, confidence, pattern));
            }
        }

        if predictions.is_empty() {
            return None;
        }

        // Sort by confidence
        predictions.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Build final prediction
        let (best_action, best_confidence, best_pattern) = predictions[0].clone();

        let alternatives: Vec<AlternativePrediction> = predictions
            .iter()
            .skip(1)
            .take(3)
            .map(|(action, conf, _)| AlternativePrediction {
                action: action.clone(),
                confidence: *conf,
            })
            .collect();

        let prediction = Prediction {
            predicted_action: best_action,
            confidence: best_confidence,
            reasoning: self.generate_reasoning(&best_pattern, &predictions),
            pattern_id: Some(best_pattern.id.clone()),
            alternative_predictions: alternatives,
        };

        info!(
            "🔮 Prediction: {} in {} (confidence: {:.0}%)",
            prediction.predicted_action.action_type_str(),
            prediction.predicted_action.app_name,
            prediction.confidence * 100.0
        );

        // Cache the prediction
        self.prediction_cache.insert(context_key, prediction.clone());

        Some(prediction)
    }

    /// Find patterns that match recent action history
    fn find_matching_patterns(&self) -> Vec<(WorkflowPattern, usize)> {
        let mut matches = Vec::new();

        let recent_sigs: Vec<ActionSignature> = self.recent_actions
            .iter()
            .rev()
            .take(5)
            .map(ActionSignature::from)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        for pattern in &self.patterns {
            // Try to match pattern sequence with recent actions
            if let Some(match_pos) = self.find_partial_match(&pattern.sequence, &recent_sigs) {
                matches.push((pattern.clone(), match_pos));
            }
        }

        // Sort by pattern quality (occurrences * confidence)
        matches.sort_by(|a, b| {
            let score_a = a.0.occurrences as f64 * a.0.confidence;
            let score_b = b.0.occurrences as f64 * b.0.confidence;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    /// Find partial match between pattern and recent actions
    fn find_partial_match(
        &self,
        pattern_seq: &[ActionSignature],
        recent_sigs: &[ActionSignature],
    ) -> Option<usize> {
        if pattern_seq.is_empty() || recent_sigs.is_empty() {
            return None;
        }

        // Try to match the tail of recent_sigs with the beginning of pattern_seq
        for len in (1..=std::cmp::min(pattern_seq.len() - 1, recent_sigs.len())).rev() {
            let recent_tail = &recent_sigs[recent_sigs.len().saturating_sub(len)..];
            let pattern_head = &pattern_seq[..len];

            if self.sequences_match(recent_tail, pattern_head) {
                return Some(len - 1); // Return position in pattern
            }
        }

        None
    }

    /// Check if two sequences match (with fuzzy matching)
    fn sequences_match(&self, seq1: &[ActionSignature], seq2: &[ActionSignature]) -> bool {
        if seq1.len() != seq2.len() {
            return false;
        }

        seq1.iter().zip(seq2.iter()).all(|(a, b)| {
            a.app_name == b.app_name && a.action_type == b.action_type
        })
    }

    /// Calculate confidence for a prediction
    fn calculate_prediction_confidence(&self, pattern: &WorkflowPattern, match_pos: usize) -> f64 {
        // Base confidence from pattern
        let mut confidence = pattern.confidence;

        // Boost if match is near the end of pattern (more context)
        let match_ratio = (match_pos + 1) as f64 / pattern.sequence.len() as f64;
        confidence *= 0.7 + (0.3 * match_ratio);

        // Boost for high occurrence patterns
        if pattern.occurrences >= 10 {
            confidence *= 1.1;
        }

        confidence.min(1.0)
    }

    /// Generate human-readable reasoning
    fn generate_reasoning(
        &self,
        pattern: &WorkflowPattern,
        all_predictions: &[(ActionSignature, f64, WorkflowPattern)],
    ) -> String {
        let occurrences = pattern.occurrences;
        let alternatives = all_predictions.len() - 1;

        if alternatives == 0 {
            format!(
                "Based on '{}' pattern (seen {} times)",
                pattern.name, occurrences
            )
        } else {
            format!(
                "Based on '{}' pattern (seen {} times, {} alternatives)",
                pattern.name, occurrences, alternatives
            )
        }
    }

    /// Create context key for caching
    fn create_context_key(&self) -> String {
        self.recent_actions
            .iter()
            .rev()
            .take(3)
            .map(|a| format!("{}:{:?}", a.app_name, a.action_type))
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Get prediction statistics
    pub fn get_stats(&self) -> PredictionStats {
        PredictionStats {
            patterns_loaded: self.patterns.len(),
            recent_actions_count: self.recent_actions.len(),
            cached_predictions: self.prediction_cache.len(),
        }
    }
}

impl ActionSignature {
    pub fn action_type_str(&self) -> &str {
        match &self.action_type {
            super::learning::ActionType::AppSwitch => "switch to",
            super::learning::ActionType::WindowFocus => "focus",
            super::learning::ActionType::FileOpen => "open file",
            super::learning::ActionType::FileSave => "save file",
            super::learning::ActionType::Typing => "type",
            super::learning::ActionType::Click => "click",
            super::learning::ActionType::Scroll => "scroll",
            super::learning::ActionType::Copy => "copy",
            super::learning::ActionType::Paste => "paste",
            super::learning::ActionType::Command => "run command",
            super::learning::ActionType::Custom(s) => s,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionStats {
    pub patterns_loaded: usize,
    pub recent_actions_count: usize,
    pub cached_predictions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patterns::learning::{ActionType, WorkflowPattern};
    use chrono::Utc;

    fn create_test_pattern() -> WorkflowPattern {
        WorkflowPattern {
            id: "test_pattern".to_string(),
            name: "Test Workflow".to_string(),
            sequence: vec![
                ActionSignature {
                    app_name: "VS Code".to_string(),
                    action_type: ActionType::AppSwitch,
                    window_pattern: None,
                },
                ActionSignature {
                    app_name: "Terminal".to_string(),
                    action_type: ActionType::AppSwitch,
                    window_pattern: None,
                },
                ActionSignature {
                    app_name: "Browser".to_string(),
                    action_type: ActionType::AppSwitch,
                    window_pattern: None,
                },
            ],
            occurrences: 10,
            confidence: 0.85,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            avg_duration_secs: 30.0,
            tags: vec!["development".to_string()],
        }
    }

    #[test]
    fn test_prediction() {
        let mut predictor = ActionPredictor::new();
        predictor.update_patterns(vec![create_test_pattern()]);

        // Simulate recent actions matching the pattern start
        let recent = vec![
            UserAction {
                app_name: "VS Code".to_string(),
                action_type: ActionType::AppSwitch,
                window_title: None,
                timestamp: Utc::now().timestamp(),
                context: HashMap::new(),
            },
        ];

        predictor.update_recent_actions(recent);

        let prediction = predictor.predict_next_action();
        assert!(prediction.is_some());

        let pred = prediction.unwrap();
        assert_eq!(pred.predicted_action.app_name, "Terminal");
        assert!(pred.confidence > 0.5);
    }
}
