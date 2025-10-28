#![allow(unused_imports)]
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::debug;

use crate::context::aggregator::Context;

/// Context fingerprint generator using SimHash algorithm
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FingerprintGenerator {
    hash_weights: Vec<f64>, // Pre-computed weights for different features
}

/// Context fingerprint with SimHash
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ContextFingerprint {
    pub simhash: u64,
    pub domain: String,
    pub features: Vec<String>,
    pub generated_at_ms: u64,
}

#[allow(dead_code)]
impl FingerprintGenerator {
    pub fn new() -> Self {
        Self {
            hash_weights: vec![1.0, 0.8, 0.6, 0.4, 0.2], // Decreasing importance
        }
    }

    /// Generate fingerprint from context
    pub fn generate(&self, ctx: &Context) -> ContextFingerprint {
        let features = self.extract_features(ctx);
        let simhash = self.compute_simhash(&features);

        debug!(
            "[FINGERPRINT] Generated for {}: simhash={:016x}, features={}",
            ctx.app.name,
            simhash,
            features.len()
        );

        ContextFingerprint {
            simhash,
            domain: ctx.app.name.clone(),
            features,
            generated_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Extract relevant features from context
    fn extract_features(&self, ctx: &Context) -> Vec<String> {
        let mut features = Vec::new();

        // App name (highest weight)
        features.push(format!("app:{}", ctx.app.name.to_lowercase()));

        // Window title keywords (medium weight)
        let title_words = self.extract_keywords(&ctx.app.window_title);
        for word in title_words {
            features.push(format!("title:{}", word));
        }

        // Clipboard content keywords (medium weight)
        if let Some(clipboard) = &ctx.clipboard {
            let clipboard_words = self.extract_keywords(clipboard);
            for word in clipboard_words.iter().take(10) {
                // Limit to 10 words
                features.push(format!("clip:{}", word));
            }
        }

        // Bundle ID (low weight)
        features.push(format!("bundle:{}", ctx.app.bundle_id));

        // Idle state (very low weight)
        let idle_state = if ctx.idle_seconds < 5.0 {
            "active"
        } else if ctx.idle_seconds < 30.0 {
            "short_idle"
        } else {
            "long_idle"
        };
        features.push(format!("idle:{}", idle_state));

        features
    }

    /// Extract keywords from text (simple implementation)
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .filter(|word| word.len() > 2) // Filter short words
            .map(|word| word.to_lowercase())
            .filter(|word| !self.is_stop_word(word)) // Filter stop words
            .collect()
    }

    /// Check if word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        let stop_words = [
            "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "a",
            "an", "is", "are", "was", "were", "be", "been", "have", "has", "had", "do", "does",
            "did", "will", "would", "could", "should", "may", "might", "this", "that", "these",
            "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
            "my", "your", "his", "her", "its", "our", "their",
        ];
        stop_words.contains(&word)
    }

    /// Compute SimHash from features
    fn compute_simhash(&self, features: &[String]) -> u64 {
        let mut hash_counts = vec![0i32; 64];

        for (i, feature) in features.iter().enumerate() {
            let weight = if i < self.hash_weights.len() {
                self.hash_weights[i]
            } else {
                0.1 // Default low weight
            };

            let hash = self.hash_feature(feature);

            for bit in 0..64 {
                if (hash >> bit) & 1 == 1 {
                    hash_counts[bit] += (weight * 10.0) as i32; // Scale weight
                } else {
                    hash_counts[bit] -= (weight * 10.0) as i32;
                }
            }
        }

        let mut simhash = 0u64;
        for (bit, count) in hash_counts.iter().enumerate() {
            if *count > 0 {
                simhash |= 1u64 << bit;
            }
        }

        simhash
    }

    /// Hash a single feature
    fn hash_feature(&self, feature: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        feature.hash(&mut hasher);
        hasher.finish()
    }

    /// Compute similarity between two SimHashes
    pub fn similarity(hash1: u64, hash2: u64) -> f32 {
        let hamming_distance = (hash1 ^ hash2).count_ones();
        1.0 - (hamming_distance as f32 / 64.0)
    }

    /// Compute similarity between fingerprint and cluster centroid
    pub fn similarity_to_cluster(&self, fp: &ContextFingerprint, cluster_centroid: u64) -> f32 {
        Self::similarity(fp.simhash, cluster_centroid)
    }
}

#[allow(dead_code)]
impl ContextFingerprint {
    /// Get similarity to another fingerprint
    pub fn similarity_to(&self, other: &ContextFingerprint) -> f32 {
        FingerprintGenerator::similarity(self.simhash, other.simhash)
    }

    /// Check if fingerprint is similar to another (above threshold)
    pub fn is_similar_to(&self, other: &ContextFingerprint, threshold: f32) -> bool {
        self.similarity_to(other) >= threshold
    }

    /// Get age of fingerprint
    pub fn age(&self) -> std::time::Duration {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        std::time::Duration::from_millis(now_ms - self.generated_at_ms)
    }

    /// Check if fingerprint is stale (older than given duration)
    pub fn is_stale(&self, max_age: std::time::Duration) -> bool {
        self.age() > max_age
    }
}
