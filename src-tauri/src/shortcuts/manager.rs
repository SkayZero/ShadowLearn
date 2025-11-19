use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState, GlobalShortcutExt};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub screenshot_analyze: String,  // Default: "Ctrl+Shift+S"
    pub toggle_bubbles: String,      // Default: "Ctrl+Shift+H"
    pub open_dashboard: String,      // Default: "Ctrl+Shift+D"
    pub dismiss_bubble: String,      // Default: "Escape"
    pub enabled: bool,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            screenshot_analyze: "Ctrl+Shift+S".to_string(),
            toggle_bubbles: "Ctrl+Shift+H".to_string(),
            open_dashboard: "Ctrl+Shift+D".to_string(),
            dismiss_bubble: "Escape".to_string(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShortcutAction {
    ScreenshotAnalyze,
    ToggleBubbles,
    OpenDashboard,
    DismissBubble,
}

pub struct ShortcutManager {
    config: ShortcutConfig,
    registered_shortcuts: Arc<Mutex<HashMap<String, ShortcutAction>>>,
}

impl ShortcutManager {
    pub fn new(config: ShortcutConfig) -> Self {
        Self {
            config,
            registered_shortcuts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register all global shortcuts
    pub async fn register_all(&self, app: &AppHandle) -> Result<(), String> {
        if !self.config.enabled {
            info!("⚠️ Shortcuts disabled in config");
            return Ok(());
        }

        info!("🎹 Registering global shortcuts...");

        // Screenshot + Analyze
        self.register_shortcut(
            app,
            &self.config.screenshot_analyze,
            ShortcutAction::ScreenshotAnalyze,
        )
        .await?;

        // Toggle Bubbles
        self.register_shortcut(
            app,
            &self.config.toggle_bubbles,
            ShortcutAction::ToggleBubbles,
        )
        .await?;

        // Open Dashboard
        self.register_shortcut(
            app,
            &self.config.open_dashboard,
            ShortcutAction::OpenDashboard,
        )
        .await?;

        // Dismiss Bubble (Escape)
        self.register_shortcut(
            app,
            &self.config.dismiss_bubble,
            ShortcutAction::DismissBubble,
        )
        .await?;

        info!("✅ All shortcuts registered successfully");
        Ok(())
    }

    /// Register a single shortcut
    async fn register_shortcut(
        &self,
        app: &AppHandle,
        shortcut: &str,
        action: ShortcutAction,
    ) -> Result<(), String> {
        let shortcut_parsed = shortcut
            .parse()
            .map_err(|e| format!("Invalid shortcut '{}': {:?}", shortcut, e))?;

        let app_handle = app.clone();
        let action_clone = action.clone();

        app.global_shortcut()
            .on_shortcut(shortcut_parsed, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    info!("🎹 Shortcut triggered: {:?}", action_clone);

                    // Emit event to frontend
                    if let Err(e) = app_handle.emit("shortcut-triggered", &action_clone) {
                        error!("Failed to emit shortcut event: {}", e);
                    }
                }
            })
            .map_err(|e| format!("Failed to register shortcut '{}': {}", shortcut, e))?;

        // Store in registry
        let mut registry = self.registered_shortcuts.lock().await;
        registry.insert(shortcut.to_string(), action.clone());

        info!("✅ Registered shortcut: {} → {:?}", shortcut, action);
        Ok(())
    }

    /// Unregister all shortcuts
    pub async fn unregister_all(&self, app: &AppHandle) -> Result<(), String> {
        info!("🎹 Unregistering all shortcuts...");

        let registry = self.registered_shortcuts.lock().await;

        for shortcut in registry.keys() {
            let shortcut_parsed = shortcut
                .parse()
                .map_err(|e| format!("Invalid shortcut '{}': {:?}", shortcut, e))?;

            if let Err(e) = app.global_shortcut().unregister(shortcut_parsed) {
                warn!("⚠️ Failed to unregister '{}': {}", shortcut, e);
            } else {
                info!("✅ Unregistered: {}", shortcut);
            }
        }

        info!("✅ All shortcuts unregistered");
        Ok(())
    }

    /// Get current configuration
    pub fn config(&self) -> &ShortcutConfig {
        &self.config
    }

    /// Get list of registered shortcuts
    pub async fn list_shortcuts(&self) -> HashMap<String, ShortcutAction> {
        self.registered_shortcuts.lock().await.clone()
    }
}

impl Default for ShortcutManager {
    fn default() -> Self {
        Self::new(ShortcutConfig::default())
    }
}
