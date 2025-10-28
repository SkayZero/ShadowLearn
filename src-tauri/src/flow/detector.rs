/**
 * Flow State Detector
 * Detects user's cognitive flow state for Ambient LED
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::debug;

use crate::context::ContextAggregator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStateData {
    pub flow_state: String, // "deep" | "normal" | "blocked"
    pub confidence: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typing_speed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_score: Option<f64>,
}

/// Detect current flow state based on context
#[tauri::command]
pub async fn detect_flow_state(
    context_aggregator: State<'_, Arc<tokio::sync::Mutex<ContextAggregator>>>,
) -> Result<FlowStateData, String> {
    let mut ctx_guard = context_aggregator.lock().await;
    
    // Get last context
    let ctx = ctx_guard
        .get_last_context()
        .map_err(|e| format!("Failed to get context: {}", e))?;

    // Heuristics for flow state detection
    let flow_state = detect_state(&ctx);
    let confidence = calculate_confidence(&ctx);

    debug!(
        "🎨 Flow state detected: {} (confidence: {:.2})",
        flow_state, confidence
    );

    Ok(FlowStateData {
        flow_state,
        confidence,
        typing_speed: None, // TODO: Implement typing speed tracking
        focus_score: None,  // TODO: Implement focus scoring
    })
}

fn detect_state(ctx: &crate::context::Context) -> String {
    // Deep work: Very short idle time + active app
    if ctx.idle_seconds < 5.0 && !ctx.app.name.is_empty() {
        return "deep".to_string();
    }

    // Blocked: Long idle time (> 30s)
    if ctx.idle_seconds > 30.0 {
        return "blocked".to_string();
    }

    // Normal: Everything else
    "normal".to_string()
}

fn calculate_confidence(ctx: &crate::context::Context) -> f64 {
    // Confidence based on how clear the signal is
    if ctx.idle_seconds < 5.0 {
        0.9 // Very confident about deep work
    } else if ctx.idle_seconds > 60.0 {
        0.95 // Very confident about blocked
    } else {
        0.6 // Medium confidence for normal state
    }
}

