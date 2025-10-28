use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace};

use crate::context::aggregator::Context;

pub struct TriggerManager {
    last_trigger: Option<Instant>,
    last_dismiss: Option<Instant>,
    cooldown_base: Duration,
    cooldown_dismiss: Duration,
    idle_threshold: Duration,
    debounce: Duration,
    allowlist: Vec<String>,
    trigger_count: HashMap<String, usize>,
    total_triggers: usize,
    debounce_start: Option<Instant>, // Timestamp quand le debounce a commencé
    quick_response_threshold: Duration, // Si réponse < 5s, pas de penalty

    // J16: Anti-spam & UX
    ignored_triggers: HashMap<String, usize>, // Compteur triggers ignorés par app
    muted_apps: HashMap<String, Instant>,     // Apps mutées avec timestamp
    mute_duration: Duration,                  // Durée du mute (10 min)
    dismissed_count: usize,                   // Total dismissals
    snoozed_count: usize,                     // Total snoozes

    // J16: Hystérésis idle
    idle_activated: bool,         // État idle activé
    idle_off_threshold: Duration, // Seuil pour désactiver idle (5s)

    // J16: Déclenchement discret
    bubble_visible: bool,              // Bulle actuellement visible
    interaction_lock: Option<Instant>, // Verrou après interaction (45s)
}

impl TriggerManager {
    pub fn new() -> Self {
        Self {
            last_trigger: None,
            last_dismiss: None,
            cooldown_base: Duration::from_secs(45),
            cooldown_dismiss: Duration::from_secs(90),
            idle_threshold: Duration::from_secs(12), // 12s pour éviter le spam
            debounce: Duration::from_secs(2),
            allowlist: vec![
                "Visual Studio Code".into(),
                "Cursor".into(),
                "cursor".into(), // bundle id
                "FL Studio".into(),
                "Blender".into(),
                "Figma".into(),
                "Google Chrome".into(),
                "chrome".into(),
                "Firefox".into(),
                "Safari".into(),
                "Notes".into(),
                "Terminal".into(),
                "iTerm".into(),
            ],
            trigger_count: HashMap::new(),
            total_triggers: 0,
            debounce_start: None,
            quick_response_threshold: Duration::from_secs(5), // 5s pour réponse rapide

            // J16: Anti-spam & UX
            ignored_triggers: HashMap::new(),
            muted_apps: HashMap::new(),
            mute_duration: Duration::from_secs(10 * 60), // 10 minutes
            dismissed_count: 0,
            snoozed_count: 0,

            // J16: Hystérésis idle
            idle_activated: false,
            idle_off_threshold: Duration::from_secs(5), // 5s pour désactiver

            // J16: Déclenchement discret
            bubble_visible: false,
            interaction_lock: None,
        }
    }

    /// Vérifie si un trigger doit être déclenché
    pub fn should_trigger(&mut self, ctx: &Context) -> TriggerDecision {
        // 0. J16: Déclenchement discret - pas de re-popup si bulle visible
        if self.bubble_visible {
            trace!("Trigger rejected: bubble already visible");
            return TriggerDecision::Rejected(RejectReason::NotIdle); // Utilise NotIdle pour simplicité
        }

        // 0.1. J16: Vérifier verrou interaction (45s après interaction)
        if let Some(lock_time) = self.interaction_lock {
            let elapsed = lock_time.elapsed();
            if elapsed < Duration::from_secs(45) {
                let remaining = Duration::from_secs(45).saturating_sub(elapsed);
                trace!(
                    "Trigger rejected: interaction lock ({:?} remaining)",
                    remaining
                );
                return TriggerDecision::Rejected(RejectReason::Cooldown {
                    remaining_ms: remaining.as_millis() as u64,
                });
            } else {
                // Verrou expiré, le supprimer
                self.interaction_lock = None;
            }
        }

        // 1. Check if app is muted (J16)
        if self.is_app_muted(&ctx.app.name) {
            trace!("Trigger rejected: app '{}' is muted", ctx.app.name);
            return TriggerDecision::Rejected(RejectReason::Muted);
        }

        // 1. Check allowlist
        if !self.is_allowed(ctx) {
            trace!("Trigger rejected: app '{}' not in allowlist", ctx.app.name);
            return TriggerDecision::Rejected(RejectReason::NotAllowlisted);
        }

        // 2. Check cooldown
        let cooldown = if self.last_dismiss.is_some() {
            self.cooldown_dismiss
        } else {
            self.cooldown_base
        };

        if let Some(last) = self.last_trigger {
            let elapsed = last.elapsed();
            if elapsed < cooldown {
                let remaining = cooldown.saturating_sub(elapsed);
                trace!("Trigger rejected: cooldown ({:?} remaining)", remaining);
                return TriggerDecision::Rejected(RejectReason::Cooldown {
                    remaining_ms: remaining.as_millis() as u64,
                });
            }
        }

        // 3. J16: Hystérésis idle (ON=12s, OFF=5s)
        let idle_threshold_secs = self.idle_threshold.as_secs_f64();
        let idle_off_threshold_secs = self.idle_off_threshold.as_secs_f64();

        // Si pas encore activé et idle suffisant → activer
        if !self.idle_activated && ctx.idle_seconds >= idle_threshold_secs {
            self.idle_activated = true;
            info!("🟢 Idle activé (hystérésis): {:.1}s", ctx.idle_seconds);
        }

        // Si activé mais utilisateur actif → désactiver
        if self.idle_activated && ctx.idle_seconds < idle_off_threshold_secs {
            self.idle_activated = false;
            info!("🔴 Idle désactivé (hystérésis): {:.1}s", ctx.idle_seconds);
        }

        // Vérifier si idle activé
        if !self.idle_activated {
            trace!(
                "Trigger rejected: idle not activated ({:.1}s < {}s)",
                ctx.idle_seconds,
                idle_threshold_secs
            );
            return TriggerDecision::Rejected(RejectReason::NotIdle);
        }

        // 4. Check debounce (idle must be stable)
        let required_idle = (self.idle_threshold + self.debounce).as_secs_f64();
        if ctx.idle_seconds < required_idle {
            let wait = Duration::from_secs_f64(required_idle - ctx.idle_seconds);
            debug!("Debouncing: {:.1}s wait remaining", wait.as_secs_f64());
            return TriggerDecision::Debouncing {
                wait_ms: wait.as_millis() as u64,
            };
        }

        info!("✅ Trigger ALLOW for app '{}'", ctx.app.name);
        TriggerDecision::Allow
    }

    /// Enregistre qu'un trigger a été déclenché
    pub fn record_trigger(&mut self, app_name: &str) {
        self.last_trigger = Some(Instant::now());
        *self.trigger_count.entry(app_name.into()).or_insert(0) += 1;
        self.total_triggers += 1;
        self.last_dismiss = None; // Reset dismiss state
        info!(
            "📊 Trigger recorded for '{}' (total: {})",
            app_name, self.total_triggers
        );
    }

    /// Enregistre qu'un utilisateur a agi
    /// Si l'action est rapide (< 5s après trigger), pas de cooldown
    pub fn record_action(&mut self) {
        // Check si l'action était rapide
        let was_quick_response = if let Some(last) = self.last_trigger {
            last.elapsed() < self.quick_response_threshold
        } else {
            false
        };

        if was_quick_response {
            info!("⚡ Quick response detected (< 5s) → no cooldown penalty");
            // Réponse rapide : on remet le cooldown au minimum (ou on le skip)
            self.last_trigger = None; // Reset complètement le cooldown
        } else {
            info!("✅ User action recorded (standard cooldown applies)");
        }

        self.last_dismiss = None; // Reset dismiss penalty
        self.debounce_start = None; // Reset debounce
    }

    /// Reset le debounce (appelé sur activité utilisateur)
    pub fn reset_debounce(&mut self) {
        if self.debounce_start.is_some() {
            debug!("🔄 Debounce reset due to user activity");
            self.debounce_start = None;
        }
    }

    /// Vérifie si l'app est dans l'allowlist
    fn is_allowed(&self, ctx: &Context) -> bool {
        let name_lower = ctx.app.name.to_lowercase();
        let bundle_lower = ctx.app.bundle_id.to_lowercase();

        self.allowlist.iter().any(|pattern| {
            let pattern_lower = pattern.to_lowercase();
            name_lower.contains(&pattern_lower) || bundle_lower.contains(&pattern_lower)
        })
    }

    /// Ajoute une app à l'allowlist
    pub fn add_to_allowlist(&mut self, app_name: String) {
        if !self.allowlist.contains(&app_name) {
            self.allowlist.push(app_name.clone());
            info!("➕ Added '{}' to allowlist", app_name);
        }
    }

    /// Retire une app de l'allowlist
    pub fn remove_from_allowlist(&mut self, app_name: &str) {
        self.allowlist.retain(|a| a != app_name);
        info!("➖ Removed '{}' from allowlist", app_name);
    }

    /// Récupère les stats
    pub fn get_stats(&self) -> TriggerStats {
        TriggerStats {
            total_triggers: self.total_triggers,
            triggers_per_app: self.trigger_count.clone(),
            current_cooldown_ms: self.get_current_cooldown_ms(),
            allowlist: self.allowlist.clone(),
            cooldown_base_ms: self.cooldown_base.as_millis() as u64,
            cooldown_dismiss_ms: self.cooldown_dismiss.as_millis() as u64,
        }
    }

    fn get_current_cooldown_ms(&self) -> Option<u64> {
        self.last_trigger.map(|last| {
            let cooldown = if self.last_dismiss.is_some() {
                self.cooldown_dismiss
            } else {
                self.cooldown_base
            };
            let remaining = cooldown.saturating_sub(last.elapsed());
            remaining.as_millis() as u64
        })
    }

    // J16: Anti-spam & UX methods

    /// Vérifie si une app est mutée
    pub fn is_app_muted(&self, app_name: &str) -> bool {
        if let Some(mute_time) = self.muted_apps.get(app_name) {
            mute_time.elapsed() < self.mute_duration
        } else {
            false
        }
    }

    /// Enregistre un trigger ignoré (pour stats seulement, PAS de mute auto)
    pub fn record_ignored_trigger(&mut self, app_name: &str) {
        let count = self
            .ignored_triggers
            .entry(app_name.to_string())
            .or_insert(0);
        *count += 1;
        
        // PAS d'auto-mute - juste comptage pour stats
        debug!("📊 Ignored trigger #{count} for app '{}'", app_name);
    }

    /// Mute une app manuellement
    pub fn mute_app(&mut self, app_name: &str) {
        self.muted_apps.insert(app_name.to_string(), Instant::now());
        info!("🔇 Manually muted app '{}' for 10 minutes", app_name);
    }

    /// Dé-mute une app immédiatement
    pub fn unmute_app(&mut self, app_name: &str) {
        self.muted_apps.remove(app_name);
        self.ignored_triggers.remove(app_name); // Reset le compteur aussi
        info!("🔊 Unmuted app '{}'", app_name);
    }

    /// Enregistre un dismiss
    pub fn record_dismiss(&mut self) {
        self.dismissed_count += 1;
        self.last_dismiss = Some(Instant::now());
    }

    /// Enregistre un snooze
    pub fn record_snooze(&mut self) {
        self.snoozed_count += 1;
    }

    /// Reset les compteurs ignorés pour une app (quand trigger accepté)
    pub fn reset_ignored_count(&mut self, app_name: &str) {
        self.ignored_triggers.remove(app_name);
    }

    /// Nettoie les mutes expirés
    pub fn cleanup_expired_mutes(&mut self) {
        let now = Instant::now();
        self.muted_apps
            .retain(|_, mute_time| now.duration_since(*mute_time) < self.mute_duration);
    }

    /// J16: Marquer la bulle comme visible/invisible
    pub fn set_bubble_visible(&mut self, visible: bool) {
        self.bubble_visible = visible;
        if visible {
            info!("🟢 Bulle visible");
        } else {
            info!("🔴 Bulle cachée");
        }
    }

    /// J16: Enregistrer une interaction utilisateur (verrou 45s)
    pub fn record_interaction(&mut self) {
        self.interaction_lock = Some(Instant::now());
        info!("🔒 Verrou interaction activé (45s)");
    }

    /// J16: Vérifier si interaction lock actif
    pub fn is_interaction_locked(&self) -> bool {
        if let Some(lock_time) = self.interaction_lock {
            lock_time.elapsed() < Duration::from_secs(45)
        } else {
            false
        }
    }

    /// J16: Obtenir le nombre de triggers ignorés pour une app
    pub fn get_ignored_count(&self, app_name: &str) -> usize {
        self.ignored_triggers.get(app_name).copied().unwrap_or(0)
    }
    pub fn get_interaction_lock_remaining(&self) -> Option<Duration> {
        if let Some(lock_time) = self.interaction_lock {
            let elapsed = lock_time.elapsed();
            if elapsed < Duration::from_secs(45) {
                Some(Duration::from_secs(45).saturating_sub(elapsed))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Récupère les stats étendues (J16)
    pub fn get_extended_stats(&self) -> ExtendedTriggerStats {
        ExtendedTriggerStats {
            total_triggers: self.total_triggers,
            triggers_per_app: self.trigger_count.clone(),
            current_cooldown_ms: self.get_current_cooldown_ms(),
            allowlist: self.allowlist.clone(),
            cooldown_base_ms: self.cooldown_base.as_millis() as u64,
            cooldown_dismiss_ms: self.cooldown_dismiss.as_millis() as u64,

            // J16: Nouveaux compteurs
            dismissed_count: self.dismissed_count,
            snoozed_count: self.snoozed_count,
            ignored_per_app: self.ignored_triggers.clone(),
            muted_apps: self.muted_apps.keys().cloned().collect(),
        }
    }
}

/// Décision de trigger (DTO sérialisable)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TriggerDecision {
    Allow,
    Debouncing { wait_ms: u64 },
    Rejected(RejectReason),
}

/// Raison de rejet
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum RejectReason {
    NotAllowlisted,
    Cooldown { remaining_ms: u64 },
    NotIdle,
    Muted, // J16: App mutée
}

/// Statistiques de triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerStats {
    pub total_triggers: usize,
    pub triggers_per_app: HashMap<String, usize>,
    pub current_cooldown_ms: Option<u64>,
    pub allowlist: Vec<String>,
    pub cooldown_base_ms: u64,
    pub cooldown_dismiss_ms: u64,
}

/// Statistiques étendues (J16: Anti-spam & UX)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedTriggerStats {
    pub total_triggers: usize,
    pub triggers_per_app: HashMap<String, usize>,
    pub current_cooldown_ms: Option<u64>,
    pub allowlist: Vec<String>,
    pub cooldown_base_ms: u64,
    pub cooldown_dismiss_ms: u64,

    // J16: Nouveaux compteurs
    pub dismissed_count: usize,
    pub snoozed_count: usize,
    pub ignored_per_app: HashMap<String, usize>,
    pub muted_apps: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::app_detector::ActiveApp;
    use std::time::Instant;

    fn mock_context(app_name: &str, idle_secs: f64) -> Context {
        Context {
            id: "test".into(),
            app: ActiveApp {
                name: app_name.into(),
                bundle_id: app_name.to_lowercase(),
                window_title: "Test".into(),
                timestamp: Instant::now(),
            },
            clipboard: None,
            idle_seconds: idle_secs,
            timestamp: 0,
        }
    }

    #[test]
    fn test_allowlist_blocks_unknown_apps() {
        let manager = TriggerManager::new();
        let ctx = mock_context("Unknown App", 15.0);

        matches!(
            manager.should_trigger(&ctx),
            TriggerDecision::Rejected(RejectReason::NotAllowlisted)
        );
    }

    #[test]
    fn test_idle_threshold() {
        let manager = TriggerManager::new();
        let ctx = mock_context("Cursor", 10.0); // < 12s

        matches!(
            manager.should_trigger(&ctx),
            TriggerDecision::Rejected(RejectReason::NotIdle)
        );
    }

    #[test]
    fn test_debouncing() {
        let manager = TriggerManager::new();
        let ctx = mock_context("Cursor", 13.0); // 12s < idle < 14s

        matches!(
            manager.should_trigger(&ctx),
            TriggerDecision::Debouncing { .. }
        );
    }

    #[test]
    fn test_allow() {
        let manager = TriggerManager::new();
        let ctx = mock_context("Cursor", 15.0); // > 14s

        matches!(manager.should_trigger(&ctx), TriggerDecision::Allow);
    }
}
