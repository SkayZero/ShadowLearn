use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Keyboard,
    Mouse,
    Scroll,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleState {
    pub os_idle_seconds: f64,
    pub local_idle_seconds: f64,
    pub effective_idle_seconds: f64,
    pub last_activity_type: ActivityType,
}

pub struct IdleDetector {
    last_activity: Instant,
    last_activity_type: ActivityType,
}

impl IdleDetector {
    pub fn new() -> Self {
        Self {
            last_activity: Instant::now(),
            last_activity_type: ActivityType::Unknown,
        }
    }

    /// Récupère le nombre de secondes d'inactivité locale (depuis dernière activité app)
    pub fn get_local_idle_seconds(&self) -> f64 {
        self.last_activity.elapsed().as_secs_f64()
    }

    /// Récupère l'état complet de l'idle (OS + local + effective)
    pub fn get_idle_state(&self) -> IdleState {
        let local_idle = self.get_local_idle_seconds();
        let os_idle = local_idle; // TODO: Remplacer par CGEventSource sur macOS
        let effective_idle = os_idle.min(local_idle);

        debug!(
            "🕐 Idle state: OS={:.1}s, Local={:.1}s, Effective={:.1}s",
            os_idle, local_idle, effective_idle
        );

        IdleState {
            os_idle_seconds: os_idle,
            local_idle_seconds: local_idle,
            effective_idle_seconds: effective_idle,
            last_activity_type: self.last_activity_type,
        }
    }

    /// Récupère le nombre de secondes d'inactivité effective (min entre OS et local)
    pub fn get_idle_seconds(&self) -> f64 {
        self.get_idle_state().effective_idle_seconds
    }

    /// Reset l'activité avec type (appelé sur événements dans l'app)
    pub fn reset_activity(&mut self, activity_type: ActivityType) {
        self.last_activity = Instant::now();
        self.last_activity_type = activity_type;
        debug!("🔄 Activity reset: {:?}", activity_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_detector_creation() {
        let detector = IdleDetector::new();
        let idle = detector.get_idle_seconds();
        assert!(idle >= 0.0);
    }
}
