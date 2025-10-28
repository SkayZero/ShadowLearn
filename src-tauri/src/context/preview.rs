/**
 * Context Preview Commands
 * Provides lightweight context snapshots for UI previews
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use super::ContextAggregator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPreview {
    pub app_name: String,
    pub window_title: String,
    pub idle_seconds: f64,
    pub session_duration_minutes: u64,
    pub recent_screenshots: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

/// Get lightweight context preview
#[tauri::command]
pub async fn get_context_preview(
    context_aggregator: State<'_, Arc<tokio::sync::Mutex<ContextAggregator>>>,
) -> Result<ContextPreview, String> {
    let mut ctx_guard = context_aggregator.lock().await;
    
    let ctx = ctx_guard
        .get_last_context()
        .map_err(|e| format!("Failed to get context: {}", e))?;

    // Calculate session duration (mock for now)
    let session_duration_minutes = if ctx.idle_seconds < 60.0 {
        10 // Active session
    } else {
        0
    };

    Ok(ContextPreview {
        app_name: ctx.app.name.clone(),
        window_title: ctx.app.window_title.clone(),
        idle_seconds: ctx.idle_seconds,
        session_duration_minutes,
        recent_screenshots: 0, // TODO: Track recent screenshots
        pending_suggestion: None, // TODO: Check for pending suggestions
        domain: Some("code".to_string()), // TODO: Detect domain from context
    })
}

