use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Structure principale pour monitorer la santé des composants
pub struct HealthMonitor {
    last_check: Arc<AtomicU64>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            last_check: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Met à jour le timestamp du dernier check
    pub fn update_check(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_check.store(now, Ordering::SeqCst);
    }

    /// Récupère le timestamp du dernier check
    pub fn get_last_check(&self) -> u64 {
        self.last_check.load(Ordering::SeqCst)
    }

    /// Effectue un check complet de la santé du système
    pub async fn check_health(&self) -> HealthStatus {
        self.update_check();

        HealthStatus {
            idle_detector: self.check_idle_detector().await,
            screenshot: self.check_screenshot().await,
            permissions: self.check_permissions().await,
            timestamp: self.get_last_check(),
        }
    }

    /// Check la santé du détecteur d'inactivité
    async fn check_idle_detector(&self) -> ComponentHealth {
        // Pour l'instant, on retourne Healthy par défaut
        // TODO: Implémenter la vraie logique quand idle detector sera en place

        // Simulation: vérifier que le dernier check est récent
        let last_check = self.get_last_check();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if last_check == 0 {
            // Premier check
            ComponentHealth::Healthy
        } else if now - last_check < 2 {
            // Check récent (< 2s)
            ComponentHealth::Healthy
        } else if now - last_check < 10 {
            // Check un peu ancien (< 10s)
            ComponentHealth::Degraded {
                reason: format!("Last check {}s ago", now - last_check),
            }
        } else {
            // Check très ancien (> 10s)
            ComponentHealth::Down {
                reason: format!("No check for {}s", now - last_check),
            }
        }
    }

    /// Check la santé du système de screenshot
    async fn check_screenshot(&self) -> ComponentHealth {
        // Pour l'instant, on retourne Healthy par défaut
        // TODO: Implémenter un test de capture dummy
        ComponentHealth::Healthy
    }

    /// Check les permissions système
    async fn check_permissions(&self) -> PermissionStatus {
        PermissionStatus {
            screen_capture: self.check_screen_capture_permission(),
            accessibility: self.check_accessibility_permission(),
        }
    }

    /// Vérifie la permission de capture d'écran (macOS)
    fn check_screen_capture_permission(&self) -> bool {
        // Pour l'instant, on retourne true par défaut
        // TODO: Implémenter CGPreflightScreenCaptureAccess() pour macOS
        #[cfg(target_os = "macos")]
        {
            // Placeholder - à implémenter avec CGPreflightScreenCaptureAccess
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    /// Vérifie la permission d'accessibilité (macOS)
    fn check_accessibility_permission(&self) -> bool {
        // Pour l'instant, on retourne true par défaut
        // TODO: Implémenter AXIsProcessTrusted() pour macOS
        #[cfg(target_os = "macos")]
        {
            // Placeholder - à implémenter avec AXIsProcessTrusted
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Status de santé global du système
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub idle_detector: ComponentHealth,
    pub screenshot: ComponentHealth,
    pub permissions: PermissionStatus,
    pub timestamp: u64,
}

/// Status de santé d'un composant individuel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "reason")]
pub enum ComponentHealth {
    Healthy,
    Degraded { reason: String },
    Down { reason: String },
}

/// Status des permissions système
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionStatus {
    pub screen_capture: bool,
    pub accessibility: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        assert_eq!(monitor.get_last_check(), 0);
    }

    #[test]
    fn test_health_monitor_update() {
        let monitor = HealthMonitor::new();
        monitor.update_check();
        assert!(monitor.get_last_check() > 0);
    }

    #[test]
    fn test_component_health_status() {
        let healthy = ComponentHealth::Healthy;
        assert!(healthy.is_healthy());
        assert!(!healthy.is_degraded());
        assert!(!healthy.is_down());

        let degraded = ComponentHealth::Degraded {
            reason: "Test".to_string(),
        };
        assert!(!degraded.is_healthy());
        assert!(degraded.is_degraded());
        assert!(!degraded.is_down());

        let down = ComponentHealth::Down {
            reason: "Test".to_string(),
        };
        assert!(!down.is_healthy());
        assert!(!down.is_degraded());
        assert!(down.is_down());
    }

    #[tokio::test]
    async fn test_check_health() {
        let monitor = HealthMonitor::new();
        let status = monitor.check_health().await;

        assert!(status.idle_detector.is_healthy());
        assert!(status.screenshot.is_healthy());
        assert!(status.permissions.is_all_granted());
        assert!(status.timestamp > 0);
    }
}
