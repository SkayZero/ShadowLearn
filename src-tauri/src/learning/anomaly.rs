use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, warn};

/// Détecteur d'anomalies basé sur Hampel (MAD - Median Absolute Deviation)
pub struct AnomalyDetector {
    window_size: usize,
    mad_threshold: f32,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            window_size: 50,
            mad_threshold: 3.0,
        }
    }

    pub fn with_config(window_size: usize, mad_threshold: f32) -> Self {
        Self {
            window_size,
            mad_threshold,
        }
    }

    /// Détecter si une valeur est une anomalie par rapport à l'historique
    pub fn is_anomaly(&self, value: f32, history: &[f32]) -> bool {
        if history.len() < 10 {
            debug!(
                "Not enough data for anomaly detection: {} samples",
                history.len()
            );
            return false; // Pas assez de données
        }

        let recent: Vec<f32> = history
            .iter()
            .rev()
            .take(self.window_size)
            .copied()
            .collect();

        let median = self.median(&recent);
        let mad = self.mad(&recent, median);

        if mad < 1e-6 {
            debug!("No variance in data, skipping anomaly detection");
            return false; // Pas de variance
        }

        let modified_z_score = 0.6745 * (value - median).abs() / mad;

        if modified_z_score > self.mad_threshold {
            warn!(
                "Anomaly detected: value={:.3}, median={:.3}, mad={:.3}, z_score={:.3}",
                value, median, mad, modified_z_score
            );
            return true;
        }

        false
    }

    /// Détecter les patterns suspects (bots, spam)
    pub fn detect_pattern_anomaly(&self, rewards: &[f32]) -> bool {
        if rewards.len() < 20 {
            return false;
        }

        // Détecter les patterns trop réguliers (bots)
        let variance = self.calculate_variance(rewards);
        if variance < 0.01 {
            warn!(
                "Pattern anomaly detected: very low variance ({:.6})",
                variance
            );
            return true;
        }

        // Détecter les séquences répétitives
        if self.detect_repetitive_pattern(rewards) {
            warn!("Pattern anomaly detected: repetitive sequence");
            return true;
        }

        false
    }

    /// Détecter le drift temporel
    pub fn detect_temporal_drift(&self, rewards: &[f32], timestamps: &[Instant]) -> bool {
        if rewards.len() < 30 || timestamps.len() != rewards.len() {
            return false;
        }

        let recent_avg = rewards.iter().rev().take(10).sum::<f32>() / 10.0;
        let older_avg = rewards.iter().take(20).sum::<f32>() / 20.0;

        let drift = (recent_avg - older_avg).abs();
        if drift > 0.3 {
            warn!(
                "Temporal drift detected: recent_avg={:.3}, older_avg={:.3}, drift={:.3}",
                recent_avg, older_avg, drift
            );
            return true;
        }

        false
    }

    /// Calculer la médiane
    fn median(&self, values: &[f32]) -> f32 {
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    /// Calculer la MAD (Median Absolute Deviation)
    fn mad(&self, values: &[f32], median: f32) -> f32 {
        let deviations: Vec<f32> = values.iter().map(|&v| (v - median).abs()).collect();

        self.median(&deviations)
    }

    /// Calculer la variance
    fn calculate_variance(&self, values: &[f32]) -> f32 {
        if values.is_empty() {
            return 0.0;
        }

        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance =
            values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32;

        variance
    }

    /// Détecter les patterns répétitifs
    fn detect_repetitive_pattern(&self, rewards: &[f32]) -> bool {
        if rewards.len() < 10 {
            return false;
        }

        // Vérifier si les dernières valeurs sont identiques
        let last_5: Vec<f32> = rewards.iter().rev().take(5).copied().collect();
        let first_value = last_5[0];

        if last_5.iter().all(|&v| (v - first_value).abs() < 1e-6) {
            return true;
        }

        // Vérifier les patterns de 2 valeurs alternées
        if rewards.len() >= 6 {
            let last_6: Vec<f32> = rewards.iter().rev().take(6).copied().collect();
            let val1 = last_6[0];
            let val2 = last_6[1];

            if last_6.iter().enumerate().all(|(i, &v)| {
                if i % 2 == 0 {
                    (v - val1).abs() < 1e-6
                } else {
                    (v - val2).abs() < 1e-6
                }
            }) {
                return true;
            }
        }

        false
    }

    /// Obtenir les statistiques de l'historique
    pub fn get_statistics(&self, history: &[f32]) -> AnomalyStats {
        if history.is_empty() {
            return AnomalyStats::default();
        }

        let median = self.median(history);
        let mad = self.mad(history, median);
        let variance = self.calculate_variance(history);
        let mean = history.iter().sum::<f32>() / history.len() as f32;

        AnomalyStats {
            mean,
            median,
            mad,
            variance,
            sample_size: history.len(),
        }
    }
}

/// Statistiques d'anomalie
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnomalyStats {
    pub mean: f32,
    pub median: f32,
    pub mad: f32,
    pub variance: f32,
    pub sample_size: usize,
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detection() {
        let detector = AnomalyDetector::new();
        let history = vec![1.0, 1.1, 0.9, 1.2, 1.0, 0.95, 1.05, 1.1, 0.98, 1.02];

        assert!(!detector.is_anomaly(1.15, &history)); // Normal
        assert!(detector.is_anomaly(5.0, &history)); // Anomaly
        assert!(detector.is_anomaly(-2.0, &history)); // Anomaly
    }

    #[test]
    fn test_pattern_anomaly() {
        let detector = AnomalyDetector::new();

        // Pattern répétitif
        let repetitive = vec![0.5; 25];
        assert!(detector.detect_pattern_anomaly(&repetitive));

        // Pattern normal
        let normal = vec![0.5, 0.6, 0.4, 0.7, 0.3, 0.8, 0.2, 0.9, 0.1, 1.0];
        assert!(!detector.detect_pattern_anomaly(&normal));
    }

    #[test]
    fn test_temporal_drift() {
        let detector = AnomalyDetector::new();

        // Créer des timestamps fictifs
        let now = Instant::now();
        let timestamps: Vec<Instant> = (0..50)
            .map(|i| now - std::time::Duration::from_secs(i))
            .collect();

        // Drift significatif
        let mut rewards = vec![0.2; 20]; // Anciennes valeurs faibles
        rewards.extend(vec![0.8; 30]); // Nouvelles valeurs élevées

        assert!(detector.detect_temporal_drift(&rewards, &timestamps));

        // Pas de drift
        let stable_rewards = vec![0.5; 50];
        assert!(!detector.detect_temporal_drift(&stable_rewards, &timestamps));
    }

    #[test]
    fn test_statistics() {
        let detector = AnomalyDetector::new();
        let history = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let stats = detector.get_statistics(&history);
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.sample_size, 5);
    }
}
