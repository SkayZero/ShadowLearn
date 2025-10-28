/**
 * Message Feedback Handler
 * Records and processes feedback with emotional intelligence
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::learning::LearningSystem;
use crate::triggers::state_machine::{TriggerEvent, TriggerStateMachine};
use crate::triggers::CooldownReason;

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub message_id: String,
    pub helpful: bool,
    pub timestamp: i64,
}

/// Record message feedback (thumbs up/down)
#[tauri::command]
pub async fn record_message_feedback(
    message_id: String,
    helpful: bool,
    learning: State<'_, Arc<tokio::sync::Mutex<LearningSystem>>>,
    state_machine: State<'_, Arc<tokio::sync::Mutex<TriggerStateMachine>>>,
    digest_manager: State<'_, Arc<tokio::sync::Mutex<crate::digest::DigestManager>>>,
) -> Result<(), String> {
    info!(
        "👍👎 Recording message feedback: id={}, helpful={}",
        message_id, helpful
    );

    // Record feedback in learning system
    {
        let mut learning_guard = learning.lock().await;
        learning_guard
            .record_feedback(message_id.clone(), helpful)
            .await
            .map_err(|e| {
                warn!("Failed to record feedback in learning system: {}", e);
                format!("Failed to record feedback: {}", e)
            })?;

        // Adjust confidence weights based on feedback
        learning_guard.adjust_confidence_weights(helpful).await;
    }

    // If negative feedback, slightly increase cooldown
    if !helpful {
        let mut sm = state_machine.lock().await;
        
        // Check if in cooldown state and extend it
        if let Ok(_) = sm.transition(TriggerEvent::EnterCooldown {
            reason: CooldownReason::UserDismissed,
        }) {
            info!("⏸️ Negative feedback: extending cooldown slightly");
        }
    } else {
        info!("✅ Positive feedback recorded");
        
        // Record accepted suggestion in digest manager
        let mut digest = digest_manager.lock().await;
        digest.record_suggestion_accepted();
    }

    Ok(())
}

