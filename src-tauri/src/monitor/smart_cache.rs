use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Smart cache pour optimiser les captures d'écran
/// Adapte automatiquement l'intervalle de capture selon l'activité
pub struct SmartCache {
    /// Historique des derniers hashes (avec timestamp)
    last_hashes: VecDeque<(u64, Instant)>,

    /// Capacité max de l'historique
    max_history: usize,

    /// Seuil de répétition avant augmentation d'intervalle
    repeat_threshold: usize,

    /// Intervalle minimum (secondes)
    min_interval_secs: u64,

    /// Intervalle maximum (secondes)
    max_interval_secs: u64,

    /// Intervalle actuel (secondes)
    current_interval_secs: u64,

    /// Compteur de changements détectés
    change_count: usize,

    /// Compteur d'écrans identiques
    identical_count: usize,
}

impl SmartCache {
    /// Créer un nouveau smart cache
    pub fn new(
        initial_interval_secs: u64,
        min_interval_secs: u64,
        max_interval_secs: u64,
    ) -> Self {
        Self {
            last_hashes: VecDeque::with_capacity(10),
            max_history: 10,
            repeat_threshold: 3,
            min_interval_secs,
            max_interval_secs,
            current_interval_secs: initial_interval_secs,
            change_count: 0,
            identical_count: 0,
        }
    }

    /// Enregistrer un nouveau hash et déterminer si on doit analyser
    pub fn should_analyze(&mut self, hash: u64) -> bool {
        let now = Instant::now();

        // Vérifier si le hash est répété
        let is_repeated = self.last_hashes.iter().any(|(h, _)| *h == hash);

        if is_repeated {
            self.identical_count += 1;
            self.change_count = 0; // Reset change counter

            // Si beaucoup de répétitions → augmenter l'intervalle
            if self.identical_count >= self.repeat_threshold {
                self.increase_interval();
                debug!("📊 Increased interval to {}s (identical screens)", self.current_interval_secs);
            }

            // Pas besoin d'analyser si identique
            false
        } else {
            self.change_count += 1;
            self.identical_count = 0; // Reset identical counter

            // Si beaucoup de changements → diminuer l'intervalle
            if self.change_count >= self.repeat_threshold {
                self.decrease_interval();
                debug!("📊 Decreased interval to {}s (active user)", self.current_interval_secs);
            }

            // Ajouter à l'historique
            self.last_hashes.push_back((hash, now));

            // Limiter la taille de l'historique
            if self.last_hashes.len() > self.max_history {
                self.last_hashes.pop_front();
            }

            // Analyser car changement détecté
            true
        }
    }

    /// Obtenir l'intervalle adaptatif actuel
    pub fn adaptive_interval(&self) -> Duration {
        Duration::from_secs(self.current_interval_secs)
    }

    /// Augmenter l'intervalle (utilisateur inactif)
    fn increase_interval(&mut self) {
        let new_interval = (self.current_interval_secs * 2).min(self.max_interval_secs);

        if new_interval != self.current_interval_secs {
            info!("⏱️ Adaptive interval: {}s → {}s (slower)",
                  self.current_interval_secs, new_interval);
            self.current_interval_secs = new_interval;
        }
    }

    /// Diminuer l'intervalle (utilisateur actif)
    fn decrease_interval(&mut self) {
        let new_interval = (self.current_interval_secs / 2).max(self.min_interval_secs);

        if new_interval != self.current_interval_secs {
            info!("⏱️ Adaptive interval: {}s → {}s (faster)",
                  self.current_interval_secs, new_interval);
            self.current_interval_secs = new_interval;
        }
    }

    /// Reset le cache (utile après changement d'app ou contexte)
    pub fn reset(&mut self) {
        self.last_hashes.clear();
        self.change_count = 0;
        self.identical_count = 0;
        info!("🔄 Smart cache reset");
    }

    /// Obtenir les statistiques du cache
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            history_size: self.last_hashes.len(),
            current_interval_secs: self.current_interval_secs,
            change_count: self.change_count,
            identical_count: self.identical_count,
        }
    }

    /// Nettoyer les vieux hashes (plus de 5 minutes)
    pub fn cleanup_old(&mut self) {
        let now = Instant::now();
        let threshold = Duration::from_secs(300); // 5 minutes

        self.last_hashes.retain(|(_, timestamp)| {
            now.duration_since(*timestamp) < threshold
        });
    }
}

impl Default for SmartCache {
    fn default() -> Self {
        Self::new(5, 2, 30) // Default: 5s, min 2s, max 30s
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheStats {
    pub history_size: usize,
    pub current_interval_secs: u64,
    pub change_count: usize,
    pub identical_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = SmartCache::new(5, 2, 30);
        assert_eq!(cache.current_interval_secs, 5);
        assert_eq!(cache.min_interval_secs, 2);
        assert_eq!(cache.max_interval_secs, 30);
    }

    #[test]
    fn test_should_analyze_new_hash() {
        let mut cache = SmartCache::default();

        // Premier hash → doit analyser
        assert!(cache.should_analyze(123));

        // Hash différent → doit analyser
        assert!(cache.should_analyze(456));

        // Hash identique → ne doit PAS analyser
        assert!(!cache.should_analyze(456));
    }

    #[test]
    fn test_adaptive_interval_increase() {
        let mut cache = SmartCache::new(5, 2, 30);

        // Répéter le même hash plusieurs fois
        cache.should_analyze(100);
        assert!(!cache.should_analyze(100)); // Identical
        assert!(!cache.should_analyze(100)); // Identical
        assert!(!cache.should_analyze(100)); // Identical → interval increased

        // L'intervalle devrait avoir augmenté
        assert_eq!(cache.current_interval_secs, 10); // 5 * 2
    }

    #[test]
    fn test_adaptive_interval_decrease() {
        let mut cache = SmartCache::new(10, 2, 30);

        // Changements fréquents
        assert!(cache.should_analyze(1));
        assert!(cache.should_analyze(2));
        assert!(cache.should_analyze(3));
        assert!(cache.should_analyze(4)); // → interval decreased

        // L'intervalle devrait avoir diminué
        assert_eq!(cache.current_interval_secs, 5); // 10 / 2
    }

    #[test]
    fn test_reset() {
        let mut cache = SmartCache::default();

        cache.should_analyze(123);
        cache.should_analyze(456);

        assert_eq!(cache.last_hashes.len(), 2);

        cache.reset();

        assert_eq!(cache.last_hashes.len(), 0);
        assert_eq!(cache.change_count, 0);
        assert_eq!(cache.identical_count, 0);
    }

    #[test]
    fn test_stats() {
        let mut cache = SmartCache::default();

        cache.should_analyze(123);
        cache.should_analyze(456);

        let stats = cache.stats();
        assert_eq!(stats.history_size, 2);
        assert_eq!(stats.current_interval_secs, 5);
    }
}
