use super::manager::{ShortcutAction, ShortcutConfig, ShortcutManager};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;
use tracing::info;

/// Get shortcut configuration
#[tauri::command]
pub async fn get_shortcuts_config(
    shortcuts: tauri::State<'_, Arc<Mutex<ShortcutManager>>>,
) -> Result<ShortcutConfig, String> {
    let manager = shortcuts.lock().await;
    Ok(manager.config().clone())
}

/// List all registered shortcuts
#[tauri::command]
pub async fn list_shortcuts(
    shortcuts: tauri::State<'_, Arc<Mutex<ShortcutManager>>>,
) -> Result<HashMap<String, ShortcutAction>, String> {
    let manager = shortcuts.lock().await;
    Ok(manager.list_shortcuts().await)
}

/// Trigger a specific action manually (for testing or UI buttons)
#[tauri::command]
pub async fn trigger_shortcut_action(
    app: tauri::AppHandle,
    action: ShortcutAction,
) -> Result<(), String> {
    info!("🎹 Manually triggering shortcut action: {:?}", action);

    // Emit event to frontend (same as keyboard shortcut)
    app.emit("shortcut-triggered", &action)
        .map_err(|e| format!("Failed to emit shortcut event: {}", e))?;

    Ok(())
}
