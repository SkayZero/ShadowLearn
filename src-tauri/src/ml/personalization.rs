use chrono::{DateTime, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Patterns d'usage détectés pour un utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePatterns {
    /// Apps les plus utilisées avec fréquence
    pub favorite_apps: HashMap<String, AppUsageStats>,
    /// Heures de productivité détectées
    pub productive_hours: Vec<u8>,
    /// Jours de la semaine les plus actifs
    pub active_weekdays: Vec<Weekday>,
    /// Durée moyenne d'inactivité avant trigger
    pub avg_idle_before_trigger: f64,
    /// Temps de réponse moyen aux triggers
    pub avg_response_time_ms: f64,
    /// Apps souvent ignorées (pour auto-mute)
    pub frequently_ignored_apps: HashMap<String, u32>,
    /// Contextes de clipboard les plus fréquents
    pub clipboard_patterns: HashMap<String, u32>,
}

/// Statistiques d'usage pour une app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageStats {
    /// Nombre total de triggers pour cette app
    pub total_triggers: u32,
    /// Nombre de triggers acceptés (avec interaction)
    pub accepted_triggers: u32,
    /// Nombre de triggers ignorés
    pub ignored_triggers: u32,
    /// Taux d'acceptation (0.0 à 1.0)
    pub acceptance_rate: f64,
    /// Heures d'usage les plus fréquentes
    pub peak_hours: Vec<u8>,
    /// Dernière utilisation
    pub last_used: Option<DateTime<Utc>>,
}

/// Suggestions intelligentes basées sur les patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSuggestions {
    /// Apps recommandées pour l'allowlist
    pub recommended_apps: Vec<String>,
    /// Heure optimale pour les triggers
    pub optimal_trigger_hour: Option<u8>,
    /// Seuils recommandés
    pub recommended_thresholds: RecommendedThresholds,
    /// Apps à muter automatiquement
    pub apps_to_mute: Vec<String>,
}

/// Seuils recommandés par le ML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedThresholds {
    /// Seuil d'inactivité recommandé (en secondes)
    pub idle_threshold: u32,
    /// Cooldown de base recommandé (en secondes)
    pub base_cooldown: u32,
    /// Cooldown après dismiss recommandé (en secondes)
    pub dismiss_cooldown: u32,
    /// Seuil de debounce recommandé (en secondes)
    pub debounce_threshold: u32,
}

/// Gestionnaire de personnalisation ML
pub struct PersonalizationManager {
    /// Patterns d'usage actuels
    patterns: UsagePatterns,
    /// Historique des événements pour apprentissage
    event_history: Vec<UserEvent>,
    /// Configuration ML
    config: MLConfig,
}

/// Configuration du système ML
#[derive(Debug, Clone)]
pub struct MLConfig {
    /// Nombre minimum d'événements pour apprentissage
    pub min_events_for_learning: usize,
    /// Période de rétention des données (en jours)
    pub data_retention_days: u32,
}

/// Événement utilisateur pour apprentissage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub app_name: String,
    pub context: Option<String>,
    pub user_response: Option<UserResponse>,
}

/// Types d'événements utilisateur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    TriggerFired,
    TriggerAccepted,
    TriggerIgnored,
    TriggerDismissed,
    AppMuted,
    AppUnmuted,
    ClipboardChanged,
    IdleDetected,
}

/// Réponse de l'utilisateur à un trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserResponse {
    Accepted,  // Utilisateur a interagi avec le trigger
    Ignored,   // Utilisateur a ignoré le trigger
    Dismissed, // Utilisateur a fermé le trigger
    Snoozed,   // Utilisateur a mis en pause
}

impl PersonalizationManager {
    /// Crée un nouveau gestionnaire de personnalisation
    pub fn new() -> Self {
        Self {
            patterns: UsagePatterns::default(),
            event_history: Vec::new(),
            config: MLConfig::default(),
        }
    }

    /// Enregistre un événement utilisateur pour apprentissage
    pub fn record_event(&mut self, event: UserEvent) {
        debug!("📊 Recording user event: {:?}", event.event_type);

        // Ajouter à l'historique
        self.event_history.push(event.clone());

        // Nettoyer l'historique ancien
        self.cleanup_old_events();

        // Mettre à jour les patterns si assez de données
        if self.event_history.len() >= self.config.min_events_for_learning {
            self.update_patterns();
        }
    }

    /// Met à jour les patterns d'usage basés sur l'historique
    fn update_patterns(&mut self) {
        debug!(
            "🧠 Updating usage patterns from {} events",
            self.event_history.len()
        );

        // Analyser les apps favorites
        self.analyze_favorite_apps();

        // Analyser les heures productives
        self.analyze_productive_hours();

        // Analyser les patterns d'inactivité
        self.analyze_idle_patterns();

        // Analyser les réponses aux triggers
        self.analyze_trigger_responses();

        info!(
            "✅ Usage patterns updated: {} favorite apps, {} productive hours",
            self.patterns.favorite_apps.len(),
            self.patterns.productive_hours.len()
        );
    }

    /// Analyse les apps favorites basées sur l'usage
    fn analyze_favorite_apps(&mut self) {
        let mut app_stats: HashMap<String, AppUsageStats> = HashMap::new();

        for event in &self.event_history {
            let stats = app_stats
                .entry(event.app_name.clone())
                .or_insert(AppUsageStats {
                    total_triggers: 0,
                    accepted_triggers: 0,
                    ignored_triggers: 0,
                    acceptance_rate: 0.0,
                    peak_hours: Vec::new(),
                    last_used: None,
                });

            match event.event_type {
                EventType::TriggerFired => {
                    stats.total_triggers += 1;
                    stats.last_used = Some(event.timestamp);
                    stats.peak_hours.push(event.timestamp.hour() as u8);
                }
                EventType::TriggerAccepted => {
                    stats.accepted_triggers += 1;
                }
                EventType::TriggerIgnored => {
                    stats.ignored_triggers += 1;
                }
                _ => {}
            }
        }

        // Calculer les taux d'acceptation
        for stats in app_stats.values_mut() {
            if stats.total_triggers > 0 {
                stats.acceptance_rate =
                    stats.accepted_triggers as f64 / stats.total_triggers as f64;
            }
        }

        self.patterns.favorite_apps = app_stats;
    }

    /// Analyse les heures les plus productives
    fn analyze_productive_hours(&mut self) {
        let mut hour_counts: HashMap<u8, u32> = HashMap::new();

        for event in &self.event_history {
            if matches!(event.event_type, EventType::TriggerAccepted) {
                let hour = event.timestamp.hour() as u8;
                *hour_counts.entry(hour).or_insert(0) += 1;
            }
        }

        // Prendre les 3 heures les plus productives
        let mut sorted_hours: Vec<(u8, u32)> = hour_counts.into_iter().collect();
        sorted_hours.sort_by(|a, b| b.1.cmp(&a.1));

        self.patterns.productive_hours = sorted_hours
            .into_iter()
            .take(3)
            .map(|(hour, _)| hour)
            .collect();
    }

    /// Analyse les patterns d'inactivité
    fn analyze_idle_patterns(&mut self) {
        let mut idle_times: Vec<f64> = Vec::new();

        for event in &self.event_history {
            if matches!(event.event_type, EventType::IdleDetected) {
                // Extraire le temps d'inactivité du contexte si disponible
                if let Some(context) = &event.context {
                    if let Ok(idle_seconds) = context.parse::<f64>() {
                        idle_times.push(idle_seconds);
                    }
                }
            }
        }

        if !idle_times.is_empty() {
            self.patterns.avg_idle_before_trigger =
                idle_times.iter().sum::<f64>() / idle_times.len() as f64;
        }
    }

    /// Analyse les réponses aux triggers
    fn analyze_trigger_responses(&mut self) {
        let mut response_times: Vec<f64> = Vec::new();
        let mut ignored_apps: HashMap<String, u32> = HashMap::new();

        for event in &self.event_history {
            match event.event_type {
                EventType::TriggerIgnored => {
                    *ignored_apps.entry(event.app_name.clone()).or_insert(0) += 1;
                }
                EventType::TriggerAccepted => {
                    // Calculer le temps de réponse si possible
                    if let Some(context) = &event.context {
                        if let Ok(response_time) = context.parse::<f64>() {
                            response_times.push(response_time);
                        }
                    }
                }
                _ => {}
            }
        }

        if !response_times.is_empty() {
            self.patterns.avg_response_time_ms =
                response_times.iter().sum::<f64>() / response_times.len() as f64;
        }

        self.patterns.frequently_ignored_apps = ignored_apps;
    }

    /// Génère des suggestions intelligentes
    pub fn generate_suggestions(&self) -> SmartSuggestions {
        debug!("🎯 Generating smart suggestions from patterns");

        let mut recommended_apps = Vec::new();
        let mut apps_to_mute = Vec::new();

        // Recommander les apps avec taux d'acceptation élevé
        for (app_name, stats) in &self.patterns.favorite_apps {
            if stats.acceptance_rate > 0.7 && stats.total_triggers >= 3 {
                recommended_apps.push(app_name.clone());
            }

            // Muter les apps souvent ignorées
            if stats.acceptance_rate < 0.2 && stats.total_triggers >= 5 {
                apps_to_mute.push(app_name.clone());
            }
        }

        // Heure optimale basée sur les heures productives
        let optimal_hour = self.patterns.productive_hours.first().copied();

        // Seuils recommandés basés sur les patterns
        let recommended_thresholds = RecommendedThresholds {
            idle_threshold: (self.patterns.avg_idle_before_trigger * 0.8) as u32,
            base_cooldown: 45,     // Valeur par défaut
            dismiss_cooldown: 90,  // Valeur par défaut
            debounce_threshold: 2, // Valeur par défaut
        };

        SmartSuggestions {
            recommended_apps,
            optimal_trigger_hour: optimal_hour,
            recommended_thresholds,
            apps_to_mute,
        }
    }

    /// Nettoie les événements anciens
    fn cleanup_old_events(&mut self) {
        let cutoff_date =
            Utc::now() - chrono::Duration::days(self.config.data_retention_days as i64);
        self.event_history
            .retain(|event| event.timestamp > cutoff_date);
    }

    /// Obtient les patterns actuels
    pub fn get_patterns(&self) -> &UsagePatterns {
        &self.patterns
    }

    /// Sauvegarde les patterns dans un fichier JSON
    pub fn save_patterns(&self, file_path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.patterns)
            .map_err(|e| format!("Failed to serialize patterns: {}", e))?;

        std::fs::write(file_path, json)
            .map_err(|e| format!("Failed to write patterns file: {}", e))?;

        info!("💾 Patterns saved to {}", file_path);
        Ok(())
    }

    /// Charge les patterns depuis un fichier JSON
    pub fn load_patterns(&mut self, file_path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read patterns file: {}", e))?;

        self.patterns = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize patterns: {}", e))?;

        info!("📂 Patterns loaded from {}", file_path);
        Ok(())
    }
}

impl Default for UsagePatterns {
    fn default() -> Self {
        Self {
            favorite_apps: HashMap::new(),
            productive_hours: Vec::new(),
            active_weekdays: Vec::new(),
            avg_idle_before_trigger: 12.0,
            avg_response_time_ms: 0.0,
            frequently_ignored_apps: HashMap::new(),
            clipboard_patterns: HashMap::new(),
        }
    }
}

impl Default for MLConfig {
    fn default() -> Self {
        Self {
            min_events_for_learning: 10,
            data_retention_days: 30,
        }
    }
}
