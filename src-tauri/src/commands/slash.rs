/**
 * Slash Commands Module
 * Handles execution of slash commands from chat input
 */

use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[tauri::command]
pub async fn execute_slash_command(
    command: String,
    context: String,
    llm_client: State<'_, Arc<tokio::sync::Mutex<crate::chat::LLMChatClient>>>,
) -> Result<SlashCommandResult, String> {
    info!("Executing slash command: {} with context: {}", command, context);

    match command.as_str() {
        "help" => {
            Ok(SlashCommandResult {
                success: true,
                message: "Commandes disponibles:\n/explain - Expliquer un concept\n/resume - Résumer du texte\n/debug - Analyser une erreur\n/improve - Suggérer des améliorations\n/translate - Traduire du texte".to_string(),
                data: None,
            })
        }
        "explain" => {
            let prompt = format!("Explique clairement et simplement : {}", context);
            let request = crate::chat::ChatRequest {
                messages: vec![crate::chat::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                include_context: false,
                temperature: 0.7,
            };
            let response = llm_client
                .lock()
                .await
                .chat(request)
                .await
                .map_err(|e| format!("Failed to chat with LLM: {}", e))?;

            Ok(SlashCommandResult {
                success: true,
                message: response.content,
                data: None,
            })
        }
        "resume" => {
            let prompt = format!("Résume ce texte de manière concise : {}", context);
            let request = crate::chat::ChatRequest {
                messages: vec![crate::chat::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                include_context: false,
                temperature: 0.7,
            };
            let response = llm_client
                .lock()
                .await
                .chat(request)
                .await
                .map_err(|e| format!("Failed to chat with LLM: {}", e))?;

            Ok(SlashCommandResult {
                success: true,
                message: response.content,
                data: None,
            })
        }
        "debug" => {
            let prompt = format!("Analyse cette erreur et propose une solution : {}", context);
            let request = crate::chat::ChatRequest {
                messages: vec![crate::chat::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                include_context: false,
                temperature: 0.7,
            };
            let response = llm_client
                .lock()
                .await
                .chat(request)
                .await
                .map_err(|e| format!("Failed to chat with LLM: {}", e))?;

            Ok(SlashCommandResult {
                success: true,
                message: response.content,
                data: None,
            })
        }
        "improve" => {
            let prompt = format!("Suggère des améliorations pour : {}", context);
            let request = crate::chat::ChatRequest {
                messages: vec![crate::chat::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                include_context: false,
                temperature: 0.7,
            };
            let response = llm_client
                .lock()
                .await
                .chat(request)
                .await
                .map_err(|e| format!("Failed to chat with LLM: {}", e))?;

            Ok(SlashCommandResult {
                success: true,
                message: response.content,
                data: None,
            })
        }
        "translate" => {
            let prompt = format!("Traduis ce texte en français (ou en anglais s'il est déjà en français) : {}", context);
            let request = crate::chat::ChatRequest {
                messages: vec![crate::chat::ChatMessage {
                    role: "user".to_string(),
                    content: prompt,
                }],
                include_context: false,
                temperature: 0.7,
            };
            let response = llm_client
                .lock()
                .await
                .chat(request)
                .await
                .map_err(|e| format!("Failed to chat with LLM: {}", e))?;

            Ok(SlashCommandResult {
                success: true,
                message: response.content,
                data: None,
            })
        }
        _ => {
            Ok(SlashCommandResult {
                success: false,
                message: format!("Commande inconnue: /{}", command),
                data: None,
            })
        }
    }
}

