use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// Types d'outcomes possibles
#[derive(Debug, Clone)]
pub enum Outcome {
    Used {
        helpful: bool,
        reverted: bool,
        time_to_flow: Option<Duration>,
    },
    Ignored,
    #[allow(dead_code)]
    Dismissed,
}

/// Calculateur de rewards avec pondération par trust
pub struct RewardCalculator {
    // Coefficients de pondération
    helpful_weight: f32,
    used_weight: f32,
    reverted_penalty: f32,
    time_bonus_factor: f32,
}

impl RewardCalculator {
    pub fn new() -> Self {
        Self {
            helpful_weight: 0.4,
            used_weight: 0.3,
            reverted_penalty: 0.5,
            time_bonus_factor: 0.1,
        }
    }

    #[allow(dead_code)]
    pub fn with_weights(
        helpful_weight: f32,
        used_weight: f32,
        reverted_penalty: f32,
        time_bonus_factor: f32,
    ) -> Self {
        Self {
            helpful_weight,
            used_weight,
            reverted_penalty,
            time_bonus_factor,
        }
    }

    /// Calculer le reward brut à partir d'un outcome
    pub fn compute(&self, outcome: &Outcome) -> f32 {
        let reward = match outcome {
            Outcome::Used {
                helpful,
                reverted,
                time_to_flow,
            } => {
                let mut base_reward = 0.0;

                if *helpful {
                    base_reward += self.helpful_weight;
                }

                base_reward += self.used_weight;

                if *reverted {
                    base_reward -= self.reverted_penalty;
                }

                // Bonus pour temps rapide jusqu'au flow
                if let Some(time) = time_to_flow {
                    let time_seconds = time.as_secs() as f32;
                    if time_seconds < 30.0 {
                        base_reward += self.time_bonus_factor * (30.0 - time_seconds) / 30.0;
                    }
                }

                base_reward
            }
            Outcome::Ignored => 0.0,
            Outcome::Dismissed => -0.1, // Léger penalty pour dismissal
        };

        // Clamp dans [0, 1]
        reward.clamp(0.0, 1.0)
    }

    /// Appliquer le poids de trust au reward
    pub fn apply_trust_weight(&self, raw_reward: f32, trust_weight: f32) -> f32 {
        let weighted_reward = raw_reward * trust_weight;
        weighted_reward.clamp(0.0, 1.0)
    }

    /// Calculer le reward avec trust et détection d'anomalies
    #[allow(dead_code)]
    pub fn compute_with_trust(
        &self,
        outcome: &Outcome,
        trust_weight: f32,
        is_anomaly: bool,
    ) -> f32 {
        if is_anomaly {
            debug!("Anomaly detected, returning neutral reward");
            return 0.5; // Reward neutre pour les anomalies
        }

        let raw_reward = self.compute(outcome);
        self.apply_trust_weight(raw_reward, trust_weight)
    }

    /// Obtenir les métriques de reward
    pub fn get_reward_metrics(&self, outcomes: &[Outcome]) -> RewardMetrics {
        let mut total_rewards = 0.0;
        let mut helpful_count = 0;
        let mut used_count = 0;
        let mut reverted_count = 0;
        let mut ignored_count = 0;
        let mut dismissed_count = 0;

        for outcome in outcomes {
            let reward = self.compute(outcome);
            total_rewards += reward;

            match outcome {
                Outcome::Used {
                    helpful, reverted, ..
                } => {
                    used_count += 1;
                    if *helpful {
                        helpful_count += 1;
                    }
                    if *reverted {
                        reverted_count += 1;
                    }
                }
                Outcome::Ignored => ignored_count += 1,
                Outcome::Dismissed => dismissed_count += 1,
            }
        }

        RewardMetrics {
            total_rewards,
            average_reward: if outcomes.is_empty() {
                0.0
            } else {
                total_rewards / outcomes.len() as f32
            },
            helpful_rate: if used_count == 0 {
                0.0
            } else {
                helpful_count as f32 / used_count as f32
            },
            usage_rate: if outcomes.is_empty() {
                0.0
            } else {
                used_count as f32 / outcomes.len() as f32
            },
            reversion_rate: if used_count == 0 {
                0.0
            } else {
                reverted_count as f32 / used_count as f32
            },
            ignored_count,
            dismissed_count,
        }
    }
}

/// Métriques de reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMetrics {
    pub total_rewards: f32,
    pub average_reward: f32,
    pub helpful_rate: f32,
    pub usage_rate: f32,
    pub reversion_rate: f32,
    pub ignored_count: usize,
    pub dismissed_count: usize,
}

impl Default for RewardCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_reward_calculation() {
        let calculator = RewardCalculator::new();

        // Test outcome positif
        let positive_outcome = Outcome::Used {
            helpful: true,
            reverted: false,
            time_to_flow: Some(Duration::from_secs(10)),
        };
        let reward = calculator.compute(&positive_outcome);
        assert!(reward > 0.7);

        // Test outcome négatif
        let negative_outcome = Outcome::Used {
            helpful: false,
            reverted: true,
            time_to_flow: None,
        };
        let reward = calculator.compute(&negative_outcome);
        assert!(reward < 0.3);

        // Test ignored
        let ignored_outcome = Outcome::Ignored;
        let reward = calculator.compute(&ignored_outcome);
        assert_eq!(reward, 0.0);

        // Test dismissed
        let dismissed_outcome = Outcome::Dismissed;
        let reward = calculator.compute(&dismissed_outcome);
        assert_eq!(reward, 0.0); // Clamp à 0
    }

    #[test]
    fn test_trust_weighting() {
        let calculator = RewardCalculator::new();
        let raw_reward = 0.8;

        // Trust élevé
        let high_trust_weight = 1.2;
        let weighted = calculator.apply_trust_weight(raw_reward, high_trust_weight);
        assert_eq!(weighted, 1.0); // Clamp à 1.0

        // Trust faible
        let low_trust_weight = 0.5;
        let weighted = calculator.apply_trust_weight(raw_reward, low_trust_weight);
        assert_eq!(weighted, 0.4); // 0.8 * 0.5
    }

    #[test]
    fn test_reward_metrics() {
        let calculator = RewardCalculator::new();
        let outcomes = vec![
            Outcome::Used {
                helpful: true,
                reverted: false,
                time_to_flow: None,
            },
            Outcome::Used {
                helpful: false,
                reverted: true,
                time_to_flow: None,
            },
            Outcome::Ignored,
            Outcome::Dismissed,
        ];

        let metrics = calculator.get_reward_metrics(&outcomes);
        assert_eq!(metrics.used_count, 2);
        assert_eq!(metrics.helpful_count, 1);
        assert_eq!(metrics.reverted_count, 1);
        assert_eq!(metrics.ignored_count, 1);
        assert_eq!(metrics.dismissed_count, 1);
        assert_eq!(metrics.helpful_rate, 0.5);
        assert_eq!(metrics.usage_rate, 0.5);
        assert_eq!(metrics.reversion_rate, 0.5);
    }
}
