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

/// Toggle Spotlight window visibility (for testing and HUD button)
#[tauri::command]
pub async fn toggle_spotlight(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri::Manager;

    info!("🔍 Manually toggling Spotlight window");

    if let Some(spotlight_window) = app.get_webview_window("spotlight") {
        let is_visible = spotlight_window.is_visible().unwrap_or(false);

        if is_visible {
            info!("🔍 Hiding Spotlight window");
            spotlight_window.hide()
                .map_err(|e| format!("Failed to hide spotlight: {}", e))?;
            Ok(false)
        } else {
            info!("🔍 Showing Spotlight window");
            spotlight_window.show()
                .map_err(|e| format!("Failed to show spotlight: {}", e))?;
            spotlight_window.set_focus()
                .map_err(|e| format!("Failed to focus spotlight: {}", e))?;

            // Emit event to tell Spotlight frontend to show content
            app.emit("spotlight:show", ())
                .map_err(|e| format!("Failed to emit spotlight:show: {}", e))?;

            Ok(true)
        }
    } else {
        Err("Spotlight window not found".to_string())
    }
}
