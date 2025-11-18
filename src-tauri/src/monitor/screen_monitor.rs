use super::change_detector::ChangeDetector;
use super::vision_client::ClaudeVisionClient;
use crate::screenshot::ScreenshotCapturer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Intervalle de capture en secondes
    pub interval_secs: u64,
    /// Seuil de similarité (0.0 = différent, 1.0 = identique)
    pub similarity_threshold: f32,
    /// Activer l'analyse par Claude Vision
    pub use_vision: bool,
    /// Activer le monitoring
    pub enabled: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_secs: 5,
            similarity_threshold: 0.85,
            use_vision: false, // Désactivé par défaut (peut être activé avec ANTHROPIC_API_KEY)
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenChange {
    pub timestamp: u64,
    pub image_path: String,
    pub image_base64: String,
    pub analysis: Option<String>, // Résultat de l'analyse Vision (si activée)
}

pub struct ScreenMonitor {
    config: MonitorConfig,
    change_detector: Arc<Mutex<ChangeDetector>>,
    capturer: Arc<Mutex<Option<ScreenshotCapturer>>>,
    vision_client: Arc<Mutex<Option<ClaudeVisionClient>>>,
    is_running: Arc<Mutex<bool>>,
}

impl ScreenMonitor {
    pub fn new(config: MonitorConfig) -> Self {
        // Try to init Claude Vision client if API key is available
        let vision_client = if config.use_vision {
            match ClaudeVisionClient::new() {
                Ok(client) => {
                    info!("✅ Claude Vision client initialized");
                    Some(client)
                }
                Err(e) => {
                    warn!("⚠️ Claude Vision client init failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        Self {
            change_detector: Arc::new(Mutex::new(ChangeDetector::new(config.similarity_threshold))),
            capturer: Arc::new(Mutex::new(None)),
            vision_client: Arc::new(Mutex::new(vision_client)),
            is_running: Arc::new(Mutex::new(false)),
            config,
        }
    }

    /// Démarre la boucle de monitoring
    pub async fn start(&self, app: AppHandle) {
        let mut is_running = self.is_running.lock().await;

        if *is_running {
            warn!("⚠️ Screen monitor already running");
            return;
        }

        *is_running = true;
        drop(is_running);

        info!("🎬 Starting screen monitor (interval: {}s)", self.config.interval_secs);

        // Clone pour le spawn
        let config = self.config.clone();
        let change_detector = self.change_detector.clone();
        let capturer = self.capturer.clone();
        let vision_client = self.vision_client.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.interval_secs));

            loop {
                interval.tick().await;

                // Check si on doit continuer
                if !*is_running.lock().await {
                    info!("🛑 Screen monitor stopped");
                    break;
                }

                if !config.enabled {
                    continue;
                }

                // Capture + détection
                match Self::capture_and_check(
                    &app,
                    &capturer,
                    &change_detector,
                    &vision_client,
                ).await {
                    Ok(Some(change)) => {
                        info!("📸 Screen change detected, emitting event");
                        // Émettre un événement pour le frontend
                        if let Err(e) = app.emit("screen-change", &change) {
                            error!("Failed to emit screen-change event: {}", e);
                        }
                    }
                    Ok(None) => {
                        // Pas de changement
                    }
                    Err(e) => {
                        error!("❌ Screen capture/check failed: {}", e);
                    }
                }
            }
        });
    }

    /// Stop la boucle de monitoring
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        info!("🛑 Screen monitor stop requested");
    }

    /// Capture l'écran et vérifie si un changement significatif s'est produit
    async fn capture_and_check(
        app: &AppHandle,
        capturer: &Arc<Mutex<Option<ScreenshotCapturer>>>,
        change_detector: &Arc<Mutex<ChangeDetector>>,
        vision_client: &Arc<Mutex<Option<ClaudeVisionClient>>>,
    ) -> Result<Option<ScreenChange>, String> {
        // Initialiser le capturer si nécessaire
        {
            let mut capturer_guard = capturer.lock().await;
            if capturer_guard.is_none() {
                *capturer_guard = Some(
                    ScreenshotCapturer::new()
                        .map_err(|e| format!("Failed to init capturer: {}", e))?,
                );
            }
        }

        // Capture via le Tauri command existant
        let capture_result = crate::screenshot::capture_screenshot(app.clone())
            .await
            .map_err(|e| format!("Capture failed: {}", e))?;

        let image_path = std::path::PathBuf::from(&capture_result.path);

        // Vérifier si changement significatif
        let has_change = {
            let mut detector = change_detector.lock().await;
            detector.has_significant_change(&image_path)?
        };

        if !has_change {
            return Ok(None);
        }

        // Changement détecté ! Analyser avec Claude Vision si disponible
        let analysis = {
            let client = vision_client.lock().await;
            if let Some(ref vision) = *client {
                match vision.suggest_action(&capture_result.data).await {
                    Ok(suggestion) => {
                        info!("✅ Claude Vision suggestion: {}", suggestion);
                        Some(suggestion)
                    }
                    Err(e) => {
                        warn!("⚠️ Vision analysis failed: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        };

        Ok(Some(ScreenChange {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            image_path: capture_result.path,
            image_base64: capture_result.data,
            analysis,
        }))
    }

    /// Reset le détecteur de changement
    pub async fn reset_detector(&self) {
        let mut detector = self.change_detector.lock().await;
        detector.reset();
    }

    /// Check si le monitor tourne
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MonitorConfig::default();
        assert_eq!(config.interval_secs, 5);
        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.use_vision, false);
        assert_eq!(config.enabled, true);
    }
}
