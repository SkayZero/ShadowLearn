use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::persistence::database::DatabaseManager;

/// Configuration du système de trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    pub rate_limit_window: Duration,
    pub rate_limit_max: usize,
    pub quarantine_threshold: f32,
    pub quarantine_min_events: f32,
    pub decay_factor: f32,
    pub anomaly_threshold: f32,
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            rate_limit_window: Duration::from_secs(60),
            rate_limit_max: 10,
            quarantine_threshold: 0.1,
            quarantine_min_events: 30.0,
            decay_factor: 0.95,
            anomaly_threshold: 3.0,
        }
    }
}

/// Événement de trust avec timestamp
#[derive(Debug, Clone)]
pub struct TrustEvent {
    pub id: String,
    pub timestamp: Instant,
    pub reward: f32,
    pub device_id: String,
}

/// Niveau de confiance utilisateur
#[derive(Debug, Clone)]
pub struct UserTrust {
    pub id: String,
    pub device_id: String,
    pub pos: f32,         // Feedback positif cumulé
    pub neg: f32,         // Feedback négatif cumulé
    pub trust: f32,       // Score de trust [0, 1]
    pub quarantine: bool, // En quarantaine
    pub last_updated: Instant,
    pub created_at: Instant,
}

/// Niveau de confiance avec métadonnées (sérialisable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustLevel {
    pub score: f32,
    pub confidence: f32, // Confiance dans le score
    pub sample_size: usize,
    pub last_updated_ms: u64, // Timestamp en millisecondes
    pub quarantine: bool,
}

/// Gestionnaire de trust avec rate limiting et quarantaine
pub struct TrustManager {
    db: Arc<Mutex<DatabaseManager>>,
    device_id: String,
    recent_events: VecDeque<TrustEvent>,
    config: TrustConfig,
}

impl TrustManager {
    pub fn new(db: Arc<Mutex<DatabaseManager>>, device_id: String) -> Self {
        Self {
            db,
            device_id,
            recent_events: VecDeque::new(),
            config: TrustConfig::default(),
        }
    }

    pub fn with_config(
        db: Arc<Mutex<DatabaseManager>>,
        device_id: String,
        config: TrustConfig,
    ) -> Self {
        Self {
            db,
            device_id,
            recent_events: VecDeque::new(),
            config,
        }
    }

    /// Obtenir le trust actuel de l'utilisateur
    pub async fn get_trust(&self) -> Result<UserTrust, String> {
        match self.db.lock().await.get_user_trust(&self.device_id).await {
            Ok(trust) => Ok(trust),
            Err(_) => {
                // Créer un nouveau trust si inexistant
                let new_trust = UserTrust {
                    id: Uuid::new_v4().to_string(),
                    device_id: self.device_id.clone(),
                    pos: 0.0,
                    neg: 0.0,
                    trust: 0.5, // Score neutre par défaut
                    quarantine: false,
                    last_updated: Instant::now(),
                    created_at: Instant::now(),
                };
                self.db.lock().await.create_user_trust(&new_trust).await?;
                Ok(new_trust)
            }
        }
    }

    /// Mettre à jour le trust à partir d'un reward
    pub async fn update_from_reward(&mut self, reward: f32) -> Result<f32, String> {
        // Validation critique des inputs
        if !reward.is_finite() || !(0.0..=1.0).contains(&reward) {
            return Err("Invalid reward value: must be finite and in [0, 1]".to_string());
        }

        // Nettoyer les anciens événements
        self.cleanup_old_events();

        // Vérifier le rate limiting
        if self.recent_events.len() >= self.config.rate_limit_max {
            warn!(
                "Rate limit exceeded for device {} ({} events in {}s)",
                self.device_id,
                self.recent_events.len(),
                self.config.rate_limit_window.as_secs()
            );
            return Err("Rate limit exceeded".to_string());
        }

        // Créer l'événement
        let event = TrustEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Instant::now(),
            reward,
            device_id: self.device_id.clone(),
        };

        // Ajouter à la queue récente
        self.recent_events.push_back(event.clone());

        // Sauvegarder l'événement
        self.db.lock().await.save_trust_event(&event).await?;

        // Obtenir le trust actuel
        let mut trust = self.get_trust().await?;

        // Appliquer le decay mensuel si nécessaire
        self.apply_decay(&mut trust);

        // Mettre à jour Beta distribution
        if reward >= 0.6 {
            trust.pos += reward;
        } else {
            trust.neg += 1.0 - reward;
        }

        // Recalculer le score de trust
        let total_events = trust.pos + trust.neg;
        if total_events > 0.0 {
            trust.trust = trust.pos / total_events;
        } else {
            trust.trust = 0.5; // Score neutre
        }

        // Vérifier la quarantaine
        if trust.trust < self.config.quarantine_threshold
            && total_events > self.config.quarantine_min_events
        {
            trust.quarantine = true;
            warn!(
                "Device {} quarantined (trust={:.3}, events={:.1})",
                self.device_id, trust.trust, total_events
            );
        }

        trust.last_updated = Instant::now();

        // Sauvegarder le trust mis à jour
        self.db.lock().await.update_user_trust(&trust).await?;

        info!(
            "Trust updated for device {}: {:.3} (pos={:.1}, neg={:.1})",
            self.device_id, trust.trust, trust.pos, trust.neg
        );

        Ok(trust.trust)
    }

    /// Obtenir le niveau de confiance avec métadonnées
    pub async fn get_trust_level(&self) -> Result<TrustLevel, String> {
        let trust = self.get_trust().await?;
        let sample_size = (trust.pos + trust.neg) as usize;

        let confidence = if sample_size < 10 {
            0.3 // Faible confiance
        } else if sample_size < 50 {
            0.7 // Confiance moyenne
        } else {
            1.0 // Haute confiance
        };

        Ok(TrustLevel {
            score: trust.trust,
            confidence,
            sample_size,
            last_updated_ms: trust.last_updated.elapsed().as_millis() as u64,
            quarantine: trust.quarantine,
        })
    }

    /// Obtenir le poids de trust pour pondérer les rewards
    pub fn get_trust_weight(&self, trust: f32) -> f32 {
        trust.clamp(0.2, 1.2)
    }

    /// Vérifier si l'utilisateur est en quarantaine
    pub async fn is_quarantined(&self) -> Result<bool, String> {
        let trust = self.get_trust().await?;
        Ok(trust.quarantine)
    }

    /// Nettoyer les événements anciens
    fn cleanup_old_events(&mut self) {
        let cutoff = Instant::now() - self.config.rate_limit_window;
        while let Some(event) = self.recent_events.front() {
            if event.timestamp < cutoff {
                self.recent_events.pop_front();
            } else {
                break;
            }
        }
    }

    /// Appliquer le decay mensuel
    fn apply_decay(&self, trust: &mut UserTrust) {
        let decay_threshold = Duration::from_secs(30 * 86400); // 30 jours
        if trust.last_updated.elapsed() > decay_threshold {
            trust.pos *= self.config.decay_factor;
            trust.neg *= self.config.decay_factor;
            debug!("Applied decay to trust for device {}", self.device_id);
        }
    }

    /// Charger les événements récents au démarrage
    pub async fn load_recent_events(&mut self) -> Result<(), String> {
        let events = self
            .db
            .lock()
            .await
            .get_recent_trust_events(&self.device_id, self.config.rate_limit_max)
            .await?;
        self.recent_events = events.into_iter().collect();
        debug!(
            "Loaded {} recent events for device {}",
            self.recent_events.len(),
            self.device_id
        );
        Ok(())
    }

    /// Sauvegarder les événements récents
    pub async fn save_recent_events(&self) -> Result<(), String> {
        for event in &self.recent_events {
            self.db.lock().await.save_trust_event(event).await?;
        }
        Ok(())
    }

    /// Obtenir l'historique des rewards pour détection d'anomalies
    pub async fn get_reward_history(&self, limit: usize) -> Result<Vec<f32>, String> {
        self.db
            .lock()
            .await
            .get_recent_rewards(&self.device_id, limit)
            .await
    }

    /// Réinitialiser le trust (pour tests ou reset utilisateur)
    pub async fn reset_trust(&mut self) -> Result<(), String> {
        let mut trust = self.get_trust().await?;
        trust.pos = 0.0;
        trust.neg = 0.0;
        trust.trust = 0.5;
        trust.quarantine = false;
        trust.last_updated = Instant::now();

        self.db.lock().await.update_user_trust(&trust).await?;
        self.recent_events.clear();

        info!("Trust reset for device {}", self.device_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_trust_update() {
        // Mock database pour les tests
        let db = Arc::new(DatabaseManager::new(":memory:").await.unwrap());
        let mut trust_manager = TrustManager::new(db, "test_device".to_string());

        // Test update positif
        let trust1 = trust_manager.update_from_reward(0.8).await.unwrap();
        assert!(trust1 > 0.5);

        // Test update négatif
        let trust2 = trust_manager.update_from_reward(0.2).await.unwrap();
        assert!(trust2 < trust1);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let db = Arc::new(DatabaseManager::new(":memory:").await.unwrap());
        let mut trust_manager = TrustManager::new(db, "test_device".to_string());

        // Dépasser la limite de rate
        for i in 0..15 {
            let result = trust_manager.update_from_reward(0.5).await;
            if i >= 10 {
                assert!(result.is_err()); // Doit échouer après 10
            }
        }
    }

    #[tokio::test]
    async fn test_quarantine() {
        let db = Arc::new(DatabaseManager::new(":memory:").await.unwrap());
        let mut trust_manager = TrustManager::new(db, "test_device".to_string());

        // Envoyer beaucoup de feedback négatif
        for _ in 0..35 {
            let _ = trust_manager.update_from_reward(0.1).await;
        }

        let trust_level = trust_manager.get_trust_level().await.unwrap();
        assert!(trust_level.quarantine);
        assert!(trust_level.score < 0.1);
    }
}
