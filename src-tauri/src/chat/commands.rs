use crate::chat::llm_client::{ChatMessage, ChatRequest, LLMChatClient};
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

/// Send a chat message to the LLM
#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    include_context: bool,
    llm_client: tauri::State<'_, Arc<tokio::sync::Mutex<LLMChatClient>>>,
) -> Result<String, String> {
    let start = Instant::now();
    let _timestamp = Utc::now().to_rfc3339();

    info!("💬 Chat request: {}", message);
    tracing::info!("Building ChatRequest...");

    // Build request
    let request = ChatRequest {
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: message,
        }],
        include_context,
        temperature: 0.7,
    };

    // Call LLM
    tracing::info!("Calling LLM client...");
    let response = {
        let client = llm_client.lock().await;
        let result = client.chat(request).await;
        tracing::info!("LLM response received");
        result
    };

    let ttfr_ms = start.elapsed().as_millis() as u64;
    let (ok, content, provider, used_fallback) = match response {
        Ok(resp) => (true, resp.content, resp.provider, resp.used_fallback),
        Err(e) => (false, e.clone(), "unknown".to_string(), false),
    };

    // Log to console (jsonl disabled to avoid rebuild loops)
    info!("📊 Chat metrics: provider={}, used_fallback={}, ttfr_ms={}, ok={}", provider, used_fallback, ttfr_ms, ok);
    
    // Note: JSONL logging disabled because it triggers Vite HMR rebuild
    // To re-enable, use a file path outside the project root or in tmp/

    if ok {
        Ok(content)
    } else {
        // Return user-friendly error
        Err(format!(
            "Cloud indisponible. Essayez Ollama (modèle local). Erreur: {}",
            content
        ))
    }
}

/// Log an event to events.jsonl
fn log_to_jsonl(entry: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = "src-tauri/events.jsonl";
    let line = serde_json::to_string(entry)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    
    writeln!(file, "{}", line)?;
    Ok(())
}

/// Get LLM stats
#[tauri::command]
pub async fn get_llm_stats(
    llm_client: tauri::State<'_, Arc<tokio::sync::Mutex<LLMChatClient>>>,
) -> std::result::Result<crate::chat::llm_client::LLMClientStats, String> {
    let client = llm_client.lock().await;
    let stats = client.get_stats().await;
    Ok(stats)
}

/// Check LLM health
#[tauri::command]
pub async fn check_llm_health(
    llm_client: tauri::State<'_, Arc<tokio::sync::Mutex<LLMChatClient>>>,
) -> Result<(bool, bool), String> {
    let client = llm_client.lock().await;
    Ok(client.check_health().await)
}

