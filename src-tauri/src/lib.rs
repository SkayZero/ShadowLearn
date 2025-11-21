use std::sync::Arc;
use tauri::{AppHandle, Emitter, Listener, Manager, PhysicalPosition};
use tauri_plugin_window_state::StateFlags;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

mod adaptive;
mod chat;
mod commands; // Clueless: Slash Commands
mod config; // J5
mod crypto;
mod permissions;
mod privacy; // Privacy zones for screen monitoring
mod artefact;
mod clustering;
mod context;
mod digest; // Clueless: Daily Digest
mod features;
mod flow; // Clueless: Flow State Detection
mod focus; // Killer Feature: Focus Mode
mod learn; // Killer Feature: Learn by Doing
mod productivity; // Phase 3: Productivity Dashboard & Weekly Insights
mod health;
mod intent;
mod learning;
mod ml;
mod monitor; // Screen Monitoring avec détection de changements
mod shortcuts; // Global keyboard shortcuts
mod opportunities; // Clueless: One-Tap Toast
mod pause; // Clueless Phase 3: Pause Mode
mod patterns; // Phase 2.1: Pattern Recognition ML
mod persistence;
mod personality; // Clueless Phase 3: Personalities
mod pills; // Clueless: Smart Pills / Micro Suggestions
mod plugins; // Phase 4: Plugin System
mod recovery;
mod replay; // Killer Feature: Shadow Replay
mod streaks; // Clueless Phase 3: Streaks
mod screenshot;
mod snooze;
mod telemetry;
mod triggers;
mod validator;

use context::{Context, ContextAggregator};
use features::{Feature, FeatureFlags, FeaturesState};
use health::{HealthMonitor, HealthStatus};
use ml::{
    EventType as MLEventType, PersonalizationManager, SmartSuggestions, UsagePatterns, UserEvent,
    UserResponse,
};
use persistence::{
    CapturedContext, Conversation, Message, MessageRole, PersistenceManager, PersistenceStats,
};
use recovery::{RecoveryManager, RecoveryStats};
use snooze::{SnoozeDuration, SnoozeManager};
use telemetry::{EventType, Telemetry, TelemetryEvent, TelemetryStats};
use triggers::manager::ExtendedTriggerStats;
use triggers::{TriggerDecision, TriggerManager, TriggerStats};

#[tauri::command]
async fn toggle_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        let visible = window.is_visible().unwrap_or(false);
        if visible {
            window.hide().map_err(|e| e.to_string())?;
            info!("🔒 Window '{}' hidden", label);
        } else {
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().ok();
            info!("✅ Window '{}' shown", label);
        }
        Ok(())
    } else {
        warn!("⚠️ Window '{}' not found", label);
        Err(format!("Window '{}' not found", label))
    }
}

#[tauri::command]
async fn ensure_chat_visible(app: AppHandle) -> Result<(), String> {
    info!("🔍 ensure_chat_visible called");
    if let Some(window) = app.get_webview_window("chat") {
        info!("📍 Chat window found, checking visibility...");
        let is_visible = window.is_visible().unwrap_or(false);
        info!("👁️ Chat window visibility before: {}", is_visible);

        // Force window to stay visible and focused (without always_on_top)
        window.show().map_err(|e| {
            error!("❌ Failed to show window: {}", e);
            e.to_string()
        })?;
        info!("✅ Window.show() succeeded");

        window.set_focus().map_err(|e| {
            error!("❌ Failed to set focus: {}", e);
            e.to_string()
        })?;
        info!("✅ Window.set_focus() succeeded");

        // Unminimize if minimized
        window.unminimize().ok();
        info!("✅ Window.unminimize() called");

        let is_visible_after = window.is_visible().unwrap_or(false);
        info!("👁️ Chat window visibility after: {}", is_visible_after);

        info!("💬 Chat window ensured visible and focused");
        Ok(())
    } else {
        error!("⚠️ Chat window not found in app");
        Err("Chat window not found".to_string())
    }
}

#[tauri::command]
fn broadcast_event(app: AppHandle, event: String, payload: String) -> Result<(), String> {
    app.emit(&event, payload).map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_health_status(
    health_monitor: tauri::State<'_, Arc<HealthMonitor>>,
    telemetry: tauri::State<'_, Arc<Telemetry>>,
) -> Result<HealthStatus, String> {
    let start = std::time::Instant::now();
    let result = health_monitor.check_health().await;
    let duration_ms = start.elapsed().as_millis() as u64;

    // Record telemetry event
    telemetry.record_event(TelemetryEvent::new(EventType::HealthCheck).with_duration(duration_ms));

    Ok(result)
}

#[tauri::command]
fn get_telemetry_stats(
    telemetry: tauri::State<'_, Arc<Telemetry>>,
) -> Result<TelemetryStats, String> {
    Ok(telemetry.get_stats())
}

#[tauri::command]
fn get_recovery_stats(
    recovery: tauri::State<'_, Arc<RecoveryManager>>,
) -> Result<RecoveryStats, String> {
    Ok(recovery.get_stats())
}

#[tauri::command]
fn record_telemetry_event(
    telemetry: tauri::State<'_, Arc<Telemetry>>,
    event_type: String,
    duration_ms: Option<u64>,
) -> Result<(), String> {
    let event_type = match event_type.as_str() {
        "idle_check" => EventType::IdleCheck,
        "screenshot_capture" => EventType::ScreenshotCapture,
        "health_check" => EventType::HealthCheck,
        "component_restart" => EventType::ComponentRestart,
        "window_toggle" => EventType::WindowToggle,
        "message_sent" => EventType::MessageSent,
        _ => return Err(format!("Unknown event type: {}", event_type)),
    };

    let mut event = TelemetryEvent::new(event_type);
    if let Some(duration) = duration_ms {
        event = event.with_duration(duration);
    }

    telemetry.record_event(event);
    Ok(())
}

#[tauri::command]
fn get_features_state(flags: tauri::State<'_, Arc<FeatureFlags>>) -> FeaturesState {
    flags.get_state()
}

// ========== TRIGGER COMMANDS ==========

#[tauri::command]
async fn start_trigger_loop(app: AppHandle) -> Result<(), String> {
    info!("🔄 Starting trigger loop from command...");
    triggers::trigger_loop::start_trigger_loop(app).await;
    Ok(())
}

#[tauri::command]
async fn check_should_trigger(
    context_aggregator: tauri::State<'_, Arc<Mutex<ContextAggregator>>>,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<TriggerDecision, String> {
    let ctx = context_aggregator
        .lock()
        .await
        .capture()
        .await
        .map_err(|e| e.to_string())?;
    let decision = trigger_manager.lock().await.should_trigger(&ctx);
    Ok(decision)
}

#[tauri::command]
async fn record_trigger_fired(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.record_trigger(&app_name);
    Ok(())
}

#[tauri::command]
async fn record_bubble_dismissed(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.record_dismiss();
    Ok(())
}

#[tauri::command]
async fn record_trigger_action(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.record_action();
    Ok(())
}

#[tauri::command]
async fn get_trigger_stats(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<TriggerStats, String> {
    Ok(trigger_manager.lock().await.get_stats())
}

#[tauri::command]
async fn add_to_allowlist(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.add_to_allowlist(app_name);
    Ok(())
}

#[tauri::command]
async fn remove_from_allowlist(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager
        .lock()
        .await
        .remove_from_allowlist(&app_name);
    Ok(())
}

// ========== PHASE 3A: OPPORTUNITY MOCK TRIGGER (DEBUG) ==========

#[tauri::command]
async fn trigger_mock_opportunity(
    app: AppHandle,
    opportunity_type: String,
) -> Result<(), String> {
    use chrono::Utc;
    use serde_json::json;

    info!("🧪 Triggering mock opportunity: {}", opportunity_type);

    let timestamp = Utc::now().timestamp();
    let mock_id = format!("mock_{}_{}", opportunity_type, timestamp);

    // Create mock opportunity based on type
    let mock_data = match opportunity_type.as_str() {
        "refacto" => json!({
            "id": mock_id,
            "title": "Code répété détecté",
            "description": "Tu utilises le même pattern 3 fois dans UserService.ts. Veux-tu créer une fonction réutilisable ?",
            "context": {
                "app": "VS Code",
                "file": "src/services/UserService.ts",
                "line": 42,
                "codeSnippet": "const user = await db.users.findOne({ id });\nif (!user) throw new Error('Not found');"
            },
            "type": "refacto",
            "confidence": 0.85,
            "timestamp": timestamp,
            "status": "pending",
            "actions": [
                {
                    "id": "discuss",
                    "label": "Discuter",
                    "icon": "💬",
                    "type": "discuss"
                },
                {
                    "id": "view",
                    "label": "Voir",
                    "icon": "👁",
                    "type": "view"
                },
                {
                    "id": "ignore",
                    "label": "Ignorer",
                    "icon": "🚫",
                    "type": "ignore"
                }
            ]
        }),
        "debug" => json!({
            "id": mock_id,
            "title": "Erreur persistante détectée",
            "description": "TypeError sur la ligne 42 depuis 90 secondes. Besoin d'aide pour déboguer ?",
            "context": {
                "app": "VS Code",
                "file": "src/components/Dashboard.tsx",
                "line": 42,
                "codeSnippet": "const data = response.data.map(item => item.value);"
            },
            "type": "debug",
            "confidence": 0.92,
            "timestamp": timestamp,
            "status": "pending",
            "actions": [
                {
                    "id": "discuss",
                    "label": "Discuter",
                    "icon": "💬",
                    "type": "discuss"
                },
                {
                    "id": "view",
                    "label": "Voir",
                    "icon": "👁",
                    "type": "view"
                },
                {
                    "id": "ignore",
                    "label": "Ignorer",
                    "icon": "🚫",
                    "type": "ignore"
                }
            ]
        }),
        "learn" => json!({
            "id": mock_id,
            "title": "Opportunité d'apprentissage",
            "description": "Tu viens d'utiliser une technique avancée de React Hooks. Veux-tu en savoir plus ?",
            "context": {
                "app": "VS Code",
                "file": "src/hooks/useCustomHook.ts",
                "line": 15
            },
            "type": "learn",
            "confidence": 0.78,
            "timestamp": timestamp,
            "status": "pending",
            "actions": [
                {
                    "id": "discuss",
                    "label": "Discuter",
                    "icon": "💬",
                    "type": "discuss"
                },
                {
                    "id": "view",
                    "label": "Voir",
                    "icon": "👁",
                    "type": "view"
                },
                {
                    "id": "ignore",
                    "label": "Ignorer",
                    "icon": "🚫",
                    "type": "ignore"
                }
            ]
        }),
        "tip" => json!({
            "id": mock_id,
            "title": "Raccourci clavier disponible",
            "description": "Savais-tu que Cmd+D peut dupliquer une ligne ? Ça pourrait t'aider ici.",
            "context": {
                "app": "VS Code",
                "file": "src/utils/helpers.ts",
                "line": 8
            },
            "type": "tip",
            "confidence": 0.65,
            "timestamp": timestamp,
            "status": "pending",
            "actions": [
                {
                    "id": "view",
                    "label": "Voir",
                    "icon": "👁",
                    "type": "view"
                },
                {
                    "id": "ignore",
                    "label": "Ignorer",
                    "icon": "🚫",
                    "type": "ignore"
                }
            ]
        }),
        _ => {
            warn!("⚠️ Unknown opportunity type: {}", opportunity_type);
            return Err(format!("Unknown opportunity type: {}", opportunity_type));
        }
    };

    // Emit opportunity:new event
    app.emit("opportunity:new", mock_data.clone())
        .map_err(|e| format!("Failed to emit opportunity:new: {}", e))?;

    info!("✅ Mock opportunity emitted: {}", mock_id);

    // Also emit hud:pulse event to trigger HUD animation
    app.emit("hud:pulse", json!({ "state": "opportunity" }))
        .map_err(|e| format!("Failed to emit hud:pulse: {}", e))?;

    info!("✅ HUD pulse event emitted");

    Ok(())
}

// ========== J17: PERSISTANCE & MÉMOIRE COMMANDS ==========

#[tauri::command]
async fn create_conversation(
    title: String,
    app_context: Option<String>,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<Conversation, String> {
    let manager = persistence_manager.lock().await;
    manager.create_conversation(title, app_context).await
}

#[tauri::command]
async fn save_message(
    conversation_id: String,
    role: String, // "user" | "assistant" | "system"
    content: String,
    metadata: Option<String>,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<Message, String> {
    let message_role = match role.as_str() {
        "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        _ => return Err(format!("Invalid message role: {}", role)),
    };

    let manager = persistence_manager.lock().await;
    manager
        .save_message(&conversation_id, message_role, content, metadata)
        .await
}

#[tauri::command]
async fn get_recent_conversations(
    limit: i32,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<Vec<Conversation>, String> {
    let manager = persistence_manager.lock().await;
    manager.get_recent_conversations(limit).await
}

#[tauri::command]
async fn get_conversation_messages(
    conversation_id: String,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<Vec<Message>, String> {
    let manager = persistence_manager.lock().await;
    manager.get_conversation_messages(&conversation_id).await
}

#[tauri::command]
async fn get_persistence_stats(
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<PersistenceStats, String> {
    let manager = persistence_manager.lock().await;
    manager.get_stats().await
}

#[tauri::command]
async fn export_data(
    file_path: String,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<(), String> {
    let manager = persistence_manager.lock().await;
    manager.export_data(&file_path).await
}

#[tauri::command]
async fn get_recent_contexts_for_app(
    app_name: String,
    limit: i32,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<Vec<CapturedContext>, String> {
    let manager = persistence_manager.lock().await;
    manager.get_recent_contexts_for_app(&app_name, limit).await
}

#[tauri::command]
async fn save_context(
    context: CapturedContext,
    persistence_manager: tauri::State<'_, Arc<Mutex<PersistenceManager>>>,
) -> Result<(), String> {
    let manager = persistence_manager.lock().await;
    manager.save_context(context).await
}

// ========== J16: DÉCLENCHEMENT DISCRET COMMANDS ==========
#[tauri::command]
async fn set_bubble_visible(
    visible: bool,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.set_bubble_visible(visible);
    Ok(())
}

#[tauri::command]
async fn record_user_interaction(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.record_interaction();
    Ok(())
}

#[tauri::command]
async fn is_interaction_locked(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<bool, String> {
    Ok(trigger_manager.lock().await.is_interaction_locked())
}

#[tauri::command]
async fn get_interaction_lock_remaining(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<Option<u64>, String> {
    Ok(trigger_manager
        .lock()
        .await
        .get_interaction_lock_remaining()
        .map(|d| d.as_millis() as u64))
}

// ========== J16: ANTI-SPAM & UX COMMANDS ==========

#[tauri::command]
async fn record_trigger_ignored(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager
        .lock()
        .await
        .record_ignored_trigger(&app_name);
    Ok(())
}

#[tauri::command]
async fn mute_app(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.mute_app(&app_name);
    Ok(())
}

#[tauri::command]
async fn unmute_app(
    app_name: String,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.unmute_app(&app_name);
    Ok(())
}

#[tauri::command]
async fn record_snooze_used(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    trigger_manager.lock().await.record_snooze();
    Ok(())
}

#[tauri::command]
async fn get_extended_trigger_stats(
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<ExtendedTriggerStats, String> {
    let mut manager = trigger_manager.lock().await;
    manager.cleanup_expired_mutes(); // Cleanup avant de retourner les stats
    Ok(manager.get_extended_stats())
}

#[tauri::command]
fn toggle_feature(
    flags: tauri::State<'_, Arc<FeatureFlags>>,
    feature: Feature,
    enabled: bool,
) -> Result<(), String> {
    if enabled {
        if flags.can_enable(feature) {
            flags.enable(feature);
            Ok(())
        } else {
            Err(format!(
                "Cannot enable {}: dependencies not satisfied",
                feature.display_name()
            ))
        }
    } else {
        flags.disable(feature);
        Ok(())
    }
}

#[tauri::command]
async fn capture_context(
    context_aggregator: tauri::State<'_, Arc<Mutex<ContextAggregator>>>,
) -> Result<Context, String> {
    context_aggregator
        .lock()
        .await
        .capture()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn reset_user_activity(
    context_aggregator: tauri::State<'_, Arc<Mutex<ContextAggregator>>>,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
    activity_type: String, // "keyboard", "mouse", "scroll"
) -> Result<(), String> {
    use context::ActivityType;

    let act_type = match activity_type.to_lowercase().as_str() {
        "keyboard" => ActivityType::Keyboard,
        "mouse" => ActivityType::Mouse,
        "scroll" => ActivityType::Scroll,
        _ => ActivityType::Unknown,
    };

    // Reset idle timer
    context_aggregator
        .lock()
        .await
        .reset_user_activity(act_type);

    // Reset debounce
    trigger_manager.lock().await.reset_debounce();

    Ok(())
}

#[tauri::command]
async fn get_idle_state(
    context_aggregator: tauri::State<'_, Arc<Mutex<ContextAggregator>>>,
) -> Result<context::IdleState, String> {
    Ok(context_aggregator.lock().await.get_idle_state())
}

// ========== SNOOZE COMMANDS ==========

#[tauri::command]
async fn snooze_triggers(
    snooze_manager: tauri::State<'_, Arc<Mutex<SnoozeManager>>>,
    duration: String, // "30min" | "2h" | "today"
) -> Result<(), String> {
    let snooze_duration = match duration.as_str() {
        "30min" => SnoozeDuration::ThirtyMinutes,
        "2h" => SnoozeDuration::TwoHours,
        "today" => SnoozeDuration::UntilToday,
        _ => return Err(format!("Invalid snooze duration: {}", duration)),
    };

    snooze_manager.lock().await.snooze(snooze_duration)
}

#[tauri::command]
async fn unsnooze_triggers(
    snooze_manager: tauri::State<'_, Arc<Mutex<SnoozeManager>>>,
) -> Result<(), String> {
    snooze_manager.lock().await.unsnooze()
}

#[tauri::command]
async fn get_snooze_status(
    snooze_manager: tauri::State<'_, Arc<Mutex<SnoozeManager>>>,
) -> Result<Option<u64>, String> {
    Ok(snooze_manager.lock().await.get_snooze_until())
}

// ========== WINDOW MANAGEMENT COMMANDS ==========

#[tauri::command]
async fn show_window(app_handle: tauri::AppHandle, window_label: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window
            .show()
            .map_err(|e| format!("Failed to show window: {}", e))?;
        window
            .set_focus()
            .map_err(|e| format!("Failed to focus window: {}", e))?;
        info!("✅ Window '{}' shown and focused", window_label);
    } else {
        return Err(format!("Window '{}' not found", window_label));
    }

    Ok(())
}

#[tauri::command]
async fn hide_window(app_handle: tauri::AppHandle, window_label: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window
            .hide()
            .map_err(|e| format!("Failed to hide window: {}", e))?;
        info!("✅ Window '{}' hidden", window_label);
    } else {
        return Err(format!("Window '{}' not found", window_label));
    }

    Ok(())
}

#[tauri::command]
async fn minimize_window(app_handle: tauri::AppHandle, window_label: String) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        window
            .minimize()
            .map_err(|e| format!("Failed to minimize window: {}", e))?;
        info!("✅ Window '{}' minimized", window_label);
    } else {
        return Err(format!("Window '{}' not found", window_label));
    }

    Ok(())
}

#[tauri::command]
async fn is_window_visible(
    app_handle: tauri::AppHandle,
    window_label: String,
) -> Result<bool, String> {
    if let Some(window) = app_handle.get_webview_window(&window_label) {
        Ok(window.is_visible().unwrap_or(false))
    } else {
        Err(format!("Window '{}' not found", window_label))
    }
}

// ========== J19: LEARNING SYSTEM COMMANDS ==========

#[tauri::command]
async fn record_user_feedback(
    suggestion_id: String,
    helpful: bool,
    used: bool,
    reverted: bool,
    time_to_flow_ms: Option<i64>,
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
    context_aggregator: tauri::State<'_, Arc<Mutex<context::aggregator::ContextAggregator>>>,
) -> Result<f32, String> {
    let outcome = if used {
        learning::reward::Outcome::Used {
            helpful,
            reverted,
            time_to_flow: time_to_flow_ms.map(|ms| std::time::Duration::from_millis(ms as u64)),
        }
    } else {
        learning::reward::Outcome::Ignored
    };

    // Obtenir le contexte actuel
    let context = {
        let mut aggregator = context_aggregator.lock().await;
        aggregator
            .peek()
            .map_err(|e| format!("Failed to get context: {}", e))?
    };

    // Créer un contexte complet pour l'apprentissage
    let full_context = context::aggregator::Context {
        id: uuid::Uuid::new_v4().to_string(),
        app: context.app,
        clipboard: None,
        idle_seconds: context.idle_seconds,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        capture_duration_ms: 0,
    };

    let mut system = learning_system.lock().await;
    let reward = system
        .record_outcome(&suggestion_id, &full_context, "suggestion", outcome)
        .await?;

    info!(
        "User feedback recorded: suggestion={}, reward={:.3}",
        suggestion_id, reward
    );
    Ok(reward)
}

#[tauri::command]
async fn get_user_trust_level(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<learning::trust::TrustLevel, String> {
    let system = learning_system.lock().await;
    system.get_trust_level().await
}

#[tauri::command]
async fn get_trust_recommendations(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<learning::TrustRecommendations, String> {
    let system = learning_system.lock().await;
    system.get_trust_recommendations().await
}

#[tauri::command]
async fn reset_user_trust(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<(), String> {
    let mut system = learning_system.lock().await;
    system.reset_trust().await?;
    info!("User trust reset successfully");
    Ok(())
}

// ========== J18: PERSONNALISATION ML COMMANDS ==========

#[tauri::command]
async fn record_ml_event(
    event_type: String, // "trigger_fired" | "trigger_accepted" | "trigger_ignored" | etc.
    app_name: String,
    context: Option<String>,
    user_response: Option<String>, // "accepted" | "ignored" | "dismissed" | "snoozed"
    personalization_manager: tauri::State<'_, Arc<Mutex<PersonalizationManager>>>,
) -> Result<(), String> {
    let ml_event_type = match event_type.as_str() {
        "trigger_fired" => MLEventType::TriggerFired,
        "trigger_accepted" => MLEventType::TriggerAccepted,
        "trigger_ignored" => MLEventType::TriggerIgnored,
        "trigger_dismissed" => MLEventType::TriggerDismissed,
        "app_muted" => MLEventType::AppMuted,
        "app_unmuted" => MLEventType::AppUnmuted,
        "clipboard_changed" => MLEventType::ClipboardChanged,
        "idle_detected" => MLEventType::IdleDetected,
        _ => return Err(format!("Invalid event type: {}", event_type)),
    };

    let ml_user_response = if let Some(response) = user_response {
        match response.as_str() {
            "accepted" => Some(UserResponse::Accepted),
            "ignored" => Some(UserResponse::Ignored),
            "dismissed" => Some(UserResponse::Dismissed),
            "snoozed" => Some(UserResponse::Snoozed),
            _ => return Err(format!("Invalid user response: {}", response)),
        }
    } else {
        None
    };

    let event = UserEvent {
        timestamp: chrono::Utc::now(),
        event_type: ml_event_type,
        app_name,
        context,
        user_response: ml_user_response,
    };

    personalization_manager.lock().await.record_event(event);
    Ok(())
}

#[tauri::command]
async fn get_usage_patterns(
    personalization_manager: tauri::State<'_, Arc<Mutex<PersonalizationManager>>>,
) -> Result<UsagePatterns, String> {
    Ok(personalization_manager.lock().await.get_patterns().clone())
}

#[tauri::command]
async fn get_smart_suggestions(
    personalization_manager: tauri::State<'_, Arc<Mutex<PersonalizationManager>>>,
) -> Result<SmartSuggestions, String> {
    Ok(personalization_manager.lock().await.generate_suggestions())
}

#[tauri::command]
async fn apply_smart_suggestions(
    suggestions: SmartSuggestions,
    trigger_manager: tauri::State<'_, Arc<Mutex<TriggerManager>>>,
) -> Result<(), String> {
    let mut manager = trigger_manager.lock().await;

    // Appliquer les apps recommandées à l'allowlist
    for app in suggestions.recommended_apps {
        manager.add_to_allowlist(app);
    }

    // Muter les apps recommandées pour mute
    for app in suggestions.apps_to_mute {
        manager.mute_app(&app); // Mute avec durée par défaut
    }

    // TODO: Appliquer les seuils recommandés
    // manager.set_idle_threshold(Duration::from_secs(suggestions.recommended_thresholds.idle_threshold as u64));

    Ok(())
}

#[tauri::command]
async fn save_ml_patterns(
    personalization_manager: tauri::State<'_, Arc<Mutex<PersonalizationManager>>>,
) -> Result<(), String> {
    let patterns_path = dirs::data_dir()
        .ok_or_else(|| "Could not find data directory".to_string())?
        .join("ShadowLearn")
        .join("ml_patterns.json");

    personalization_manager
        .lock()
        .await
        .save_patterns(&patterns_path.to_string_lossy())
}

#[tauri::command]
async fn load_ml_patterns(
    personalization_manager: tauri::State<'_, Arc<Mutex<PersonalizationManager>>>,
) -> Result<(), String> {
    let patterns_path = dirs::data_dir()
        .ok_or_else(|| "Could not find data directory".to_string())?
        .join("ShadowLearn")
        .join("ml_patterns.json");

    if patterns_path.exists() {
        personalization_manager
            .lock()
            .await
            .load_patterns(&patterns_path.to_string_lossy())
    } else {
        Ok(()) // Pas de patterns à charger
    }
}

// ========== J20: ARTEFACT VALIDATION COMMANDS ==========

#[tauri::command]
async fn validate_artefact(
    artefact_path: String,
    artefact_type: String,
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<bool, String> {
    use crate::validator::ArtefactType;
    use std::path::Path;

    let path = Path::new(&artefact_path);
    let artefact_type = match artefact_type.to_lowercase().as_str() {
        "blend" => ArtefactType::Blend,
        "midi" | "mid" => ArtefactType::Midi,
        "python" | "py" => ArtefactType::Python,
        "shader" | "glsl" | "vert" | "frag" => ArtefactType::Shader,
        "json" => ArtefactType::Json,
        "text" | "txt" | "md" => ArtefactType::Text,
        _ => ArtefactType::Unknown,
    };

    learning_system
        .lock()
        .await
        .validate_before_learning(path, artefact_type)
        .await
}

#[tauri::command]
async fn get_validation_stats(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<crate::validator::stats::ValidationStats, String> {
    Ok(learning_system.lock().await.get_validation_stats().clone())
}

#[tauri::command]
async fn get_validator_status(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<crate::validator::ValidatorStatus, String> {
    Ok(learning_system.lock().await.get_validator_status())
}

#[tauri::command]
async fn clear_validation_cache(
    learning_system: tauri::State<'_, Arc<Mutex<learning::LearningSystem>>>,
) -> Result<(), String> {
    learning_system.lock().await.clear_validation_cache();
    Ok(())
}

// J21.5: Feature Flags Commands
#[tauri::command]
fn get_feature_flags(flags: tauri::State<'_, Arc<FeatureFlags>>) -> FeaturesState {
    flags.get_state()
}

#[tauri::command]
fn enable_feature(
    flags: tauri::State<'_, Arc<FeatureFlags>>,
    feature: String,
) -> Result<(), String> {
    let feature_enum = match feature.as_str() {
        "idle_detection" => Feature::IdleDetection,
        "screenshot" => Feature::Screenshot,
        "smart_triggers" => Feature::SmartTriggers,
        "telemetry" => Feature::Telemetry,
        "use_intent_gate" => Feature::UseIntentGate,
        _ => return Err("Unknown feature".into()),
    };
    flags.enable(feature_enum);
    Ok(())
}

#[tauri::command]
fn disable_feature(
    flags: tauri::State<'_, Arc<FeatureFlags>>,
    feature: String,
) -> Result<(), String> {
    let feature_enum = match feature.as_str() {
        "idle_detection" => Feature::IdleDetection,
        "screenshot" => Feature::Screenshot,
        "smart_triggers" => Feature::SmartTriggers,
        "telemetry" => Feature::Telemetry,
        "use_intent_gate" => Feature::UseIntentGate,
        _ => return Err("Unknown feature".into()),
    };
    flags.disable(feature_enum);
    Ok(())
}

// J23: Artefact Generation commands
#[tauri::command]
async fn generate_artifact(
    domain: String,
    intent: String,
    trust_score: f32,
    idle_time: f32,
    cluster_id: String,
    artefact_type: String,
) -> Result<crate::artefact::GeneratedArtifact, String> {
    // TODO: Get learning system from app state
    // For now, return a placeholder
    use crate::artefact::ArtefactGenerator;
    use crate::validator::ArtefactType;
    
    let artefact_type_enum = match artefact_type.as_str() {
        "code" | "text" => ArtefactType::Text,
        "blender" | "blend" => ArtefactType::Blend,
        "midi" | "mid" => ArtefactType::Midi,
        "shader" => ArtefactType::Shader,
        "json" => ArtefactType::Json,
        "python" | "py" => ArtefactType::Python,
        _ => ArtefactType::Unknown,
    };

    let mut generator = ArtefactGenerator::new();
    generator.generate(
        domain,
        intent,
        trust_score,
        idle_time,
        cluster_id,
        artefact_type_enum,
    ).await
}

#[tauri::command]
fn get_artifact_stats() -> crate::artefact::ArtefactStats {
    // TODO: Get from learning system state
    // For now, return empty stats
    crate::artefact::ArtefactStats::new()
}

// J24: Feedback commands
#[tauri::command]
async fn record_artifact_feedback(
    suggestion_id: String,
    outcome: String,
) -> Result<f32, String> {
    use crate::learning::feedback::FeedbackOutcome;
    
    let feedback_outcome = match outcome.as_str() {
        "positive" => FeedbackOutcome::Positive,
        "negative" => FeedbackOutcome::Negative,
        "neutral" => FeedbackOutcome::Neutral,
        _ => return Err("Invalid outcome".to_string()),
    };

    // TODO: Get learning system from app state
    // For now, simulate feedback collection
    info!("[J24] Recording feedback for {}: {:?}", suggestion_id, feedback_outcome);
    
    Ok(0.5) // Placeholder trust score
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("🚀 Starting ShadowLearn...");

    // Initialize system components
    let feature_flags = Arc::new(FeatureFlags::from_env());
    let health_monitor = Arc::new(HealthMonitor::new());
    let recovery_manager = Arc::new(RecoveryManager::new()); // Max 3 restarts
    let telemetry = Arc::new(Telemetry::new(1000, 100)); // 1000 events, 100 samples per histogram

    // Initialize context aggregator
    let context_aggregator = Arc::new(Mutex::new(
        ContextAggregator::new().expect("Failed to initialize context aggregator"),
    ));

    // Initialize trigger manager
    let trigger_manager = Arc::new(Mutex::new(TriggerManager::new()));
    info!("✅ Trigger manager initialized");

    // Initialize state machine (J2)
    let state_machine = Arc::new(Mutex::new(triggers::state_machine::TriggerStateMachine::new()));
    info!("✅ State machine initialized");

    // Initialize LLM chat client (J3)
    let llm_client = Arc::new(tokio::sync::Mutex::new(chat::LLMChatClient::new()));
    info!("✅ LLM chat client initialized");

    // Initialize snooze manager
    let snooze_manager = Arc::new(Mutex::new(
        SnoozeManager::new().expect("Failed to initialize snooze manager"),
    ));
    info!("✅ Snooze manager initialized");

    // Initialize persistence manager
    let persistence_manager = Arc::new(Mutex::new(
        PersistenceManager::new()
            .await
            .expect("Failed to initialize persistence manager"),
    ));
    info!("✅ Persistence manager initialized");

    // Initialize personalization manager (J18)
    let personalization_manager = Arc::new(Mutex::new(PersonalizationManager::new()));
    info!("✅ Personalization manager initialized");

    // Initialize config manager (J5)
    let config_manager = Arc::new(Mutex::new(
        config::ConfigManager::new().expect("Failed to initialize config manager")
    ));
    info!("✅ Config manager initialized");

    // Initialize pause manager
    let pause_manager = Arc::new(Mutex::new(pause::PauseManager::new()));
    info!("✅ Pause manager initialized");

    // Initialize streak manager
    let streak_manager = Arc::new(Mutex::new(streaks::StreakManager::new()));
    info!("✅ Streak manager initialized");

    // Initialize personality manager
    let personality_manager = Arc::new(Mutex::new(personality::PersonalityManager::new()));
    info!("✅ Personality manager initialized");

    // Initialize digest manager (Clueless)
    let digest_manager = Arc::new(Mutex::new(digest::DigestManager::new()));
    info!("✅ Digest manager initialized");

    // Initialize pills manager (Clueless)
    let pills_manager = Arc::new(Mutex::new(pills::PillsManager::new()));
    info!("✅ Pills manager initialized");

    // Initialize productivity manager (Phase 3)
    let productivity_manager = Arc::new(Mutex::new(productivity::ProductivityManager::new()));
    info!("✅ Productivity manager initialized");

    // Initialize plugin manager (Phase 4)
    let plugin_manager = match plugins::PluginManager::new() {
        Ok(mut manager) => {
            match manager.load_all_plugins() {
                Ok(count) => info!("✅ Plugin manager initialized with {} plugins", count),
                Err(e) => warn!("⚠️ Failed to load plugins: {}", e),
            }
            Arc::new(Mutex::new(manager))
        }
        Err(e) => {
            warn!("⚠️ Plugin manager initialization failed: {}", e);
            Arc::new(Mutex::new(plugins::PluginManager::new().unwrap_or_else(|_| panic!("Failed to create plugin manager"))))
        }
    };

    // Initialize replay manager (Killer Feature)
    let replay_manager = Arc::new(Mutex::new(replay::ReplayManager::new(10000))); // Store last 10k events
    info!("✅ Replay manager initialized");

    // Initialize focus manager (Killer Feature)
    let focus_manager = Arc::new(Mutex::new(focus::FocusManager::new()));
    info!("✅ Focus manager initialized");

    // Initialize learn manager (Killer Feature)
    let learn_manager = Arc::new(Mutex::new(learn::LearnManager::new(100))); // Store 100 workflows
    info!("✅ Learn manager initialized");

    // Initialize screenshot capturer
    if let Err(e) = screenshot::init_capturer() {
        info!(
            "⚠️ Screenshot capturer init failed: {} (will retry on first capture)",
            e
        );
    }

    // Initialize screen monitor
    let monitor_config = monitor::MonitorConfig::default();
    let screen_monitor = Arc::new(Mutex::new(monitor::ScreenMonitor::new(monitor_config)));
    info!("✅ Screen monitor initialized");

    // Initialize shortcut manager
    let shortcut_config = shortcuts::manager::ShortcutConfig::default();
    let shortcut_manager = Arc::new(Mutex::new(shortcuts::ShortcutManager::new(shortcut_config)));
    info!("✅ Shortcut manager initialized");

    // Initialize privacy zone manager
    let privacy_config = privacy::PrivacyZonesConfig::default();
    let privacy_manager = Arc::new(Mutex::new(privacy::PrivacyZoneManager::new(privacy_config)));
    info!("✅ Privacy zone manager initialized");

    // Initialize pattern recognition manager (Phase 2.1)
    let app_dir = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("shadowlearn_data");
    let pattern_manager = match patterns::commands::PatternManager::new(app_dir) {
        Ok(manager) => {
            info!("✅ Pattern recognition manager initialized");
            Arc::new(manager)
        }
        Err(e) => {
            error!("❌ Failed to initialize pattern manager: {}", e);
            panic!("Cannot start without pattern manager");
        }
    };

    // Log feature state
    let state = feature_flags.get_state();
    info!("✅ Features enabled: {}/{}", state.enabled_count(), 4);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::all())
                .build(),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            // Global handler for all windows - LOG ALL EVENTS for debugging
            let label = window.label();

            // Log all events for chat window to diagnose hiding issue
            if label == "chat" {
                match event {
                    tauri::WindowEvent::CloseRequested { .. } => {
                        info!("🚨 [{}] CloseRequested event", label);
                    }
                    tauri::WindowEvent::Focused(focused) => {
                        info!("🎯 [{}] Focused event: {}", label, focused);
                    }
                    tauri::WindowEvent::Moved(_) => {
                        // Too verbose, skip
                    }
                    tauri::WindowEvent::Resized(_) => {
                        // Too verbose, skip
                    }
                    _ => {
                        info!("📡 [{}] Window event: {:?}", label, event);
                    }
                }
            }

            // Only hide chat window on CloseRequested, keep context visible
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label() == "chat" {
                        api.prevent_close();
                        if let Err(e) = window.hide() {
                            warn!("Failed to hide window '{}': {}", window.label(), e);
                        } else {
                            info!("🔒 Window '{}' hidden (close button pressed)", window.label());
                        }
                    }
                    // Context window can be closed normally
                }
                _ => {}
            }
        })
        .manage(feature_flags)
        .manage(health_monitor)
        .manage(recovery_manager)
        .manage(telemetry)
        .manage(context_aggregator.clone())
        .manage(trigger_manager.clone())
        .manage(state_machine.clone())
        .manage(llm_client.clone())
        .manage(snooze_manager.clone())
        .manage(Arc::new(Mutex::new(learning::LearningSystem::new(
            persistence_manager.lock().await.get_database(),
            "default_device".to_string(),
        ))))
        .manage(persistence_manager.clone())
        .manage(personalization_manager.clone())
        .manage(config_manager.clone()) // J5
        .manage(pause_manager)
        .manage(streak_manager)
        .manage(personality_manager)
        .manage(digest_manager) // Clueless: Daily Digest
        .manage(pills_manager) // Clueless: Smart Pills
        .manage(productivity_manager) // Phase 3: Productivity Dashboard
        .manage(plugin_manager) // Phase 4: Plugin System
        .manage(replay_manager) // Killer Feature: Shadow Replay
        .manage(focus_manager) // Killer Feature: Focus Mode
        .manage(learn_manager) // Killer Feature: Learn by Doing
        .manage(screen_monitor) // Screen Monitor
        .manage(shortcut_manager) // Global Shortcuts
        .manage(privacy_manager) // Privacy Zones
        .manage(pattern_manager) // Phase 2.1: Pattern Recognition ML
        .invoke_handler(tauri::generate_handler![
            toggle_window,
            ensure_chat_visible,
            broadcast_event,
            get_health_status,
            get_telemetry_stats,
            get_recovery_stats,
            record_telemetry_event,
            get_features_state,
            toggle_feature,
            capture_context,
            start_trigger_loop,
            check_should_trigger,
            record_trigger_fired,
            record_bubble_dismissed,
            record_trigger_action,
            get_trigger_stats,
            add_to_allowlist,
            remove_from_allowlist,
            trigger_mock_opportunity,
            screenshot::capture_screenshot,
            screenshot::check_screenshot_permission,
            screenshot::request_screenshot_permission,
            reset_user_activity,
            get_idle_state,
            snooze_triggers,
            unsnooze_triggers,
            get_snooze_status,
            // J17: Persistance & Mémoire commands
            create_conversation,
            save_message,
            get_recent_conversations,
            get_conversation_messages,
            get_persistence_stats,
            export_data,
            get_recent_contexts_for_app,
            save_context,
            // J16: Anti-spam & UX commands
            record_trigger_ignored,
            mute_app,
            unmute_app,
            record_snooze_used,
            get_extended_trigger_stats,
            // J16: Déclenchement discret commands
            set_bubble_visible,
            record_user_interaction,
            is_interaction_locked,
            get_interaction_lock_remaining,
            // J19: Learning System commands
            record_user_feedback,
            get_user_trust_level,
            get_trust_recommendations,
            reset_user_trust,
            // J20: Artefact Validation commands
            validate_artefact,
            get_validation_stats,
            get_validator_status,
            clear_validation_cache,
            // J21.5: Feature Flags commands
            get_feature_flags,
            enable_feature,
            disable_feature,
            // J23: Artefact Generation commands
            generate_artifact,
            get_artifact_stats,
            // J24: Feedback commands
            record_artifact_feedback,
            // Clueless: Opportunities commands
            opportunities::record_opportunity_response,
            opportunities::record_message_feedback,
            // Clueless: Flow State commands
            flow::detect_flow_state,
            // Clueless: Context Preview commands
            context::preview::get_context_preview,
            // Phase 3: Streaks commands
            streaks::commands::get_streak,
            streaks::commands::record_activity,
            // Phase 3: Personality commands
            personality::commands::get_personality,
            personality::commands::set_personality,
            // Phase 3: Pause commands
            pause::commands::get_pause_state,
            pause::commands::set_pause_state,
            // Clueless: Digest commands
            digest::get_daily_digest,
            digest::record_suggestion_shown,
            digest::record_suggestion_accepted,
            // Clueless: Pills commands
            pills::get_micro_suggestions,
            pills::dismiss_pill,
            // Clueless: Slash Commands
            commands::slash::execute_slash_command,
            // Phase 2.1: Pattern Recognition ML commands
            patterns::commands::record_user_action,
            patterns::commands::get_next_action_prediction,
            patterns::commands::get_learned_patterns,
            patterns::commands::get_patterns_by_tag,
            patterns::commands::get_all_repetitive_tasks,
            patterns::commands::get_high_priority_repetitive_tasks,
            patterns::commands::get_pattern_system_stats,
            patterns::commands::save_patterns_to_disk,
            patterns::commands::clear_pattern_storage,
            // J18: Personnalisation ML commands
            record_ml_event,
            get_usage_patterns,
            get_smart_suggestions,
            apply_smart_suggestions,
            save_ml_patterns,
            load_ml_patterns,
            // Window Management commands
            show_window,
            hide_window,
            minimize_window,
            is_window_visible,
            // J1-6: Crypto & Permissions commands
            crypto::keymanager::check_keychain_status,
            permissions::checker::check_permissions,
            permissions::checker::request_screen_recording_permission,
            permissions::checker::request_accessibility_permission,
            // J2: State Machine commands
            triggers::state_machine::get_trigger_state,
            triggers::state_machine::get_state_explanation,
            triggers::state_machine::get_state_history,
            // J3: Chat LLM commands
            chat::commands::chat_with_ai,
            chat::commands::check_llm_health,
            chat::commands::get_llm_stats,
            // J5: Config & Privacy commands
            config::manager::get_config,
            config::manager::update_config,
            config::manager::get_config_path,
            // Screen Monitor commands
            monitor::commands::start_screen_monitor,
            monitor::commands::stop_screen_monitor,
            monitor::commands::get_monitor_status,
            monitor::commands::reset_monitor_detector,
            monitor::commands::reset_monitor_cache,
            monitor::commands::get_monitor_cache_stats,
            // Keyboard Shortcuts commands
            shortcuts::commands::get_shortcuts_config,
            shortcuts::commands::list_shortcuts,
            shortcuts::commands::trigger_shortcut_action,
            shortcuts::commands::toggle_spotlight,
            // Privacy Zones commands
            privacy::commands::get_privacy_zones_config,
            privacy::commands::add_privacy_zone,
            privacy::commands::remove_privacy_zone,
            privacy::commands::set_privacy_zones_enabled,
            privacy::commands::is_app_protected,
            // Phase 3: Productivity Dashboard commands
            productivity::get_productivity_metrics,
            productivity::record_productivity_event,
            productivity::record_flow_session_event,
            // Phase 4: Plugin System commands
            plugins::get_all_plugins,
            plugins::get_plugin_info,
            plugins::enable_plugin,
            plugins::disable_plugin,
            plugins::uninstall_plugin,
            plugins::reload_plugins,
            plugins::get_plugin_stats,
            plugins::execute_plugin_hook,
            // Killer Feature: Shadow Replay commands
            replay::get_replay_events,
            replay::get_replay_sessions,
            replay::get_replay_stats,
            replay::start_replay_playback,
            replay::stop_replay_playback,
            replay::set_replay_speed,
            replay::get_next_replay_event,
            replay::get_playback_state,
            replay::seek_replay_to,
            replay::record_replay_suggestion,
            replay::record_replay_flow_session,
            // Killer Feature: Focus Mode commands
            focus::get_focus_state,
            focus::get_focus_stats,
            focus::get_focus_config,
            focus::update_focus_config,
            focus::detect_focus_mode,
            focus::should_block_notification,
            focus::should_block_trigger,
            focus::end_focus_session,
            focus::get_recent_focus_sessions,
            // Killer Feature: Learn by Doing commands
            learn::start_workflow_recording,
            learn::stop_workflow_recording,
            learn::add_workflow_comment,
            learn::generate_workflow_tutorial,
            learn::get_recording_state,
            learn::get_all_workflows,
            learn::get_all_tutorials,
            learn::export_tutorial_as_markdown
        ])
        .setup(|app| {
            // Setup ESC=hide for existing windows
            if let Some(main_window) = app.get_webview_window("main") {
                info!("🔧 Setting up ESC=hide for main window");
                let window = main_window.clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Err(e) = window.hide() {
                            warn!("Failed to hide window: {}", e);
                        } else {
                            info!("🔒 Window hidden (ESC pressed)");
                        }
                    }
                });
            }

            // Register global shortcuts
            info!("🎹 About to register global shortcuts...");
            let shortcut_mgr = app.state::<Arc<Mutex<shortcuts::ShortcutManager>>>();
            let shortcut_mgr_clone = shortcut_mgr.inner().clone();
            let app_handle = app.handle().clone();

            // Spawn async task to register shortcuts (cannot use block_on inside Tauri runtime)
            tauri::async_runtime::spawn(async move {
                info!("🎹 Inside async block - acquiring lock...");
                let manager = shortcut_mgr_clone.lock().await;
                info!("🎹 Lock acquired - calling register_all...");
                if let Err(e) = manager.register_all(&app_handle).await {
                    error!("❌ Failed to register shortcuts: {}", e);
                } else {
                    info!("✅ All global shortcuts registered");
                }
            });
            info!("🎹 Shortcut registration task spawned");

            // Setup HUD click listener to show Spotlight
            let app_handle_for_hud = app.handle().clone();
            app.listen("hud:click", move |_event| {
                info!("🔍 HUD clicked - showing Spotlight");
                if let Some(spotlight_window) = app_handle_for_hud.get_webview_window("spotlight") {
                    if let Err(e) = spotlight_window.show() {
                        error!("Failed to show spotlight: {}", e);
                    }
                    if let Err(e) = spotlight_window.set_focus() {
                        error!("Failed to focus spotlight: {}", e);
                    }
                    // Emit event to tell Spotlight frontend to show content
                    if let Err(e) = app_handle_for_hud.emit("spotlight:show", ()) {
                        error!("Failed to emit spotlight:show: {}", e);
                    }
                } else {
                    error!("Spotlight window not found");
                }
            });
            info!("✅ HUD click listener registered");

            info!("🔍 Checking available windows...");

            // macOS Fix: Use Regular activation policy so app appears in dock
            // Previous: Accessory policy prevented Stage Manager glitches but made windows inaccessible
            // Tradeoff: App now appears in dock and windows are accessible, but may have occasional
            // Stage Manager visual glitches on focus change (acceptable vs inaccessible windows)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Regular);
                info!("🍎 macOS: Activation policy set to Regular (app accessible in dock)");
            }

            // Force show and position chat window
            if let Some(chat) = app.get_webview_window("chat") {
                info!("✅ Found chat window, showing...");
                let _ = chat.show();
                let _ = chat.set_focus();
                if let Ok(Some(monitor)) = chat.current_monitor() {
                    let size = monitor.size();
                    let _ = chat.set_position(PhysicalPosition::new(
                        size.width as i32 - 440,
                        (size.height as i32 / 2) - 280,
                    ));
                    info!("📍 Chat window positioned and visible");
                }
            } else {
                warn!("⚠️ chat window NOT FOUND!");
            }

            // Configure Spotlight window for macOS fullscreen support
            if let Some(spotlight) = app.get_webview_window("spotlight") {
                info!("✅ Found spotlight window, configuring...");

                // Force window size to 900×700 (Phase 3A testing size)
                use tauri::Size;
                if let Err(e) = spotlight.set_size(Size::Physical(tauri::PhysicalSize {
                    width: 900,
                    height: 700,
                })) {
                    warn!("⚠️ Failed to set spotlight size: {}", e);
                } else {
                    info!("📐 Spotlight size forced to 900×700");
                }

                // Ensure it's hidden initially
                let _ = spotlight.hide();

                // Set always on top to ensure visibility over fullscreen apps
                if let Err(e) = spotlight.set_always_on_top(true) {
                    warn!("⚠️ Failed to set spotlight always on top: {}", e);
                } else {
                    info!("🔍 Spotlight configured: always-on-top enabled");
                }

                info!("🔍 Spotlight window ready (hidden, will show on Cmd+Shift+Y)");
            } else {
                warn!("⚠️ spotlight window NOT FOUND!");
            }

            // Configure HUD window for macOS fullscreen support (luciole)
            if let Some(hud) = app.get_webview_window("hud") {
                info!("✅ Found HUD window, configuring for fullscreen visibility...");

                // Ensure it's visible
                let _ = hud.show();
                let _ = hud.set_always_on_top(true);

                #[cfg(target_os = "macos")]
                {
                    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior, NSMainMenuWindowLevel};
                    use cocoa::base::id;

                    if let Ok(ns_window_ptr) = hud.ns_window() {
                        let ns_window = ns_window_ptr as id;

                        unsafe {
                            // Configure window behavior to appear on all Spaces and in fullscreen
                            let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary;

                            ns_window.setCollectionBehavior_(behavior);

                            // Set window level above fullscreen apps (menu bar level + 1)
                            let level = (NSMainMenuWindowLevel as i64) + 1;
                            ns_window.setLevel_(level);

                            info!("🔥 HUD configured with NSWindowCollectionBehavior for fullscreen visibility");
                            info!("🔥 HUD window level set to {} (above menu bar)", level);
                        }
                    } else {
                        warn!("⚠️ Failed to get HUD ns_window");
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    info!("🔥 HUD configured (non-macOS: always-on-top only)");
                }

                info!("🔥 HUD ready - visible as ambient LED indicator");
            } else {
                warn!("⚠️ HUD window NOT FOUND!");
            }

            // 🔥 Lance automatiquement la boucle de triggers
            tauri::async_runtime::spawn(triggers::trigger_loop::start_trigger_loop(
                app.handle().clone(),
            ));

            info!("✅ Setup complete – trigger loop launched");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
