/**
 * Smart Pills / Micro Suggestions Module
 * Generates contextual micro-suggestions
 */

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroSuggestion {
    pub id: String,
    pub text: String,
    pub r#type: String, // "continue" | "help" | "reminder"
}

pub struct PillsManager {
    last_context: Option<crate::context::Context>,
}

impl PillsManager {
    pub fn new() -> Self {
        Self {
            last_context: None,
        }
    }

    pub fn update_context(&mut self, context: crate::context::Context) {
        self.last_context = Some(context);
    }

    pub fn generate_suggestions(&self) -> Vec<MicroSuggestion> {
        let mut suggestions = Vec::new();

        if let Some(context) = &self.last_context {
            // Suggestion based on app
            if context.app.name.contains("Code") || context.app.name.contains("Cursor") {
                suggestions.push(MicroSuggestion {
                    id: format!("code-help-{}", chrono::Utc::now().timestamp()),
                    text: "Besoin d'aide avec ce code ?".to_string(),
                    r#type: "help".to_string(),
                });
            }

            // Suggestion based on idle time
            if context.idle_seconds > 120.0 {
                suggestions.push(MicroSuggestion {
                    id: format!("break-{}", chrono::Utc::now().timestamp()),
                    text: "Prendre une pause ?".to_string(),
                    r#type: "reminder".to_string(),
                });
            }

            // Suggestion to continue work
            if context.idle_seconds < 5.0 {
                suggestions.push(MicroSuggestion {
                    id: format!("continue-{}", chrono::Utc::now().timestamp()),
                    text: "Continue comme ça ! 🚀".to_string(),
                    r#type: "continue".to_string(),
                });
            }
        }

        suggestions
    }
}

#[tauri::command]
pub async fn get_micro_suggestions(
    pills_manager: State<'_, Arc<Mutex<PillsManager>>>,
) -> Result<Vec<MicroSuggestion>, String> {
    let manager = pills_manager.lock().await;
    Ok(manager.generate_suggestions())
}

#[tauri::command]
pub async fn dismiss_pill(
    pill_id: String,
    pills_manager: State<'_, Arc<Mutex<PillsManager>>>,
) -> Result<(), String> {
    tracing::info!("Pill dismissed: {}", pill_id);
    // Could store dismissed pills if needed
    Ok(())
}

