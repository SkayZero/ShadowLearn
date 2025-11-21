/**
 * Pattern Learning Engine
 * Learns workflow patterns from user behavior
 */

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use tracing::{info, debug};

/// Maximum number of events to keep in memory
const MAX_EVENT_HISTORY: usize = 1000;

/// Minimum sequence length to consider as a pattern
const MIN_PATTERN_LENGTH: usize = 2;

/// Minimum occurrences to confirm a pattern
const MIN_PATTERN_OCCURRENCES: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserAction {
    pub app_name: String,
    pub action_type: ActionType,
    pub window_title: Option<String>,
    pub timestamp: i64,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    AppSwitch,
    WindowFocus,
    FileOpen,
    FileSave,
    Typing,
    Click,
    Scroll,
    Copy,
    Paste,
    Command,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub id: String,
    pub name: String,
    pub sequence: Vec<ActionSignature>,
    pub occurrences: usize,
    pub confidence: f64,
    pub last_seen: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub avg_duration_secs: f64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActionSignature {
    pub app_name: String,
    pub action_type: ActionType,
    pub window_pattern: Option<String>, // Regex pattern for window title
}

impl From<&UserAction> for ActionSignature {
    fn from(action: &UserAction) -> Self {
        Self {
            app_name: action.app_name.clone(),
            action_type: action.action_type.clone(),
            window_pattern: action.window_title.as_ref().map(|title| {
                // Extract pattern from window title (e.g., "file.rs" → "*.rs")
                if let Some(ext_pos) = title.rfind('.') {
                    let ext = &title[ext_pos..];
                    format!("*{}", ext)
                } else {
                    title.clone()
                }
            }),
        }
    }
}

pub struct PatternLearner {
    event_history: VecDeque<UserAction>,
    discovered_patterns: HashMap<String, WorkflowPattern>,
    sequence_counts: HashMap<Vec<ActionSignature>, usize>,
}

impl PatternLearner {
    pub fn new() -> Self {
        Self {
            event_history: VecDeque::with_capacity(MAX_EVENT_HISTORY),
            discovered_patterns: HashMap::new(),
            sequence_counts: HashMap::new(),
        }
    }

    /// Record a user action
    pub fn record_action(&mut self, action: UserAction) {
        debug!("Recording action: {:?} in {}", action.action_type, action.app_name);

        // Add to history
        self.event_history.push_back(action);

        // Keep history bounded
        if self.event_history.len() > MAX_EVENT_HISTORY {
            self.event_history.pop_front();
        }

        // Analyze for patterns
        self.analyze_recent_patterns();
    }

    /// Analyze recent events for emerging patterns
    fn analyze_recent_patterns(&mut self) {
        let history_len = self.event_history.len();
        if history_len < MIN_PATTERN_LENGTH {
            return;
        }

        // Look for sequences of various lengths
        for seq_len in MIN_PATTERN_LENGTH..=std::cmp::min(5, history_len) {
            let recent_sequence: Vec<ActionSignature> = self.event_history
                .iter()
                .rev()
                .take(seq_len)
                .map(ActionSignature::from)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            // Count this sequence
            *self.sequence_counts.entry(recent_sequence.clone()).or_insert(0) += 1;

            // Check if it's a confirmed pattern
            let count = self.sequence_counts.get(&recent_sequence).copied().unwrap_or(0);
            if count >= MIN_PATTERN_OCCURRENCES {
                self.promote_to_pattern(recent_sequence, count);
            }
        }
    }

    /// Promote a frequent sequence to a confirmed pattern
    fn promote_to_pattern(&mut self, sequence: Vec<ActionSignature>, occurrences: usize) {
        let pattern_id = self.generate_pattern_id(&sequence);

        // Pre-calculate values before any borrows
        let confidence = self.calculate_confidence(occurrences);
        let avg_duration = self.calculate_avg_duration(&sequence);

        // Check if pattern already exists
        if let Some(existing) = self.discovered_patterns.get_mut(&pattern_id) {
            existing.occurrences = occurrences;
            existing.last_seen = Utc::now();
            existing.confidence = confidence;
            return;
        }

        // Create new pattern
        let pattern = WorkflowPattern {
            id: pattern_id.clone(),
            name: self.generate_pattern_name(&sequence),
            sequence: sequence.clone(),
            occurrences,
            confidence,
            last_seen: Utc::now(),
            created_at: Utc::now(),
            avg_duration_secs: avg_duration,
            tags: self.extract_tags(&sequence),
        };

        info!(
            "🧠 New pattern discovered: {} (confidence: {:.0}%, occurrences: {})",
            pattern.name, pattern.confidence * 100.0, pattern.occurrences
        );

        self.discovered_patterns.insert(pattern_id, pattern);
    }

    /// Generate unique ID for a pattern
    fn generate_pattern_id(&self, sequence: &[ActionSignature]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        sequence.hash(&mut hasher);
        format!("pattern_{:x}", hasher.finish())
    }

    /// Generate human-readable name for a pattern
    fn generate_pattern_name(&self, sequence: &[ActionSignature]) -> String {
        if sequence.is_empty() {
            return "Unknown Pattern".to_string();
        }

        let apps: Vec<&str> = sequence.iter()
            .map(|s| s.app_name.as_str())
            .collect();

        // Create name based on unique apps
        let unique_apps: Vec<&str> = {
            let mut seen = std::collections::HashSet::new();
            apps.into_iter().filter(|&app| seen.insert(app)).collect()
        };

        match unique_apps.len() {
            1 => format!("{} workflow", unique_apps[0]),
            2 => format!("{} → {} workflow", unique_apps[0], unique_apps[1]),
            3 => format!("{} → {} → {} workflow", unique_apps[0], unique_apps[1], unique_apps[2]),
            _ => format!("Multi-app workflow ({})", sequence.len()),
        }
    }

    /// Calculate confidence score based on occurrences
    fn calculate_confidence(&self, occurrences: usize) -> f64 {
        // Logistic function: confidence increases with occurrences but plateaus
        let x = occurrences as f64;
        1.0 / (1.0 + (-0.3 * (x - 5.0)).exp())
    }

    /// Calculate average duration for a sequence
    fn calculate_avg_duration(&mut self, sequence: &[ActionSignature]) -> f64 {
        // Look for all occurrences in history and calculate avg time
        let mut durations = Vec::new();

        for window in self.event_history.make_contiguous().windows(sequence.len()) {
            let window_sigs: Vec<ActionSignature> = window.iter()
                .map(ActionSignature::from)
                .collect();

            if window_sigs == sequence {
                if let (Some(first), Some(last)) = (window.first(), window.last()) {
                    let duration = (last.timestamp - first.timestamp) as f64;
                    durations.push(duration);
                }
            }
        }

        if durations.is_empty() {
            return 0.0;
        }

        durations.iter().sum::<f64>() / durations.len() as f64
    }

    /// Extract tags from sequence
    fn extract_tags(&self, sequence: &[ActionSignature]) -> Vec<String> {
        let mut tags = Vec::new();

        // App-based tags
        let unique_apps: std::collections::HashSet<String> = sequence.iter()
            .map(|s| s.app_name.clone())
            .collect();
        tags.extend(unique_apps.into_iter());

        // Action-type tags
        let has_file_ops = sequence.iter().any(|s| {
            matches!(s.action_type, ActionType::FileOpen | ActionType::FileSave)
        });
        if has_file_ops {
            tags.push("file-operations".to_string());
        }

        let has_copy_paste = sequence.iter().any(|s| {
            matches!(s.action_type, ActionType::Copy | ActionType::Paste)
        });
        if has_copy_paste {
            tags.push("clipboard".to_string());
        }

        tags
    }

    /// Get all discovered patterns
    pub fn get_patterns(&self) -> Vec<WorkflowPattern> {
        let mut patterns: Vec<_> = self.discovered_patterns.values().cloned().collect();
        patterns.sort_by(|a, b| {
            b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
        });
        patterns
    }

    /// Get patterns matching specific tags
    pub fn get_patterns_by_tag(&self, tag: &str) -> Vec<WorkflowPattern> {
        self.discovered_patterns
            .values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> PatternStats {
        PatternStats {
            total_actions_recorded: self.event_history.len(),
            total_patterns_discovered: self.discovered_patterns.len(),
            total_sequences_tracked: self.sequence_counts.len(),
            avg_pattern_confidence: {
                let patterns = self.get_patterns();
                if patterns.is_empty() {
                    0.0
                } else {
                    patterns.iter().map(|p| p.confidence).sum::<f64>() / patterns.len() as f64
                }
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    pub total_actions_recorded: usize,
    pub total_patterns_discovered: usize,
    pub total_sequences_tracked: usize,
    pub avg_pattern_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_action(app: &str, action_type: ActionType) -> UserAction {
        UserAction {
            app_name: app.to_string(),
            action_type,
            window_title: None,
            timestamp: Utc::now().timestamp(),
            context: HashMap::new(),
        }
    }

    #[test]
    fn test_pattern_learning() {
        let mut learner = PatternLearner::new();

        // Simulate repetitive workflow: VS Code → Terminal → VS Code
        for _ in 0..5 {
            learner.record_action(create_test_action("VS Code", ActionType::AppSwitch));
            learner.record_action(create_test_action("Terminal", ActionType::AppSwitch));
            learner.record_action(create_test_action("VS Code", ActionType::AppSwitch));
        }

        let patterns = learner.get_patterns();
        assert!(!patterns.is_empty(), "Should discover at least one pattern");

        let stats = learner.get_stats();
        assert!(stats.total_patterns_discovered > 0);
    }

    #[test]
    fn test_confidence_calculation() {
        let learner = PatternLearner::new();

        assert!(learner.calculate_confidence(1) < 0.5);
        assert!(learner.calculate_confidence(5) > 0.5);
        assert!(learner.calculate_confidence(20) > 0.9);
    }
}
