use super::change_detector::ChangeDetector;
use super::vision_client::ClaudeVisionClient;
use super::ocr_client::LocalOCR;
use super::smart_cache::SmartCache;
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
    /// Activer l'analyse par Claude Vision (cloud API)
    pub use_vision: bool,
    /// Activer l'OCR local (pattern detection rapide, gratuit, privacy-first)
    pub use_local_ocr: bool,
    /// Activer le monitoring
    pub enabled: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_secs: 5,
            similarity_threshold: 0.85,
            use_vision: false, // Cloud API - désactivé par défaut
            use_local_ocr: true, // OCR local - activé par défaut (gratuit et rapide)
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
    ocr_client: Arc<Mutex<Option<LocalOCR>>>,
    smart_cache: Arc<Mutex<SmartCache>>,
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

        // Init local OCR if enabled
        let ocr_client = if config.use_local_ocr {
            info!("✅ Local OCR client initialized (pattern detection)");
            Some(LocalOCR::new())
        } else {
            None
        };

        Self {
            change_detector: Arc::new(Mutex::new(ChangeDetector::new(config.similarity_threshold))),
            capturer: Arc::new(Mutex::new(None)),
            vision_client: Arc::new(Mutex::new(vision_client)),
            ocr_client: Arc::new(Mutex::new(ocr_client)),
            smart_cache: Arc::new(Mutex::new(SmartCache::new(
                config.interval_secs,
                2,  // min interval: 2s
                30, // max interval: 30s
            ))),
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
        let ocr_client = self.ocr_client.clone();
        let smart_cache = self.smart_cache.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            loop {
                // Sleep avec intervalle adaptatif
                let sleep_duration = {
                    let cache = smart_cache.lock().await;
                    cache.adaptive_interval()
                };
                tokio::time::sleep(sleep_duration).await;

                // Check si on doit continuer
                if !*is_running.lock().await {
                    info!("🛑 Screen monitor stopped");
                    break;
                }

                if !config.enabled {
                    continue;
                }

                // Capture + détection avec smart cache
                match Self::capture_and_check_adaptive(
                    &app,
                    &capturer,
                    &change_detector,
                    &vision_client,
                    &ocr_client,
                    &smart_cache,
                ).await {
                    Ok(Some(change)) => {
                        info!("📸 Screen change detected, emitting event");
                        // Émettre un événement pour le frontend
                        if let Err(e) = app.emit("screen-change", &change) {
                            error!("Failed to emit screen-change event: {}", e);
                        }
                    }
                    Ok(None) => {
                        // Pas de changement ou cache a décidé de skip
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

    /// Capture l'écran et vérifie avec smart cache + détection de changement
    async fn capture_and_check_adaptive(
        app: &AppHandle,
        capturer: &Arc<Mutex<Option<ScreenshotCapturer>>>,
        change_detector: &Arc<Mutex<ChangeDetector>>,
        vision_client: &Arc<Mutex<Option<ClaudeVisionClient>>>,
        ocr_client: &Arc<Mutex<Option<LocalOCR>>>,
        smart_cache: &Arc<Mutex<SmartCache>>,
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

        // Vérifier si changement significatif ET obtenir le hash
        let (has_change, current_hash) = {
            let mut detector = change_detector.lock().await;
            detector.has_significant_change(&image_path)?
        };

        // Vérifier avec le smart cache si on doit analyser
        let should_analyze = {
            let mut cache = smart_cache.lock().await;
            cache.should_analyze(current_hash)
        };

        if !should_analyze {
            // Cache dit de skip (écran identique récent)
            return Ok(None);
        }

        if !has_change {
            // Pas de changement significatif selon le seuil de similarité
            return Ok(None);
        }

        // Changement détecté ET cache approuve ! Analyser avec OCR local OU Claude Vision
        let analysis = {
            // Priorité à l'OCR local (rapide, gratuit, privacy-first)
            let ocr = ocr_client.lock().await;
            if let Some(ref local_ocr) = *ocr {
                match local_ocr.analyze(&image_path) {
                    Ok(ocr_result) => {
                        info!("✅ Local OCR: {} (confidence: {:.2})",
                              ocr_result.text, ocr_result.confidence);

                        // Générer une suggestion basée sur les patterns détectés
                        let suggestion = Self::generate_ocr_suggestion(&ocr_result);
                        Some(suggestion)
                    }
                    Err(e) => {
                        warn!("⚠️ Local OCR failed: {}", e);

                        // Fallback vers Claude Vision si disponible
                        let vision = vision_client.lock().await;
                        if let Some(ref claude) = *vision {
                            match claude.suggest_action(&capture_result.data).await {
                                Ok(suggestion) => {
                                    info!("✅ Claude Vision (fallback): {}", suggestion);
                                    Some(suggestion)
                                }
                                Err(e) => {
                                    warn!("⚠️ Vision analysis also failed: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                }
            } else {
                // Pas d'OCR local, essayer Claude Vision
                let vision = vision_client.lock().await;
                if let Some(ref claude) = *vision {
                    match claude.suggest_action(&capture_result.data).await {
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

    /// Capture l'écran et vérifie si un changement significatif s'est produit
    /// (Version legacy sans smart cache, gardée pour compatibilité)
    #[allow(dead_code)]
    async fn capture_and_check(
        app: &AppHandle,
        capturer: &Arc<Mutex<Option<ScreenshotCapturer>>>,
        change_detector: &Arc<Mutex<ChangeDetector>>,
        vision_client: &Arc<Mutex<Option<ClaudeVisionClient>>>,
        ocr_client: &Arc<Mutex<Option<LocalOCR>>>,
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
        let (has_change, _current_hash) = {
            let mut detector = change_detector.lock().await;
            detector.has_significant_change(&image_path)?
        };

        if !has_change {
            return Ok(None);
        }

        // Changement détecté ! Analyser avec OCR local OU Claude Vision
        let analysis = {
            // Priorité à l'OCR local (rapide, gratuit, privacy-first)
            let ocr = ocr_client.lock().await;
            if let Some(ref local_ocr) = *ocr {
                match local_ocr.analyze(&image_path) {
                    Ok(ocr_result) => {
                        info!("✅ Local OCR: {} (confidence: {:.2})",
                              ocr_result.text, ocr_result.confidence);

                        // Générer une suggestion basée sur les patterns détectés
                        let suggestion = Self::generate_ocr_suggestion(&ocr_result);
                        Some(suggestion)
                    }
                    Err(e) => {
                        warn!("⚠️ Local OCR failed: {}", e);

                        // Fallback vers Claude Vision si disponible
                        let vision = vision_client.lock().await;
                        if let Some(ref claude) = *vision {
                            match claude.suggest_action(&capture_result.data).await {
                                Ok(suggestion) => {
                                    info!("✅ Claude Vision (fallback): {}", suggestion);
                                    Some(suggestion)
                                }
                                Err(e) => {
                                    warn!("⚠️ Vision analysis also failed: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                }
            } else {
                // Pas d'OCR local, essayer Claude Vision
                let vision = vision_client.lock().await;
                if let Some(ref claude) = *vision {
                    match claude.suggest_action(&capture_result.data).await {
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

    /// Reset le smart cache
    pub async fn reset_cache(&self) {
        let mut cache = self.smart_cache.lock().await;
        cache.reset();
    }

    /// Obtenir les stats du smart cache
    pub async fn cache_stats(&self) -> super::smart_cache::CacheStats {
        let cache = self.smart_cache.lock().await;
        cache.stats()
    }

    /// Check si le monitor tourne
    pub async fn is_running(&self) -> bool {
        *self.is_running.lock().await
    }

    /// Génère une suggestion intelligente basée sur le résultat OCR
    fn generate_ocr_suggestion(ocr_result: &crate::monitor::OCRResult) -> String {
        use crate::monitor::DetectedPattern;

        let mut suggestions = Vec::new();

        for pattern in &ocr_result.detected_patterns {
            match pattern {
                DetectedPattern::CodeEditor { language, has_errors } => {
                    if *has_errors {
                        let lang = language.as_deref().unwrap_or("code");
                        suggestions.push(format!(
                            "Erreur détectée dans ton {} ! Je peux t'aider à la corriger ?",
                            lang
                        ));
                    } else {
                        suggestions.push(
                            "Tu codes ? Je peux suggérer des améliorations ou générer des tests.".to_string()
                        );
                    }
                }
                DetectedPattern::Terminal { has_errors, error_type } => {
                    if *has_errors {
                        let err = error_type.as_deref().unwrap_or("erreur");
                        suggestions.push(format!(
                            "Commande échouée ({}) - besoin d'aide pour débugger ?",
                            err
                        ));
                    } else {
                        suggestions.push(
                            "Terminal actif - je peux suggérer des commandes optimisées.".to_string()
                        );
                    }
                }
                DetectedPattern::Browser { has_stack_trace } => {
                    if *has_stack_trace {
                        suggestions.push(
                            "Stack trace détecté ! Je peux analyser l'erreur et proposer une solution.".to_string()
                        );
                    } else {
                        suggestions.push(
                            "Navigation web - je peux résumer la page ou extraire du contenu.".to_string()
                        );
                    }
                }
                DetectedPattern::IDE { name } => {
                    suggestions.push(format!(
                        "Tu utilises {} - besoin d'aide avec ton workflow ?",
                        name
                    ));
                }
            }
        }

        if suggestions.is_empty() {
            format!("Activité détectée : {}. Besoin d'aide ?", ocr_result.text)
        } else {
            suggestions.join(" ")
        }
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
