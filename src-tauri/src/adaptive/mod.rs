#![allow(unused_imports)]
#![allow(dead_code)]

pub mod templates;
pub mod builder;
pub mod cache;

// Re-export submodules
pub use templates::*;
pub use builder::*;
pub use cache::*;

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Adaptive prompting engine that generates context-aware prompts
#[derive(Debug)]
pub struct AdaptivePromptEngine {
    builder: PromptBuilder,
    cache: PromptCache,
}

impl AdaptivePromptEngine {
    pub fn new() -> Self {
        Self {
            builder: PromptBuilder::new(),
            cache: PromptCache::new(),
        }
    }

    /// Generate an adaptive prompt based on context, intent, and trust
    pub async fn generate_prompt(
        &mut self,
        domain: &str,
        intent: &str,
        trust_score: f32,
        idle_time: f32,
        _cluster_id: &str,
    ) -> Result<String, String> {
        info!("[ADAPTIVE] Generating prompt - domain={}, intent={}, trust={:.2}, idle={:.1}s, cluster={}",
              domain, intent, trust_score, idle_time, _cluster_id);

        // Check cache first
        let cache_key = format!("{}:{}:{}", _cluster_id, intent, domain);
        if let Some(cached) = self.cache.get(&cache_key) {
            info!("[ADAPTIVE] Cache hit for key: {}", cache_key);
            return Ok(cached);
        }

        // Build adaptive prompt
        let prompt = self.builder.build(domain.to_string(), intent.to_string(), trust_score, idle_time)?;
        
        // Cache the result
        self.cache.put(&cache_key, prompt.clone());
        
        Ok(prompt)
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats()
    }

    /// Clear the prompt cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        info!("[ADAPTIVE] Cache cleared");
    }
}

// CacheStats is exported from cache.rs
pub use cache::CacheStats;

