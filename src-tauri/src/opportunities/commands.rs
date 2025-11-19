/**
 * Opportunity Commands
 * Handles user responses to opportunities shown in toast
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::{info, warn};

use crate::learning::LearningSystem;
use crate::triggers::state_machine::{TriggerEvent, TriggerStateMachine};
use crate::triggers::CooldownReason;

#[derive(Debug, Serialize, Deserialize)]
pub struct OpportunityResponse {
    pub opportunity_id: String,
    pub accepted: bool,
    pub timestamp: i64,
}

/// Record user response to an opportunity
#[tauri::command]
pub async fn record_opportunity_response(
    opportunity_id: String,
    accepted: bool,
    learning: State<'_, Arc<tokio::sync::Mutex<LearningSystem>>>,
    state_machine: State<'_, Arc<tokio::sync::Mutex<TriggerStateMachine>>>,
    digest_manager: State<'_, Arc<tokio::sync::Mutex<crate::digest::DigestManager>>>,
) -> Result<(), String> {
    info!(
        "📝 Recording opportunity response: id={}, accepted={}",
        opportunity_id, accepted
    );

    let _timestamp = chrono::Utc::now().timestamp();

    // Record feedback in learning system
    {
        let mut learning_guard = learning.lock().await;
        learning_guard
            .record_feedback(opportunity_id.clone(), accepted)
            .await
            .map_err(|e| {
                warn!("Failed to record feedback in learning system: {}", e);
                format!("Failed to record feedback: {}", e)
            })?;
    }

    // Update state machine based on response
    if !accepted {
        // User dismissed the opportunity -> enter cooldown
        let mut sm = state_machine.lock().await;
        sm.transition(TriggerEvent::EnterCooldown {
            reason: CooldownReason::UserDismissed,
        })
        .map_err(|e| format!("Failed to update state machine: {}", e))?;

        info!("🚫 Opportunity dismissed, entering cooldown");
    } else {
        info!("✅ Opportunity accepted");
        
        // Record accepted suggestion in digest manager
        let mut digest = digest_manager.lock().await;
        digest.record_suggestion_accepted();
    }

    // Store response in persistence layer (optional)
    // TODO: Add to persistence manager if needed

    Ok(())
}

