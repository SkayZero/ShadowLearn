use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Type de zone de confidentialité
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum PrivacyZone {
    /// Zone rectangulaire définie par coordonnées
    Rectangle {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        label: String,
    },
    /// Fenêtre d'application spécifique (nom de l'app)
    Window {
        app_name: String,
        fuzzy_match: bool, // Match partiel (ex: "Chrome" match "Google Chrome")
    },
    /// Région prédéfinie (barre de tâches, menu système, etc.)
    Region {
        region: PredefinedRegion,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PredefinedRegion {
    TopBar,       // Barre supérieure (menu, URL bar)
    Taskbar,      // Barre des tâches
    SystemTray,   // Zone de notification système
    Dock,         // Dock macOS / Linux
}

/// Configuration des zones de confidentialité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyZonesConfig {
    pub enabled: bool,
    pub zones: Vec<PrivacyZone>,
    pub blur_instead_of_skip: bool, // Si true, flouter au lieu de skip
}

impl Default for PrivacyZonesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            zones: vec![
                // Par défaut: protéger la barre URL/menu (top 50px)
                PrivacyZone::Rectangle {
                    x: 0,
                    y: 0,
                    width: 9999, // Full width
                    height: 50,
                    label: "Top Menu Bar".to_string(),
                },
            ],
            blur_instead_of_skip: false,
        }
    }
}

pub struct PrivacyZoneManager {
    config: PrivacyZonesConfig,
    // Cache des fenêtres bloquées (app_name -> blocked)
    blocked_apps: HashMap<String, bool>,
}

impl PrivacyZoneManager {
    pub fn new(config: PrivacyZonesConfig) -> Self {
        Self {
            config,
            blocked_apps: HashMap::new(),
        }
    }

    /// Vérifie si une zone de l'écran doit être protégée
    pub fn is_zone_protected(&self, x: u32, y: u32, width: u32, height: u32) -> bool {
        if !self.config.enabled {
            return false;
        }

        for zone in &self.config.zones {
            match zone {
                PrivacyZone::Rectangle {
                    x: zx,
                    y: zy,
                    width: zw,
                    height: zh,
                    ..
                } => {
                    // Check if rectangles overlap
                    if Self::rectangles_overlap(x, y, width, height, *zx, *zy, *zw, *zh) {
                        debug!("🔒 Zone protected by rectangle: {:?}", zone);
                        return true;
                    }
                }
                PrivacyZone::Region { region } => {
                    // Check predefined regions
                    if Self::is_in_predefined_region(x, y, width, height, region) {
                        debug!("🔒 Zone protected by region: {:?}", region);
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Vérifie si une application doit être protégée
    pub fn is_app_protected(&self, app_name: &str) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check cache first
        if let Some(&blocked) = self.blocked_apps.get(app_name) {
            return blocked;
        }

        for zone in &self.config.zones {
            if let PrivacyZone::Window {
                app_name: blocked_app,
                fuzzy_match,
            } = zone
            {
                let matches = if *fuzzy_match {
                    app_name.to_lowercase().contains(&blocked_app.to_lowercase())
                } else {
                    app_name.eq_ignore_ascii_case(blocked_app)
                };

                if matches {
                    info!("🔒 App protected: {} (zone: {})", app_name, blocked_app);
                    return true;
                }
            }
        }

        false
    }

    /// Ajoute une nouvelle zone de confidentialité
    pub fn add_zone(&mut self, zone: PrivacyZone) {
        if !self.config.zones.contains(&zone) {
            self.config.zones.push(zone.clone());
            info!("✅ Added privacy zone: {:?}", zone);
            self.clear_cache();
        }
    }

    /// Retire une zone de confidentialité
    pub fn remove_zone(&mut self, zone: &PrivacyZone) -> bool {
        let before = self.config.zones.len();
        self.config.zones.retain(|z| z != zone);
        let removed = before != self.config.zones.len();

        if removed {
            info!("✅ Removed privacy zone: {:?}", zone);
            self.clear_cache();
        }

        removed
    }

    /// Active/désactive les zones de confidentialité
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
        info!("🔒 Privacy zones {}", if enabled { "enabled" } else { "disabled" });
        self.clear_cache();
    }

    /// Obtenir la configuration actuelle
    pub fn config(&self) -> &PrivacyZonesConfig {
        &self.config
    }

    /// Clear cache (après modification de config)
    fn clear_cache(&mut self) {
        self.blocked_apps.clear();
    }

    /// Vérifie si deux rectangles se chevauchent
    fn rectangles_overlap(
        x1: u32,
        y1: u32,
        w1: u32,
        h1: u32,
        x2: u32,
        y2: u32,
        w2: u32,
        h2: u32,
    ) -> bool {
        !(x1 >= x2 + w2 || x2 >= x1 + w1 || y1 >= y2 + h2 || y2 >= y1 + h1)
    }

    /// Vérifie si une zone est dans une région prédéfinie
    fn is_in_predefined_region(
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        region: &PredefinedRegion,
    ) -> bool {
        match region {
            PredefinedRegion::TopBar => {
                // Top 80px on any platform
                y < 80
            }
            PredefinedRegion::Taskbar => {
                // Platform-specific taskbar detection
                #[cfg(target_os = "windows")]
                {
                    // Windows: bottom 48px typically
                    y + height > 2160 - 48 // Assume 4K, adjust dynamically later
                }
                #[cfg(not(target_os = "windows"))]
                {
                    false
                }
            }
            PredefinedRegion::SystemTray => {
                // Top-right corner (200x80)
                x + width > 2560 - 200 && y < 80
            }
            PredefinedRegion::Dock => {
                // macOS: bottom 80px center
                #[cfg(target_os = "macos")]
                {
                    y + height > 1440 - 80
                }
                #[cfg(not(target_os = "macos"))]
                {
                    false
                }
            }
        }
    }
}

impl Default for PrivacyZoneManager {
    fn default() -> Self {
        Self::new(PrivacyZonesConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rectangle_overlap() {
        // Overlap
        assert!(PrivacyZoneManager::rectangles_overlap(
            0, 0, 100, 100, 50, 50, 100, 100
        ));

        // No overlap
        assert!(!PrivacyZoneManager::rectangles_overlap(
            0, 0, 100, 100, 200, 200, 100, 100
        ));

        // Adjacent (no overlap)
        assert!(!PrivacyZoneManager::rectangles_overlap(
            0, 0, 100, 100, 100, 0, 100, 100
        ));
    }

    #[test]
    fn test_is_zone_protected() {
        let manager = PrivacyZoneManager::default();

        // Top bar (0-50px) should be protected by default
        assert!(manager.is_zone_protected(100, 10, 200, 30));

        // Below top bar should not be protected
        assert!(!manager.is_zone_protected(100, 100, 200, 200));
    }

    #[test]
    fn test_is_app_protected() {
        let mut config = PrivacyZonesConfig::default();
        config.zones.push(PrivacyZone::Window {
            app_name: "Banking App".to_string(),
            fuzzy_match: false,
        });
        config.zones.push(PrivacyZone::Window {
            app_name: "Password".to_string(),
            fuzzy_match: true,
        });

        let manager = PrivacyZoneManager::new(config);

        // Exact match
        assert!(manager.is_app_protected("Banking App"));
        assert!(!manager.is_app_protected("banking app")); // Case insensitive exact

        // Fuzzy match
        assert!(manager.is_app_protected("1Password"));
        assert!(manager.is_app_protected("KeePass Password Safe"));
    }

    #[test]
    fn test_add_remove_zone() {
        let mut manager = PrivacyZoneManager::default();

        let zone = PrivacyZone::Rectangle {
            x: 100,
            y: 100,
            width: 200,
            height: 200,
            label: "Test Zone".to_string(),
        };

        // Add zone
        let before = manager.config.zones.len();
        manager.add_zone(zone.clone());
        assert_eq!(manager.config.zones.len(), before + 1);

        // Remove zone
        assert!(manager.remove_zone(&zone));
        assert_eq!(manager.config.zones.len(), before);

        // Remove non-existent
        assert!(!manager.remove_zone(&zone));
    }
}
