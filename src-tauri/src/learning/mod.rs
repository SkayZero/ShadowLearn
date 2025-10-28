use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::adaptive::AdaptivePromptEngine;
use crate::artefact::ArtefactSystem;
use crate::clustering::{ClusteringSystem, ProcessedContext};
use crate::context::aggregator::Context;
use crate::intent::llm_client::{LLMClient, LLMProvider};
use crate::intent::{Intent, IntentSystem};
use crate::persistence::database::DatabaseManager;
use crate::validator::{ArtefactType, ArtefactValidator, ValidationResult};

pub mod anomaly;
pub mod feedback;
pub mod reward;
pub mod trust;

use anomaly::{AnomalyDetector, AnomalyStats};
use feedback::FeedbackCollector;
use reward::{Outcome, RewardCalculator, RewardMetrics};
use trust::{TrustLevel, TrustManager};

/// Empreinte de contexte pour clustering
#[derive(Debug, Clone)]
pub struct ContextFingerprint {
    pub domain: String,
    pub hash: String,
}

impl ContextFingerprint {
    pub fn from_context(context: &Context) -> Self {
        // Simplifié: utiliser l'app comme domaine
        let domain = context.app.name.clone();
        let hash = format!(
            "{:x}",
            md5::compute(format!("{}_{}", domain, context.app.bundle_id))
        );

        Self { domain, hash }
    }
}

/// Système d'apprentissage complet avec trust, anomalies, rewards, clustering, intent detection, adaptive prompting, artefact generation, et feedback
pub struct LearningSystem {
    trust_manager: TrustManager,
    anomaly_detector: AnomalyDetector,
    reward_calculator: RewardCalculator,
    feedback_collector: FeedbackCollector, // J24
    validator: ArtefactValidator,
    clustering_system: ClusteringSystem,
    intent_system: IntentSystem,
    adaptive_engine: AdaptivePromptEngine, // J22
    artefact_system: ArtefactSystem, // J23
    db: Arc<Mutex<DatabaseManager>>,
    device_id: String,
}

impl LearningSystem {
    pub fn new(db: Arc<Mutex<DatabaseManager>>, device_id: String) -> Self {
        // Initialize LLM client (default to Ollama, can be configured later)
        let llm_client = Arc::new(std::sync::Mutex::new(LLMClient::new(
            LLMProvider::Ollama,
            None,
        )));

        // Initialize trust manager
        let trust_manager = TrustManager::new(db.clone(), device_id.clone());

        Self {
            trust_manager,
            anomaly_detector: AnomalyDetector::new(),
            reward_calculator: RewardCalculator::new(),
            feedback_collector: FeedbackCollector::new(TrustManager::new(db.clone(), device_id.clone())), // J24
            validator: ArtefactValidator::new(),
            clustering_system: ClusteringSystem::new(),
            intent_system: IntentSystem::new(llm_client),
            adaptive_engine: AdaptivePromptEngine::new(), // J22
            artefact_system: ArtefactSystem::new(), // J23
            db,
            device_id,
        }
    }

    /// Initialiser le système (charger les données)
    #[allow(dead_code)]
    pub async fn initialize(&mut self) -> Result<(), String> {
        info!("Initializing learning system for device {}", self.device_id);

        // Charger les événements récents
        self.trust_manager.load_recent_events().await?;

        info!("Learning system initialized successfully");
        Ok(())
    }

    /// Enregistrer un outcome et mettre à jour tous les systèmes
    pub async fn record_outcome(
        &mut self,
        suggestion_id: &str,
        context: &Context,
        artefact_type: &str,
        outcome: Outcome,
    ) -> Result<f32, String> {
        debug!(
            "Recording outcome for suggestion {}: {:?}",
            suggestion_id, outcome
        );

        // Calculer le reward brut
        let raw_reward = self.reward_calculator.compute(&outcome);

        // Vérifier les anomalies
        let history = self.get_reward_history().await?;
        let is_anomaly = self.anomaly_detector.is_anomaly(raw_reward, &history);

        if is_anomaly {
            warn!(
                "Anomaly detected for reward {:.3}, ignoring feedback",
                raw_reward
            );
            return Ok(0.0); // Ignorer les feedbacks anormaux
        }

        // Vérifier la quarantaine
        if self.trust_manager.is_quarantined().await? {
            info!(
                "Ignoring feedback from quarantined device {}",
                self.device_id
            );
            return Ok(0.0);
        }

        // Obtenir le trust et appliquer le poids
        let trust_level = self.trust_manager.get_trust_level().await?;
        let trust_weight = self.trust_manager.get_trust_weight(trust_level.score);
        let weighted_reward = self
            .reward_calculator
            .apply_trust_weight(raw_reward, trust_weight);

        // Mettre à jour le trust
        let new_trust = self
            .trust_manager
            .update_from_reward(weighted_reward)
            .await?;

        // Obtenir l'empreinte du contexte
        let fingerprint = ContextFingerprint::from_context(context);
        let cluster_id = self.get_cluster_id(&fingerprint).await?;

        // Stocker l'outcome dans la DB
        let outcome_id = Uuid::new_v4().to_string();
        self.store_outcome(
            &outcome_id,
            suggestion_id,
            &outcome,
            weighted_reward,
            &cluster_id,
            artefact_type,
        )
        .await?;

        info!(
            "Outcome recorded: reward={:.3}, trust={:.3}, cluster={}",
            weighted_reward, new_trust, cluster_id
        );

        Ok(weighted_reward)
    }

    /// Obtenir le niveau de trust actuel
    pub async fn get_trust_level(&self) -> Result<TrustLevel, String> {
        self.trust_manager.get_trust_level().await
    }

    /// Obtenir les statistiques d'anomalies
    pub async fn get_anomaly_stats(&self) -> Result<AnomalyStats, String> {
        let history = self.get_reward_history().await?;
        Ok(self.anomaly_detector.get_statistics(&history))
    }

    /// Obtenir les métriques de reward
    pub async fn get_reward_metrics(&self) -> Result<RewardMetrics, String> {
        let outcomes = self
            .db
            .lock()
            .await
            .get_recent_outcomes(&self.device_id, 100)
            .await?;
        Ok(self.reward_calculator.get_reward_metrics(&outcomes))
    }

    /// Réinitialiser le trust (pour tests ou reset utilisateur)
    pub async fn reset_trust(&mut self) -> Result<(), String> {
        self.trust_manager.reset_trust().await
    }

    /// Obtenir l'historique des rewards
    async fn get_reward_history(&self) -> Result<Vec<f32>, String> {
        self.trust_manager.get_reward_history(50).await
    }

    /// Obtenir l'ID de cluster pour un contexte
    async fn get_cluster_id(&self, fingerprint: &ContextFingerprint) -> Result<String, String> {
        // Simplifié: utiliser le hash de l'empreinte comme cluster
        // En production, on ferait du clustering LSH
        Ok(fingerprint.hash.clone())
    }

    /// Stocker un outcome dans la DB
    async fn store_outcome(
        &self,
        outcome_id: &str,
        suggestion_id: &str,
        outcome: &Outcome,
        reward: f32,
        cluster_id: &str,
        artefact_type: &str,
    ) -> Result<(), String> {
        let (used, helpful, reverted, time_to_flow_ms) = match outcome {
            Outcome::Used {
                helpful,
                reverted,
                time_to_flow,
            } => (
                true,
                *helpful,
                *reverted,
                time_to_flow.map(|d| d.as_millis() as i64).unwrap_or(0),
            ),
            Outcome::Ignored => (false, false, false, 0),
            Outcome::Dismissed => (false, false, false, 0),
        };

        self.db
            .lock()
            .await
            .store_outcome(
                outcome_id,
                suggestion_id,
                used,
                helpful,
                reverted,
                time_to_flow_ms,
                reward,
                cluster_id,
                artefact_type,
            )
            .await
    }

    /// Obtenir les recommandations basées sur le trust
    pub async fn get_trust_recommendations(&self) -> Result<TrustRecommendations, String> {
        let trust_level = self.get_trust_level().await?;
        let anomaly_stats = self.get_anomaly_stats().await?;
        let reward_metrics = self.get_reward_metrics().await?;

        Ok(TrustRecommendations {
            trust_level: trust_level.clone(),
            anomaly_stats: anomaly_stats.clone(),
            reward_metrics: reward_metrics.clone(),
            recommendations: self.generate_recommendations(
                &trust_level,
                &anomaly_stats,
                &reward_metrics,
            ),
        })
    }

    /// Générer des recommandations basées sur les métriques
    fn generate_recommendations(
        &self,
        trust_level: &TrustLevel,
        anomaly_stats: &AnomalyStats,
        reward_metrics: &RewardMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if trust_level.quarantine {
            recommendations.push("Device is quarantined due to low trust score".to_string());
        } else if trust_level.score < 0.3 {
            recommendations
                .push("Low trust score - consider improving suggestion quality".to_string());
        }

        if anomaly_stats.variance < 0.01 {
            recommendations
                .push("Low variance in feedback - possible bot activity detected".to_string());
        }

        if reward_metrics.helpful_rate < 0.5 {
            recommendations
                .push("Low helpfulness rate - suggestions may not be relevant".to_string());
        }

        if reward_metrics.reversion_rate > 0.3 {
            recommendations.push("High reversion rate - suggestions may be disruptive".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("System performing well - no immediate concerns".to_string());
        }

        recommendations
    }
}

/// Recommandations de trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustRecommendations {
    pub trust_level: TrustLevel,
    pub anomaly_stats: AnomalyStats,
    pub reward_metrics: RewardMetrics,
    pub recommendations: Vec<String>,
}

/// Processed context with clustering and intent information
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContextWithIntent {
    pub processed_context: ProcessedContext,
    pub intent: Intent,
}

/// Comprehensive learning metrics for monitoring
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub clusters_count: usize,
    pub intent_cache_size: usize,
    pub cache_hit_rate: f64,
    pub average_intent_ms: f64,
    pub total_contexts_processed: u64,
    pub high_confidence_rate: f64,
}

impl LearningSystem {
    /// Validate an artefact before learning from it
    pub async fn validate_before_learning(
        &mut self,
        artefact_path: &std::path::Path,
        artefact_type: ArtefactType,
    ) -> Result<bool, String> {
        info!(
            "[VALIDATOR] Validating artefact before learning: {:?}",
            artefact_path
        );

        let result = self.validator.validate(artefact_path, artefact_type).await;

        match result {
            ValidationResult::Valid => {
                info!("[VALIDATOR] Artefact valid, proceeding with learning");
                Ok(true)
            }
            ValidationResult::Invalid(reason) => {
                warn!("[VALIDATOR] Artefact invalid: {}", reason);
                Ok(false)
            }
            ValidationResult::Error(e) => {
                error!("[VALIDATOR] Validation error: {}", e);
                Ok(false) // Treat errors as invalid
            }
            ValidationResult::Skipped(reason) => {
                info!("[VALIDATOR] Validation skipped: {}", reason);
                Ok(true) // No validator available, allow learning
            }
        }
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> &crate::validator::stats::ValidationStats {
        self.validator.get_stats()
    }

    /// Get validator status (which tools are available)
    pub fn get_validator_status(&self) -> crate::validator::ValidatorStatus {
        self.validator.get_validator_status()
    }

    /// Clear validation cache
    pub fn clear_validation_cache(&mut self) {
        self.validator.clear_cache();
    }

    /// Process context with clustering and intent detection
    #[allow(dead_code)]
    pub async fn process_context(
        &mut self,
        ctx: &Context,
    ) -> Result<ProcessedContextWithIntent, String> {
        let start_time = std::time::Instant::now();
        info!(
            "[LEARNING] Processing context: {} - {}",
            ctx.app.name, ctx.app.window_title
        );

        // Step 1: Cluster the context
        let cluster_start = std::time::Instant::now();
        let processed_context = self.clustering_system.process_context(ctx).await?;
        let cluster_ms = cluster_start.elapsed().as_millis();

        // Step 2: Detect intent
        let intent_start = std::time::Instant::now();
        let intent = self.intent_system.detect_intent(ctx).await?;
        let intent_ms = intent_start.elapsed().as_millis();

        // Step 3: Check if intent is confident enough to proceed
        if !self.intent_system.should_proceed(&intent) {
            warn!(
                "[LEARNING] Intent confidence too low: {:.2} (threshold: 0.5)",
                intent.confidence
            );
            return Err(format!("Low confidence intent: {:.2}", intent.confidence));
        }

        let total_ms = start_time.elapsed().as_millis();
        info!("[LEARNING] Context processed successfully: cluster={}, intent={:?} (confidence: {:.2}) - cluster_ms={}, intent_ms={}, total_ms={}", 
              processed_context.cluster_id, intent.intent_type, intent.confidence, cluster_ms, intent_ms, total_ms);

        Ok(ProcessedContextWithIntent {
            processed_context,
            intent,
        })
    }

    /// Get clustering statistics
    #[allow(dead_code)]
    pub fn get_clustering_stats(&self) -> &crate::clustering::ClusteringStats {
        self.clustering_system.get_stats()
    }

    /// Get intent detection statistics
    #[allow(dead_code)]
    pub fn get_intent_stats(&self) -> &crate::intent::IntentStats {
        self.intent_system.get_stats()
    }

    /// Get comprehensive learning metrics
    #[allow(dead_code)]
    pub fn get_learning_metrics(&self) -> LearningMetrics {
        let clustering_stats = self.get_clustering_stats();
        let intent_stats = self.get_intent_stats();

        LearningMetrics {
            clusters_count: self.clustering_system.get_cluster_count(),
            intent_cache_size: self.intent_system.get_cache_size(),
            cache_hit_rate: intent_stats.get_cache_hit_rate(),
            average_intent_ms: intent_stats.average_detection_time_ms,
            total_contexts_processed: clustering_stats.total_contexts_processed,
            high_confidence_rate: intent_stats.get_high_confidence_rate(),
        }
    }

    /// Clear clustering cache
    #[allow(dead_code)]
    pub fn clear_clustering_cache(&mut self) {
        // Note: ClusteringSystem doesn't have a cache to clear, but we could add one
        info!("[LEARNING] Clustering cache cleared (no-op)");
    }

    /// Clear intent cache
    #[allow(dead_code)]
    pub fn clear_intent_cache(&mut self) {
        self.intent_system.clear_cache();
    }

    /// Switch LLM provider
    pub fn switch_llm_provider(&mut self, provider: LLMProvider, _api_key: Option<String>) {
        // This would require modifying IntentSystem to expose the LLM client
        // For now, we'll just log the request
        info!("[LEARNING] LLM provider switch requested: {:?}", provider);
    }

    /// Record feedback for an opportunity or message
    /// Clueless Phase 1: Simplified feedback recording
    pub async fn record_feedback(&mut self, item_id: String, helpful: bool) -> Result<(), String> {
        info!("[LEARNING] Recording feedback: item={}, helpful={}", item_id, helpful);
        
        // Store feedback in the feedback collector
        let outcome = if helpful {
            feedback::FeedbackOutcome::Positive
        } else {
            feedback::FeedbackOutcome::Negative
        };
        
        self.feedback_collector
            .record_feedback(&item_id, outcome)
            .await
            .map_err(|e| e)?;
        
        // Log trust update (no actual update method in TrustManager)
        info!("[LEARNING] Trust update from feedback (logged only)");
        
        Ok(())
    }

    /// Adjust confidence weights based on feedback
    /// Clueless Phase 1: Update learning from feedback
    pub async fn adjust_confidence_weights(&mut self, helpful: bool) {
        info!("[LEARNING] Adjusting confidence weights: helpful={}", helpful);
        // For now, just log - can be enhanced later
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_learning_system() {
        let db = Arc::new(DatabaseManager::new(":memory:").await.unwrap());
        let mut learning_system = LearningSystem::new(db, "test_device".to_string());
        learning_system.initialize().await.unwrap();

        // Test outcome positif
        let context = Context {
            id: "test".to_string(),
            app: crate::context::app_detector::ActiveApp {
                name: "TestApp".to_string(),
                bundle_id: "com.test.app".to_string(),
                window_title: "Test Window".to_string(),
            },
            clipboard: None,
            idle_seconds: 5.0,
            timestamp: 1234567890,
            capture_duration_ms: 100,
        };

        let outcome = Outcome::Used {
            helpful: true,
            reverted: false,
            time_to_flow: Some(Duration::from_secs(10)),
        };

        let reward = learning_system
            .record_outcome("suggestion_1", &context, "snippet", outcome)
            .await
            .unwrap();

        assert!(reward > 0.0);
    }
}
