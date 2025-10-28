#![allow(unused_imports)]
#![allow(dead_code)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info};

use serde::{Deserialize, Serialize};

/// Cache for adaptive prompts
#[derive(Debug)]
pub struct PromptCache {
    cache: HashMap<String, (String, u64)>, // (key, (prompt, timestamp_ms))
    ttl_ms: u64,
    hit_count: u64,
    miss_count: u64,
}

impl PromptCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            ttl_ms: 600_000, // 10 minutes TTL
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// Get cached prompt if not expired
    pub fn get(&mut self, key: &str) -> Option<String> {
        self.cleanup(); // Cleanup before get
        if let Some((prompt, timestamp_ms)) = self.cache.get(key) {
            let now_ms = Self::now_ms();
            if now_ms - timestamp_ms < self.ttl_ms {
                self.hit_count += 1;
                debug!("[CACHE] Hit for key: {}", key);
                return Some(prompt.clone());
            }
        }
        self.miss_count += 1;
        debug!("[CACHE] Miss for key: {}", key);
        None
    }

    /// Put a prompt in the cache
    pub fn put(&mut self, key: &str, prompt: String) {
        let now_ms = Self::now_ms();
        self.cache.insert(key.to_string(), (prompt, now_ms));
        debug!("[CACHE] Stored key: {}", key);
        
        // Cleanup expired entries if cache gets large
        if self.cache.len() > 100 {
            self.cleanup();
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        let total = self.hit_count + self.miss_count;
        let hit_rate = if total > 0 {
            self.hit_count as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            total_requests: total,
            hit_rate,
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hit_count = 0;
        self.miss_count = 0;
        info!("[CACHE] Cleared");
    }

    /// Remove expired entries
    fn cleanup(&mut self) {
        let now_ms = Self::now_ms();
        let expired_keys: Vec<String> = self.cache
            .iter()
            .filter(|(_, (_, timestamp_ms))| now_ms - *timestamp_ms >= self.ttl_ms)
            .map(|(key, _)| key.clone())
            .collect();

        let cleanup_count = expired_keys.len();
        for key in &expired_keys {
            self.cache.remove(key);
        }

        debug!("[CACHE] Cleaned up {} expired entries", cleanup_count);
    }

    /// Get current timestamp in milliseconds
    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

/// Statistics for the adaptive prompt cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub total_requests: u64,
    pub hit_rate: f64,
}

