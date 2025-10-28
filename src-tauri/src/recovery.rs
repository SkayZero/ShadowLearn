use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Gestionnaire de récupération automatique des composants
pub struct RecoveryManager {
    idle_restarts: Arc<AtomicU32>,
    screenshot_restarts: Arc<AtomicU32>,
    max_restarts: u32,
}

impl RecoveryManager {
    /// Crée un nouveau gestionnaire de récupération
    pub fn new() -> Self {
        Self {
            idle_restarts: Arc::new(AtomicU32::new(0)),
            screenshot_restarts: Arc::new(AtomicU32::new(0)),
            max_restarts: 3, // Maximum 3 tentatives de redémarrage
        }
    }

    /// Obtient les statistiques de récupération
    pub fn get_stats(&self) -> RecoveryStats {
        RecoveryStats {
            idle_restarts: self.idle_restarts.load(Ordering::SeqCst),
            screenshot_restarts: self.screenshot_restarts.load(Ordering::SeqCst),
            max_restarts: self.max_restarts,
        }
    }
}

/// Statistiques de récupération
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    pub idle_restarts: u32,
    pub screenshot_restarts: u32,
    pub max_restarts: u32,
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_manager_creation() {
        let manager = RecoveryManager::new();
        assert_eq!(manager.get_restart_count(Component::IdleDetector), 0);
        assert_eq!(manager.get_restart_count(Component::Screenshot), 0);
    }

    #[test]
    fn test_recovery_stats() {
        let manager = RecoveryManager::new();
        let stats = manager.get_stats();

        assert_eq!(stats.idle_restarts, 0);
        assert_eq!(stats.screenshot_restarts, 0);
        assert_eq!(stats.max_restarts, 3);
    }
}
