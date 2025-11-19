/**
 * Tauri Commands for Pattern Recognition
 * Exposes pattern learning, prediction, and repetition detection to frontend
 */

use super::learning::{PatternLearner, WorkflowPattern, UserAction, PatternStats};
use super::prediction::{ActionPredictor, Prediction, PredictionStats};
use super::repetition::{RepetitionDetector, RepetitiveTask, RepetitionStats};
use super::storage::PatternStorage;
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

/// Global pattern recognition manager
pub struct PatternManager {
    learner: Arc<Mutex<PatternLearner>>,
    predictor: Arc<Mutex<ActionPredictor>>,
    detector: Arc<Mutex<RepetitionDetector>>,
    storage: Arc<Mutex<PatternStorage>>,
}

impl PatternManager {
    pub fn new(app_dir: std::path::PathBuf) -> Result<Self, String> {
        let storage = PatternStorage::new(app_dir)?;

        // Try to load existing patterns
        let learner = PatternLearner::new();
        let mut predictor = ActionPredictor::new();

        if let Ok(patterns) = storage.load_patterns() {
            info!("📊 Loaded {} patterns from storage", patterns.len());
            predictor.update_patterns(patterns);
        }

        Ok(Self {
            learner: Arc::new(Mutex::new(learner)),
            predictor: Arc::new(Mutex::new(predictor)),
            detector: Arc::new(Mutex::new(RepetitionDetector::new())),
            storage: Arc::new(Mutex::new(storage)),
        })
    }

    /// Record a user action across all systems
    pub async fn record_action(&self, action: UserAction) {
        // Record in learner
        {
            let mut learner = self.learner.lock().await;
            learner.record_action(action.clone());

            // Update predictor with new patterns
            let patterns = learner.get_patterns();
            let mut predictor = self.predictor.lock().await;
            predictor.update_patterns(patterns);
        }

        // Record in repetition detector
        {
            let mut detector = self.detector.lock().await;
            detector.record_action(action);
        }

        // Save to disk periodically (every 10 actions)
        // Note: In production, use a timer-based save instead
        // This is just a simple implementation
    }

    /// Get current prediction
    pub async fn get_prediction(&self) -> Option<Prediction> {
        let mut predictor = self.predictor.lock().await;
        predictor.predict_next_action()
    }

    /// Get all learned patterns
    pub async fn get_patterns(&self) -> Vec<WorkflowPattern> {
        let learner = self.learner.lock().await;
        learner.get_patterns()
    }

    /// Get all repetitive tasks
    pub async fn get_repetitive_tasks(&self) -> Vec<RepetitiveTask> {
        let detector = self.detector.lock().await;
        detector.get_repetitive_tasks()
    }

    /// Get high-priority repetitive tasks
    pub async fn get_high_priority_tasks(&self) -> Vec<RepetitiveTask> {
        let detector = self.detector.lock().await;
        detector.get_high_priority_tasks()
    }

    /// Save current state to disk
    pub async fn save(&self) -> Result<(), String> {
        let learner = self.learner.lock().await;
        let detector = self.detector.lock().await;
        let storage = self.storage.lock().await;

        storage.save_patterns(&learner.get_patterns())?;
        storage.save_tasks(&detector.get_repetitive_tasks())?;

        Ok(())
    }

    /// Get comprehensive statistics
    pub async fn get_stats(&self) -> PatternSystemStats {
        let learner = self.learner.lock().await;
        let predictor = self.predictor.lock().await;
        let detector = self.detector.lock().await;

        PatternSystemStats {
            learning: learner.get_stats(),
            prediction: predictor.get_stats(),
            repetition: detector.get_stats(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSystemStats {
    pub learning: PatternStats,
    pub prediction: PredictionStats,
    pub repetition: RepetitionStats,
}

// ===== Tauri Commands =====

#[tauri::command]
pub async fn record_user_action(
    action: UserAction,
    manager: State<'_, Arc<PatternManager>>,
) -> Result<(), String> {
    manager.record_action(action).await;
    Ok(())
}

#[tauri::command]
pub async fn get_next_action_prediction(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<Option<Prediction>, String> {
    Ok(manager.get_prediction().await)
}

#[tauri::command]
pub async fn get_learned_patterns(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<Vec<WorkflowPattern>, String> {
    Ok(manager.get_patterns().await)
}

#[tauri::command]
pub async fn get_patterns_by_tag(
    tag: String,
    manager: State<'_, Arc<PatternManager>>,
) -> Result<Vec<WorkflowPattern>, String> {
    let patterns = manager.get_patterns().await;
    Ok(patterns
        .into_iter()
        .filter(|p| p.tags.contains(&tag))
        .collect())
}

#[tauri::command]
pub async fn get_all_repetitive_tasks(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<Vec<RepetitiveTask>, String> {
    Ok(manager.get_repetitive_tasks().await)
}

#[tauri::command]
pub async fn get_high_priority_repetitive_tasks(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<Vec<RepetitiveTask>, String> {
    Ok(manager.get_high_priority_tasks().await)
}

#[tauri::command]
pub async fn get_pattern_system_stats(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<PatternSystemStats, String> {
    Ok(manager.get_stats().await)
}

#[tauri::command]
pub async fn save_patterns_to_disk(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<(), String> {
    manager.save().await
}

#[tauri::command]
pub async fn clear_pattern_storage(
    manager: State<'_, Arc<PatternManager>>,
) -> Result<(), String> {
    // Clear in-memory state
    {
        let mut learner = manager.learner.lock().await;
        *learner = PatternLearner::new();
    }
    {
        let mut predictor = manager.predictor.lock().await;
        *predictor = ActionPredictor::new();
    }
    {
        let mut detector = manager.detector.lock().await;
        *detector = RepetitionDetector::new();
    }

    // Clear disk storage
    let storage = manager.storage.lock().await;
    storage.clear()?;

    info!("🗑️ Pattern storage cleared");
    Ok(())
}
