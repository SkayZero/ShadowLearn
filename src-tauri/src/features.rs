use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn};

/// Features disponibles dans l'application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Feature {
    IdleDetection,
    Screenshot,
    SmartTriggers,
    Telemetry,
    UseIntentGate,
}

impl Feature {
    pub fn display_name(&self) -> &'static str {
        match self {
            Feature::IdleDetection => "Idle Detection",
            Feature::Screenshot => "Screenshot",
            Feature::SmartTriggers => "Smart Triggers",
            Feature::Telemetry => "Telemetry",
            Feature::UseIntentGate => "Intent Gate",
        }
    }
}

/// Gestionnaire des feature flags avec support de dépendances
pub struct FeatureFlags {
    idle_detection: AtomicBool,
    screenshot: AtomicBool,
    smart_triggers: AtomicBool,
    telemetry: AtomicBool,
    use_intent_gate: AtomicBool,
}

impl FeatureFlags {
    /// Crée une nouvelle instance à partir des variables d'environnement
    pub fn from_env() -> Self {
        let flags = Self {
            idle_detection: AtomicBool::new(env_bool("SL_IDLE_ENABLED", true)),
            screenshot: AtomicBool::new(env_bool("SL_SCREENSHOT_ENABLED", false)),
            smart_triggers: AtomicBool::new(env_bool("SL_SMART_TRIGGERS_ENABLED", true)),
            telemetry: AtomicBool::new(env_bool("SL_TELEMETRY", true)),
            use_intent_gate: AtomicBool::new(env_bool("SL_USE_INTENT_GATE", true)),
        };

        // Log initial state
        info!("🚩 Feature flags initialized:");
        info!(
            "  ├─ Idle Detection: {}",
            flags.is_enabled(Feature::IdleDetection)
        );
        info!("  ├─ Screenshot: {}", flags.is_enabled(Feature::Screenshot));
        info!(
            "  ├─ Smart Triggers: {}",
            flags.is_enabled(Feature::SmartTriggers)
        );
        info!("  ├─ Telemetry: {}", flags.is_enabled(Feature::Telemetry));
        info!(
            "  └─ Intent Gate: {}",
            flags.is_enabled(Feature::UseIntentGate)
        );

        // Vérifier les dépendances au démarrage
        if flags.is_enabled(Feature::SmartTriggers) && !flags.is_enabled(Feature::IdleDetection) {
            warn!(
                "⚠️  Smart Triggers enabled but Idle Detection disabled - disabling Smart Triggers"
            );
            flags.smart_triggers.store(false, Ordering::Release);
        }

        flags
    }

    /// Vérifie si une feature est activée
    pub fn is_enabled(&self, feature: Feature) -> bool {
        match feature {
            Feature::IdleDetection => self.idle_detection.load(Ordering::Acquire),
            Feature::Screenshot => self.screenshot.load(Ordering::Acquire),
            Feature::SmartTriggers => self.smart_triggers.load(Ordering::Acquire),
            Feature::Telemetry => self.telemetry.load(Ordering::Acquire),
            Feature::UseIntentGate => self.use_intent_gate.load(Ordering::Acquire),
        }
    }

    /// Désactive une feature avec cascade de dépendances
    pub fn disable(&self, feature: Feature) {
        match feature {
            Feature::IdleDetection => {
                warn!("🚫 Disabling {}", feature.display_name());
                self.idle_detection.store(false, Ordering::Release);

                // Cascade: SmartTriggers dépend de IdleDetection
                if self.smart_triggers.load(Ordering::Acquire) {
                    warn!("⚠️  Cascading: disabling Smart Triggers (depends on Idle Detection)");
                    self.smart_triggers.store(false, Ordering::Release);
                }
            }
            Feature::Screenshot => {
                warn!("🚫 Disabling {}", feature.display_name());
                self.screenshot.store(false, Ordering::Release);
            }
            Feature::SmartTriggers => {
                warn!("🚫 Disabling {}", feature.display_name());
                self.smart_triggers.store(false, Ordering::Release);
            }
            Feature::Telemetry => {
                info!("🚫 Disabling {}", feature.display_name());
                self.telemetry.store(false, Ordering::Release);
            }
            Feature::UseIntentGate => {
                info!("🚫 Disabling {}", feature.display_name());
                self.use_intent_gate.store(false, Ordering::Release);
            }
        }
    }

    /// Active une feature avec vérification des dépendances
    pub fn enable(&self, feature: Feature) {
        match feature {
            Feature::SmartTriggers => {
                if self.idle_detection.load(Ordering::Acquire) {
                    info!("✅ Re-enabling {}", feature.display_name());
                    self.smart_triggers.store(true, Ordering::Release);
                } else {
                    warn!(
                        "❌ Cannot enable {}: Idle Detection is disabled",
                        feature.display_name()
                    );
                }
            }
            Feature::IdleDetection => {
                info!("✅ Re-enabling {}", feature.display_name());
                self.idle_detection.store(true, Ordering::Release);
            }
            Feature::Screenshot => {
                info!("✅ Re-enabling {}", feature.display_name());
                self.screenshot.store(true, Ordering::Release);
            }
            Feature::Telemetry => {
                info!("✅ Re-enabling {}", feature.display_name());
                self.telemetry.store(true, Ordering::Release);
            }
            Feature::UseIntentGate => {
                info!("✅ Re-enabling {}", feature.display_name());
                self.use_intent_gate.store(true, Ordering::Release);
            }
        }
    }

    /// Récupère l'état de toutes les features
    pub fn get_state(&self) -> FeaturesState {
        FeaturesState {
            idle_detection: self.is_enabled(Feature::IdleDetection),
            screenshot: self.is_enabled(Feature::Screenshot),
            smart_triggers: self.is_enabled(Feature::SmartTriggers),
            telemetry: self.is_enabled(Feature::Telemetry),
            use_intent_gate: self.is_enabled(Feature::UseIntentGate),
        }
    }

    /// Vérifie si une feature peut être activée (dépendances satisfaites)
    pub fn can_enable(&self, feature: Feature) -> bool {
        match feature {
            Feature::SmartTriggers => self.is_enabled(Feature::IdleDetection),
            _ => true, // Autres features n'ont pas de dépendances
        }
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::from_env()
    }
}

/// État des features pour sérialisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesState {
    pub idle_detection: bool,
    pub screenshot: bool,
    pub smart_triggers: bool,
    pub telemetry: bool,
    pub use_intent_gate: bool,
}

impl FeaturesState {
    /// Compte le nombre de features activées
    pub fn enabled_count(&self) -> usize {
        [
            self.idle_detection,
            self.screenshot,
            self.smart_triggers,
            self.telemetry,
            self.use_intent_gate,
        ]
        .iter()
        .filter(|&&enabled| enabled)
        .count()
    }
}

/// Parse une variable d'environnement en booléen
fn env_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => {
                warn!(
                    "⚠️  Invalid value for {}: '{}', using default: {}",
                    key, v, default
                );
                None
            }
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disable_feature() {
        let flags = FeatureFlags::from_env();
        flags.disable(Feature::Screenshot);
        assert!(!flags.is_enabled(Feature::Screenshot));
    }

    #[test]
    fn test_cascade_disable_idle_to_smart_triggers() {
        let flags = FeatureFlags::from_env();

        // Enable both first
        flags.enable(Feature::IdleDetection);
        flags.enable(Feature::SmartTriggers);

        // Disable idle detection
        flags.disable(Feature::IdleDetection);

        // Smart triggers should be auto-disabled
        assert!(!flags.is_enabled(Feature::IdleDetection));
        assert!(!flags.is_enabled(Feature::SmartTriggers));
    }

    #[test]
    fn test_cannot_enable_smart_triggers_without_idle() {
        let flags = FeatureFlags::from_env();

        // Disable idle detection
        flags.disable(Feature::IdleDetection);

        // Try to enable smart triggers
        flags.enable(Feature::SmartTriggers);

        // Should fail
        assert!(!flags.is_enabled(Feature::SmartTriggers));
    }

    #[test]
    fn test_can_enable_check() {
        let flags = FeatureFlags::from_env();

        flags.disable(Feature::IdleDetection);
        assert!(!flags.can_enable(Feature::SmartTriggers));

        flags.enable(Feature::IdleDetection);
        assert!(flags.can_enable(Feature::SmartTriggers));
    }

    #[test]
    fn test_get_dependencies() {
        let flags = FeatureFlags::from_env();

        let deps = flags.get_dependencies(Feature::SmartTriggers);
        assert_eq!(deps, vec![Feature::IdleDetection]);

        let no_deps = flags.get_dependencies(Feature::Screenshot);
        assert!(no_deps.is_empty());
    }

    #[test]
    fn test_features_state() {
        let state = FeaturesState {
            idle_detection: true,
            screenshot: false,
            smart_triggers: true,
            telemetry: true,
        };

        assert_eq!(state.enabled_count(), 3);
        assert!(!state.all_enabled());
    }

    #[test]
    fn test_env_bool_parsing() {
        assert!(env_bool("NONEXISTENT", true));
        assert!(!env_bool("NONEXISTENT", false));
    }
}
