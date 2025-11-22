use super::manager::{ShortcutAction, ShortcutConfig, ShortcutManager};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, PhysicalSize, Size};
use tokio::sync::Mutex;
use tracing::{error, info};

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

    info!("🔍 [toggle_spotlight] Starting...");

    if let Some(spotlight_window) = app.get_webview_window("spotlight") {
        let is_visible = spotlight_window.is_visible().unwrap_or(false);
        info!("🔍 [toggle_spotlight] Current visibility: {}", is_visible);

        if is_visible {
            info!("🔍 [toggle_spotlight] Hiding window...");
            if let Err(e) = spotlight_window.hide() {
                error!("❌ [toggle_spotlight] Failed to hide: {}", e);
                return Err(format!("Failed to hide spotlight: {}", e));
            }
            info!("✅ [toggle_spotlight] Hidden successfully");
            Ok(false)
        } else {
            info!("🔍 [toggle_spotlight] Showing window...");

            // Step 1: Force size to 1200×900
            if let Err(e) = spotlight_window.set_size(Size::Physical(PhysicalSize {
                width: 1200,
                height: 900,
            })) {
                error!("❌ [toggle_spotlight] Failed to set size: {}", e);
            } else {
                info!("📐 [toggle_spotlight] Size forced to 1200×900");
            }

            // Step 2: Center the window
            if let Err(e) = spotlight_window.center() {
                error!("❌ [toggle_spotlight] Failed to center: {}", e);
            } else {
                info!("📍 [toggle_spotlight] Window centered");
            }

            // Step 3: Show
            if let Err(e) = spotlight_window.show() {
                error!("❌ [toggle_spotlight] Failed to show: {}", e);
                return Err(format!("Failed to show spotlight: {}", e));
            }
            info!("✅ [toggle_spotlight] Show called successfully");

            // Step 4: Set always on top
            if let Err(e) = spotlight_window.set_always_on_top(true) {
                error!("❌ [toggle_spotlight] Failed to set always on top: {}", e);
            } else {
                info!("✅ [toggle_spotlight] Always on top set");
            }

            // Step 5: Focus
            if let Err(e) = spotlight_window.set_focus() {
                error!("❌ [toggle_spotlight] Failed to focus: {}", e);
            } else {
                info!("✅ [toggle_spotlight] Focused successfully");
            }

            // Step 6: Emit event
            if let Err(e) = app.emit("spotlight:show", ()) {
                error!("❌ [toggle_spotlight] Failed to emit event: {}", e);
            } else {
                info!("✅ [toggle_spotlight] Event emitted");
            }

            info!("✅ [toggle_spotlight] Complete - window should be visible");
            Ok(true)
        }
    } else {
        error!("❌ [toggle_spotlight] Spotlight window not found!");
        Err("Spotlight window not found".to_string())
    }
}
