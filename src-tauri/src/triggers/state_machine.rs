use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum TriggerState {
    Observing {
        app_name: String,
    },
    IdleDetected {
        app_name: String,
        idle_seconds: f64,
        waiting_for_stable: bool,
    },
    ContextConfirmed {
        app_name: String,
        opportunity: OpportunityPreview,
        confidence: f32,
    },
    PromptShown {
        suggestion_id: String,
    },
    UserResponded {
        accepted: bool,
    },
    Cooldown {
        remaining_seconds: u64,
        reason: CooldownReason,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CooldownReason {
    UserAccepted,
    UserDismissed,
    LowConfidence,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityPreview {
    pub detected_task: String,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerEvent {
    AppChanged { app_name: String },
    IdleThresholdReached { idle: f64 },
    IdleStabilized,
    ContextAnalyzed { opportunity: OpportunityPreview, confidence: f32 },
    ShowPrompt { suggestion_id: String },
    UserAccepted,
    UserDismissed,
    EnterCooldown { reason: CooldownReason },
    CooldownTick { remaining: u64 },
    CooldownExpired,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct TriggerStateMachine {
    current_state: TriggerState,
    history: VecDeque<StateTransition>,
    max_history: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct StateTransition {
    pub from: TriggerState,
    pub to: TriggerState,
    pub event: TriggerEvent,
    pub timestamp: i64,
    pub explanation: String,
}

impl TriggerStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: TriggerState::Observing {
                app_name: String::new(),
            },
            history: VecDeque::new(),
            max_history: 100,
        }
    }

    pub fn transition(&mut self, event: TriggerEvent) -> Result<(), String> {
        let old_state = self.current_state.clone();

        let (new_state, explanation) = match (&self.current_state, &event) {
            // Observing → IdleDetected
            (
                TriggerState::Observing { app_name, .. },
                TriggerEvent::IdleThresholdReached { idle },
            ) => (
                TriggerState::IdleDetected {
                    app_name: app_name.clone(),
                    idle_seconds: *idle,
                    waiting_for_stable: true,
                },
                format!("Inactivité détectée ({:.0}s) dans {}", idle, app_name),
            ),

            // IdleDetected → IdleDetected (stabilization)
            (
                TriggerState::IdleDetected { app_name, idle_seconds, .. },
                TriggerEvent::IdleStabilized,
            ) => (
                TriggerState::IdleDetected {
                    app_name: app_name.clone(),
                    idle_seconds: *idle_seconds,
                    waiting_for_stable: false,
                },
                format!("Inactivité stable ({:.0}s), analyse du contexte...", idle_seconds),
            ),

            // IdleDetected → ContextConfirmed
            (
                TriggerState::IdleDetected { app_name, .. },
                TriggerEvent::ContextAnalyzed { opportunity, confidence },
            ) => {
                if *confidence < 0.5 {
                    return self.transition(TriggerEvent::EnterCooldown {
                        reason: CooldownReason::LowConfidence,
                    });
                }

                (
                    TriggerState::ContextConfirmed {
                        app_name: app_name.clone(),
                        opportunity: opportunity.clone(),
                        confidence: *confidence,
                    },
                    format!(
                        "Opportunité trouvée : {} (confiance {:.0}%)",
                        opportunity.detected_task,
                        confidence * 100.0
                    ),
                )
            }

            // ContextConfirmed → PromptShown
            (
                TriggerState::ContextConfirmed { .. },
                TriggerEvent::ShowPrompt { suggestion_id },
            ) => (
            TriggerState::PromptShown {
                suggestion_id: suggestion_id.clone(),
            },
                "Suggestion affichée à l'utilisateur".into(),
            ),

            // PromptShown → UserResponded
            (TriggerState::PromptShown { .. }, TriggerEvent::UserAccepted) => (
                TriggerState::UserResponded {
                    accepted: true,
                },
                "Utilisateur a accepté la suggestion".into(),
            ),

            (TriggerState::PromptShown { .. }, TriggerEvent::UserDismissed) => (
                TriggerState::UserResponded {
                    accepted: false,
                },
                "Utilisateur a refusé la suggestion".into(),
            ),

            // UserResponded → Cooldown
            (TriggerState::UserResponded { .. }, TriggerEvent::EnterCooldown { reason }) => {
                let duration = match reason {
                    CooldownReason::UserAccepted => 45,
                    CooldownReason::UserDismissed => 90,
                    CooldownReason::LowConfidence => 60,
                    CooldownReason::Error => 120,
                };

                (
                    TriggerState::Cooldown {
                        remaining_seconds: duration,
                        reason: reason.clone(),
                    },
                    format!("Pause de {}s ({})", duration, self.explain_cooldown_reason(reason)),
                )
            }

            // Cooldown → Cooldown (tick)
            (TriggerState::Cooldown { reason, .. }, TriggerEvent::CooldownTick { remaining }) => (
                TriggerState::Cooldown {
                    remaining_seconds: *remaining,
                    reason: reason.clone(),
                },
                format!("Pause en cours : {}s restantes", remaining),
            ),

            // Cooldown → Observing
            (TriggerState::Cooldown { .. }, TriggerEvent::CooldownExpired) => (
                TriggerState::Observing {
                    app_name: String::new(),
                },
                "Pause terminée, reprise de l'observation".into(),
            ),

            // AppChanged → Reset to Observing
            (_, TriggerEvent::AppChanged { app_name }) => (
                TriggerState::Observing {
                    app_name: app_name.clone(),
                },
                format!("Application changée : {}", app_name),
            ),

            // Error → Cooldown
            (_, TriggerEvent::Error { message }) => {
                tracing::error!("State machine error: {}", message);
                (
                    TriggerState::Cooldown {
                        remaining_seconds: 120,
                        reason: CooldownReason::Error,
                    },
                    format!("Erreur : {}. Pause de 2min.", message),
                )
            }

            _ => {
                return Err(format!(
                    "Invalid transition: {:?} with event {:?}",
                    self.current_state, event
                ));
            }
        };

        // Record transition
        let transition = StateTransition {
            from: old_state.clone(),
            to: new_state.clone(),
            event: event.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            explanation: explanation.clone(),
        };

        self.history.push_back(transition);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }

        info!("State transition: {}", explanation);

        self.current_state = new_state;

        Ok(())
    }

    pub fn get_current_state(&self) -> &TriggerState {
        &self.current_state
    }

    pub fn get_explanation(&self) -> String {
        match &self.current_state {
            TriggerState::Observing { app_name, .. } => {
                if app_name.is_empty() {
                    "👀 En attente d'une application active...".into()
                } else {
                    format!("👀 J'observe {} en attendant que vous soyez inactif", app_name)
                }
            }
            TriggerState::IdleDetected { idle_seconds, waiting_for_stable, .. } => {
                if *waiting_for_stable {
                    format!("⏱️ Inactivité détectée ({:.0}s), vérification en cours...", idle_seconds)
                } else {
                    format!("✓ Inactivité stable ({:.0}s), analyse du contexte...", idle_seconds)
                }
            }
            TriggerState::ContextConfirmed { opportunity, confidence, .. } => {
                format!(
                    "🎯 {} (confiance: {:.0}%)",
                    opportunity.detected_task,
                    confidence * 100.0
                )
            }
            TriggerState::PromptShown { .. } => {
                "💬 Suggestion affichée, en attente de votre réponse".into()
            }
            TriggerState::UserResponded { accepted, .. } => {
                if *accepted {
                    "✅ Merci ! Génération en cours...".into()
                } else {
                    "👋 Compris, je reviens plus tard".into()
                }
            }
            TriggerState::Cooldown { remaining_seconds, reason } => {
                format!(
                    "⏸️ Pause de {}s ({})",
                    remaining_seconds,
                    self.explain_cooldown_reason(reason)
                )
            }
        }
    }

    fn explain_cooldown_reason(&self, reason: &CooldownReason) -> String {
        match reason {
            CooldownReason::UserAccepted => "vous avez accepté".into(),
            CooldownReason::UserDismissed => "vous avez refusé".into(),
            CooldownReason::LowConfidence => "confiance faible".into(),
            CooldownReason::Error => "erreur technique".into(),
        }
    }

    pub fn get_history(&self) -> &VecDeque<StateTransition> {
        &self.history
    }

    pub fn can_bypass_cooldown(&self) -> bool {
        matches!(self.current_state, TriggerState::Cooldown { .. })
    }
}

#[tauri::command]
pub async fn get_trigger_state(
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<TriggerStateMachine>>>,
) -> Result<TriggerState, String> {
    Ok(state.lock().await.get_current_state().clone())
}

#[tauri::command]
pub async fn get_state_explanation(
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<TriggerStateMachine>>>,
) -> Result<String, String> {
    Ok(state.lock().await.get_explanation())
}

#[tauri::command]
pub async fn get_state_history(
    limit: usize,
    state: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<TriggerStateMachine>>>,
) -> Result<Vec<StateTransition>, String> {
    let history = state.lock().await;
    let history_snapshot = history.get_history().iter().rev().take(limit).cloned().collect();
    Ok(history_snapshot)
}
