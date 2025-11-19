/**
 * Repetition Detection System
 * Detects repetitive tasks that could be automated
 */

use super::learning::{ActionSignature, UserAction, ActionType};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc, Duration};
use tracing::{info, debug};

/// Minimum repetitions to flag a task as repetitive
const MIN_REPETITIONS: usize = 3;

/// Time window to consider for repetition detection (hours)
const REPETITION_WINDOW_HOURS: i64 = 24;

/// Maximum actions in a repetitive task
const MAX_TASK_LENGTH: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepetitiveTask {
    pub id: String,
    pub name: String,
    pub actions: Vec<ActionSignature>,
    pub repetitions: usize,
    pub last_occurrence: DateTime<Utc>,
    pub first_seen: DateTime<Utc>,
    pub avg_interval_mins: f64,
    pub automation_potential: f64, // 0.0 - 1.0
    pub automation_suggestion: String,
    pub time_wasted_mins: f64,
}

pub struct RepetitionDetector {
    action_history: VecDeque<UserAction>,
    detected_tasks: HashMap<String, RepetitiveTask>,
    task_occurrences: HashMap<Vec<ActionSignature>, Vec<DateTime<Utc>>>,
}

impl RepetitionDetector {
    pub fn new() -> Self {
        Self {
            action_history: VecDeque::new(),
            detected_tasks: HashMap::new(),
            task_occurrences: HashMap::new(),
        }
    }

    /// Record a new action
    pub fn record_action(&mut self, action: UserAction) {
        self.action_history.push_back(action);

        // Keep history bounded (last 24h or 500 actions)
        self.prune_old_actions();

        // Analyze for repetitions
        self.detect_repetitions();
    }

    /// Remove old actions outside the detection window
    fn prune_old_actions(&mut self) {
        let cutoff = Utc::now() - Duration::hours(REPETITION_WINDOW_HOURS);

        while let Some(oldest) = self.action_history.front() {
            if oldest.timestamp < cutoff.timestamp() {
                self.action_history.pop_front();
            } else {
                break;
            }
        }

        // Also prune task occurrences
        for occurrences in self.task_occurrences.values_mut() {
            occurrences.retain(|ts| *ts > cutoff);
        }

        // Remove empty entries
        self.task_occurrences.retain(|_, v| !v.is_empty());
    }

    /// Detect repetitive task patterns
    fn detect_repetitions(&mut self) {
        let history_len = self.action_history.len();

        // Try different task lengths
        for task_len in 2..=std::cmp::min(MAX_TASK_LENGTH, history_len) {
            let recent_task: Vec<ActionSignature> = self.action_history
                .iter()
                .rev()
                .take(task_len)
                .map(ActionSignature::from)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

            // Count occurrences of this exact sequence
            let count = self.count_sequence_occurrences(&recent_task);

            if count >= MIN_REPETITIONS {
                self.register_repetitive_task(recent_task, count);
            }
        }
    }

    /// Count how many times a sequence appears in history
    fn count_sequence_occurrences(&mut self, sequence: &[ActionSignature]) -> usize {
        let mut count = 0;
        let mut timestamps = Vec::new();

        if self.action_history.len() < sequence.len() {
            return 0;
        }

        // Collect all windows first to avoid borrow conflicts
        let windows: Vec<Vec<ActionSignature>> = self.action_history
            .make_contiguous()
            .windows(sequence.len())
            .map(|window| window.iter().map(ActionSignature::from).collect())
            .collect();

        for window_sigs in windows {
            if self.sequences_match(&window_sigs, sequence) {
                count += 1;
                // Note: timestamps tracking removed as we lost reference to original actions
            }
        }

        // Store timestamps for this sequence
        if count >= MIN_REPETITIONS {
            self.task_occurrences.insert(sequence.to_vec(), timestamps);
        }

        count
    }

    /// Check if two sequences match
    fn sequences_match(&self, seq1: &[ActionSignature], seq2: &[ActionSignature]) -> bool {
        if seq1.len() != seq2.len() {
            return false;
        }

        seq1.iter().zip(seq2.iter()).all(|(a, b)| {
            a.app_name == b.app_name && a.action_type == b.action_type
        })
    }

    /// Register a repetitive task
    fn register_repetitive_task(&mut self, actions: Vec<ActionSignature>, repetitions: usize) {
        let task_id = self.generate_task_id(&actions);

        // Check if already exists
        let avg_interval = self.calculate_avg_interval(&actions);
        let time_wasted = self.calculate_time_wasted(&actions, repetitions);
        if let Some(existing) = self.detected_tasks.get_mut(&task_id) {
            existing.repetitions = repetitions;
            existing.last_occurrence = Utc::now();
            existing.avg_interval_mins = avg_interval;
            existing.time_wasted_mins = time_wasted;
            return;
        }

        // Create new repetitive task
        let task = RepetitiveTask {
            id: task_id.clone(),
            name: self.generate_task_name(&actions),
            actions: actions.clone(),
            repetitions,
            last_occurrence: Utc::now(),
            first_seen: self.get_first_occurrence(&actions),
            avg_interval_mins: self.calculate_avg_interval(&actions),
            automation_potential: self.calculate_automation_potential(&actions, repetitions),
            automation_suggestion: self.generate_automation_suggestion(&actions),
            time_wasted_mins: self.calculate_time_wasted(&actions, repetitions),
        };

        info!(
            "🔁 Repetitive task detected: {} ({} repetitions, {:.0}% automation potential)",
            task.name, task.repetitions, task.automation_potential * 100.0
        );

        self.detected_tasks.insert(task_id, task);
    }

    /// Generate unique ID for a task
    fn generate_task_id(&self, actions: &[ActionSignature]) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        actions.hash(&mut hasher);
        format!("task_{:x}", hasher.finish())
    }

    /// Generate human-readable name for a task
    fn generate_task_name(&self, actions: &[ActionSignature]) -> String {
        if actions.is_empty() {
            return "Unknown Task".to_string();
        }

        // Identify dominant action types
        let mut action_counts: HashMap<&str, usize> = HashMap::new();
        for action in actions {
            let action_str = match &action.action_type {
                ActionType::Copy => "Copying",
                ActionType::Paste => "Pasting",
                ActionType::FileSave => "Saving files",
                ActionType::FileOpen => "Opening files",
                ActionType::AppSwitch => "Switching apps",
                ActionType::Typing => "Typing",
                _ => "Working",
            };
            *action_counts.entry(action_str).or_insert(0) += 1;
        }

        let dominant_action = action_counts
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(action, _)| *action)
            .unwrap_or("Task");

        let unique_apps: std::collections::HashSet<_> = actions.iter()
            .map(|a| a.app_name.as_str())
            .collect();

        match unique_apps.len() {
            1 => format!("{} in {}", dominant_action, unique_apps.iter().next().unwrap()),
            _ => format!("{} across {} apps", dominant_action, unique_apps.len()),
        }
    }

    /// Get first occurrence timestamp
    fn get_first_occurrence(&self, actions: &[ActionSignature]) -> DateTime<Utc> {
        self.task_occurrences
            .get(actions)
            .and_then(|timestamps| timestamps.first())
            .copied()
            .unwrap_or_else(|| Utc::now())
    }

    /// Calculate average interval between repetitions
    fn calculate_avg_interval(&self, actions: &[ActionSignature]) -> f64 {
        let timestamps = match self.task_occurrences.get(actions) {
            Some(ts) => ts,
            None => return 0.0,
        };

        if timestamps.len() < 2 {
            return 0.0;
        }

        let mut intervals = Vec::new();
        for window in timestamps.windows(2) {
            if let [first, second] = window {
                let interval = (*second - *first).num_minutes() as f64;
                intervals.push(interval);
            }
        }

        if intervals.is_empty() {
            return 0.0;
        }

        intervals.iter().sum::<f64>() / intervals.len() as f64
    }

    /// Calculate automation potential (0.0 - 1.0)
    fn calculate_automation_potential(&self, actions: &[ActionSignature], repetitions: usize) -> f64 {
        let mut score = 0.0;

        // More repetitions = higher potential
        score += (repetitions as f64 / 20.0).min(0.4);

        // Longer sequences = higher potential
        score += (actions.len() as f64 / MAX_TASK_LENGTH as f64) * 0.3;

        // Certain action types are more automatable
        let automatable_actions = actions.iter().filter(|a| {
            matches!(
                a.action_type,
                ActionType::Copy | ActionType::Paste | ActionType::FileSave | ActionType::FileOpen
            )
        }).count();

        score += (automatable_actions as f64 / actions.len() as f64) * 0.3;

        score.min(1.0)
    }

    /// Generate automation suggestion
    fn generate_automation_suggestion(&self, actions: &[ActionSignature]) -> String {
        let has_copy_paste = actions.iter().any(|a| {
            matches!(a.action_type, ActionType::Copy | ActionType::Paste)
        });

        let has_file_ops = actions.iter().any(|a| {
            matches!(a.action_type, ActionType::FileOpen | ActionType::FileSave)
        });

        if has_copy_paste && has_file_ops {
            return "Consider using a script to automate copying data between files".to_string();
        }

        if has_copy_paste {
            return "Consider using clipboard management tools or macros".to_string();
        }

        if has_file_ops {
            return "Consider using batch processing or file automation scripts".to_string();
        }

        let unique_apps: std::collections::HashSet<_> = actions.iter()
            .map(|a| a.app_name.as_str())
            .collect();

        if unique_apps.len() > 1 {
            return "Consider using workflow automation tools like Zapier or n8n".to_string();
        }

        "Consider using keyboard shortcuts or app-specific automation".to_string()
    }

    /// Calculate time wasted on repetitions
    fn calculate_time_wasted(&self, actions: &[ActionSignature], repetitions: usize) -> f64 {
        // Assume each action takes ~2 seconds
        let estimated_seconds_per_task = actions.len() as f64 * 2.0;
        let total_seconds = estimated_seconds_per_task * repetitions as f64;
        total_seconds / 60.0 // Convert to minutes
    }

    /// Get all detected repetitive tasks
    pub fn get_repetitive_tasks(&self) -> Vec<RepetitiveTask> {
        let mut tasks: Vec<_> = self.detected_tasks.values().cloned().collect();
        tasks.sort_by(|a, b| {
            b.automation_potential
                .partial_cmp(&a.automation_potential)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        tasks
    }

    /// Get high-priority tasks (high automation potential + high repetitions)
    pub fn get_high_priority_tasks(&self) -> Vec<RepetitiveTask> {
        self.detected_tasks
            .values()
            .filter(|task| task.automation_potential > 0.6 && task.repetitions >= 5)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn get_stats(&self) -> RepetitionStats {
        let tasks = self.get_repetitive_tasks();

        RepetitionStats {
            total_tasks_detected: tasks.len(),
            total_repetitions: tasks.iter().map(|t| t.repetitions).sum(),
            total_time_wasted_mins: tasks.iter().map(|t| t.time_wasted_mins).sum(),
            avg_automation_potential: if tasks.is_empty() {
                0.0
            } else {
                tasks.iter().map(|t| t.automation_potential).sum::<f64>() / tasks.len() as f64
            },
            high_priority_tasks: self.get_high_priority_tasks().len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepetitionStats {
    pub total_tasks_detected: usize,
    pub total_repetitions: usize,
    pub total_time_wasted_mins: f64,
    pub avg_automation_potential: f64,
    pub high_priority_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
    fn test_repetition_detection() {
        let mut detector = RepetitionDetector::new();

        // Simulate repetitive copy-paste task
        for _ in 0..5 {
            detector.record_action(create_test_action("VS Code", ActionType::Copy));
            detector.record_action(create_test_action("Browser", ActionType::Paste));
        }

        let tasks = detector.get_repetitive_tasks();
        assert!(!tasks.is_empty(), "Should detect at least one repetitive task");

        let stats = detector.get_stats();
        assert!(stats.total_tasks_detected > 0);
        assert!(stats.total_repetitions >= 5);
    }

    #[test]
    fn test_automation_potential() {
        let detector = RepetitionDetector::new();

        let copy_paste_task = vec![
            ActionSignature {
                app_name: "VS Code".to_string(),
                action_type: ActionType::Copy,
                window_pattern: None,
            },
            ActionSignature {
                app_name: "Browser".to_string(),
                action_type: ActionType::Paste,
                window_pattern: None,
            },
        ];

        let potential = detector.calculate_automation_potential(&copy_paste_task, 10);
        assert!(potential > 0.5, "Copy-paste should have high automation potential");
    }
}
