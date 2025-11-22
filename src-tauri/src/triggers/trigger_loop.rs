use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, trace, warn};

use super::manager::{TriggerDecision, TriggerManager};
use super::state_machine::{TriggerEvent, TriggerStateMachine};
use crate::context::aggregator::{Context, ContextAggregator};
use crate::ml::{EventType as MLEventType, PersonalizationManager, UserEvent};
use crate::snooze::SnoozeManager;

/// Lance la boucle de trigger en arrière-plan
/// DOIT être appelé depuis un contexte async (ex: invoke_handler ou plugin init)
pub async fn start_trigger_loop(app_handle: AppHandle) {
    tokio::spawn(async move {
        info!("🔄 Starting trigger loop...");
        trigger_loop(app_handle).await;
    });
}

/// Boucle principale de trigger
async fn trigger_loop(app_handle: AppHandle) {
    let _context_aggregator = app_handle.state::<Arc<Mutex<ContextAggregator>>>();
    let _trigger_manager = app_handle.state::<Arc<Mutex<TriggerManager>>>();
    let _snooze_manager = app_handle.state::<Arc<Mutex<SnoozeManager>>>();
    let personalization_manager = app_handle.state::<Arc<Mutex<PersonalizationManager>>>();
    let state_machine = app_handle.state::<std::sync::Arc<tokio::sync::Mutex<TriggerStateMachine>>>();

    // macOS Fix: Reduced frequency to prevent glassmorphism flicker
    // Frequent events destabilize backdrop-filter on macOS transparent windows
    let mut ticker = interval(Duration::from_millis(5000)); // 5s (was 2s)
    let mut consecutive_failures = 0;
    const MAX_FAILURES: u32 = 3;

    loop {
        ticker.tick().await;

        // Get managed state
        let context_aggregator = match app_handle.try_state::<Arc<Mutex<ContextAggregator>>>() {
            Some(ctx) => ctx,
            None => {
                error!("❌ ContextAggregator not found in app state");
                continue;
            }
        };

        let trigger_manager = match app_handle.try_state::<Arc<Mutex<TriggerManager>>>() {
            Some(tm) => tm,
            None => {
                error!("❌ TriggerManager not found in app state");
                continue;
            }
        };

        let snooze_manager = match app_handle.try_state::<Arc<Mutex<SnoozeManager>>>() {
            Some(sm) => sm,
            None => {
                error!("❌ SnoozeManager not found in app state");
                continue;
            }
        };

        // Cleanup expired mutes periodically
        {
            let mut manager = trigger_manager.lock().await;
            manager.cleanup_expired_mutes();
        }

        // Check si snoozed
        let is_snoozed = snooze_manager.lock().await.is_snoozed();
        if is_snoozed {
            trace!("😴 Triggers snoozed, skipping check");
            continue;
        }

        // Peek context (lightweight < 10ms)
        let peek_result = {
            let mut aggregator = context_aggregator.lock().await;
            match aggregator.peek() {
                Ok(result) => result,
                Err(e) => {
                    consecutive_failures += 1;
                    warn!(
                        "⚠️ Context peek failed: {} (failures: {})",
                        e, consecutive_failures
                    );

                    if consecutive_failures >= MAX_FAILURES {
                        warn!("❌ Too many failures, cooling down 5s...");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        consecutive_failures = 0;
                    }
                    continue;
                }
            }
        };

        // Reset failure counter on success
        consecutive_failures = 0;

        // 🔥 EMIT FLOW STATE EVENT - Update frontend with current flow state
        {
            let flow_state = if peek_result.idle_seconds < 5.0 {
                "deep"
            } else if peek_result.idle_seconds < 30.0 {
                "normal"
            } else {
                "blocked"
            };
            
            let flow_payload = serde_json::json!({
                "flow_state": flow_state,
                "confidence": 0.85,
                "idle_seconds": peek_result.idle_seconds,
                "app": peek_result.app.name,
            });
            
            if let Err(e) = app_handle.emit("shadow:flow_state", &flow_payload) {
                debug!("Failed to emit flow_state event: {}", e);
            }
        }

        // 🔥 EMIT CONTEXT UPDATE EVENT - Update context preview components
        {
            let context_payload = serde_json::json!({
                "app_name": peek_result.app.name,
                "window_title": peek_result.app.window_title.clone(),
                "idle_seconds": peek_result.idle_seconds,
                "session_duration_minutes": 0, // TODO: calculate actual session duration
                "recent_screenshots": 0, // TODO: get from context aggregator
                "pending_suggestion": null,
            });
            
            if let Err(e) = app_handle.emit("shadow:context_update", &context_payload) {
                debug!("Failed to emit context_update event: {}", e);
            }
        }

        // Create a minimal context for decision
        let mini_ctx = Context {
            id: "peek".into(),
            app: peek_result.app.clone(),
            clipboard: None,
            idle_seconds: peek_result.idle_seconds,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            capture_duration_ms: 0,
        };

        // Check trigger decision
        let decision = {
            let mut manager = trigger_manager.lock().await;
            manager.should_trigger(&mini_ctx)
        };

        // Skip processing if app is muted to avoid spam
        if let TriggerDecision::Rejected(crate::triggers::manager::RejectReason::Muted) = decision {
            trace!(
                "🔇 App '{}' is muted, skipping trigger check",
                peek_result.app.name
            );
            continue;
        }

        // Additional check: if app is muted, skip entirely
        {
            let manager = trigger_manager.lock().await;
            if manager.is_app_muted(&peek_result.app.name) {
                info!(
                    "🔇 App '{}' is muted (direct check), skipping",
                    peek_result.app.name
                );
                continue;
            }
        }

        // LOG FOR DEBUG - Check decision (trace level to avoid spam)
        trace!("🔍 Trigger loop iteration: app={}, idle={:.1}s", peek_result.app.name, peek_result.idle_seconds);

        // Update state machine with current decision
        let _state_change_result = {
            let mut sm = state_machine.lock().await;
            match decision {
                TriggerDecision::Allow => {
                    let _ = sm.transition(TriggerEvent::IdleThresholdReached { idle: peek_result.idle_seconds });
                }
                TriggerDecision::Debouncing { .. } => {
                    let _ = sm.transition(TriggerEvent::IdleStabilized);
                }
                TriggerDecision::Rejected(_) => {
                    // State stays in Observing or returns to Observing
                }
            }
        };

        match decision {
            TriggerDecision::Allow => {
                info!("✅ Trigger FIRED for app '{}' (idle: {:.1}s, reason: idle_ok+cooldown_ok+allowlist_ok)", 
                    peek_result.app.name, peek_result.idle_seconds);

                // Record trigger and reset ignored count
                {
                    let mut manager = trigger_manager.lock().await;
                    manager.record_trigger(&peek_result.app.name);
                    manager.reset_ignored_count(&peek_result.app.name); // Reset ignored count on successful trigger
                }

                // J18: Record ML event for trigger fired
                {
                    let ml_event = UserEvent {
                        timestamp: chrono::Utc::now(),
                        event_type: MLEventType::TriggerFired,
                        app_name: peek_result.app.name.clone(),
                        context: Some(format!("idle_seconds:{}", peek_result.idle_seconds)),
                        user_response: None,
                    };
                    personalization_manager.lock().await.record_event(ml_event);
                }

                // Capture FULL context (with clipboard, screenshot, etc.)
                let full_ctx = {
                    let mut aggregator = context_aggregator.lock().await;
                    match aggregator.capture().await {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            error!("❌ Full capture failed: {}", e);
                            continue;
                        }
                    }
                };

                // Update state machine with ShowPrompt event
                {
                    let mut sm = state_machine.lock().await;
                    let opportunity = super::state_machine::OpportunityPreview {
                        detected_task: full_ctx.app.name.clone(),
                        explanation: format!("App: {} (idle: {:.1}s)", full_ctx.app.name, full_ctx.idle_seconds),
                    };
                    let confidence = if full_ctx.idle_seconds > 30.0 { 0.8 } else { 0.6 };
                    let suggestion_id = format!("sugg_{}", chrono::Utc::now().timestamp());
                    let _ = sm.transition(TriggerEvent::ContextAnalyzed { opportunity, confidence });
                    let _ = sm.transition(TriggerEvent::ShowPrompt { suggestion_id });
                }

                // 🔥 EMIT OPPORTUNITY EVENT - DISABLED in Phase 3B
                // Phase 3B: Opportunities are now triggered by real pattern detection (file watcher)
                // instead of idle_seconds. See detection::file_watcher module.
                // {
                //     let opp_id = format!("opp_{}", chrono::Utc::now().timestamp());
                //     let opp_payload = serde_json::json!({
                //         "id": opp_id,
                //         "title": format!("J'ai une idée pour {}", full_ctx.app.name),
                //         "confidence": 0.8,
                //         "preview": format!("Tu travailles sur {} depuis {} secondes. Besoin d'aide ?", full_ctx.app.name, full_ctx.idle_seconds as u32),
                //         "app": full_ctx.app.name,
                //         "context": {
                //             "app_name": full_ctx.app.name,
                //             "idle_seconds": full_ctx.idle_seconds,
                //         }
                //     });
                //
                //     if let Err(e) = app_handle.emit("shadow:opportunity", &opp_payload) {
                //         debug!("Failed to emit opportunity event: {}", e);
                //     }
                // }

                // 🔥 EMIT MICRO SUGGESTIONS - Notify pills component
                {
                    let pills_payload = serde_json::json!([
                        {
                            "id": format!("pill_help_{}", chrono::Utc::now().timestamp()),
                            "text": format!("Besoin d'aide avec {} ?", full_ctx.app.name),
                            "type": "help",
                        }
                    ]);
                    
                    if let Err(e) = app_handle.emit("shadow:micro_suggestion", &pills_payload) {
                        debug!("Failed to emit micro_suggestion event: {}", e);
                    }
                }

                // Record suggestion shown in digest manager
                if let Some(digest_manager) = app_handle.try_state::<Arc<Mutex<crate::digest::DigestManager>>>() {
                    let mut manager = digest_manager.lock().await;
                    manager.record_suggestion_shown(&full_ctx.app.name);
                }

                // 💡 Update global state to indicate opportunity (lightweight notification for UI)
                // Note: This replaces the old setInterval-based approach
                // Frontend will listen to `shadow:opportunity` event
            }

            TriggerDecision::Rejected(reason) => {
                trace!("❌ Trigger REJECTED: {:?} (app: {}, idle: {:.1}s)", reason, peek_result.app.name, peek_result.idle_seconds);
            }

            TriggerDecision::Debouncing { wait_ms } => {
                trace!("⏳ Trigger DEBOUNCING ({}ms remaining)", wait_ms);
            }
        }
    }
}
