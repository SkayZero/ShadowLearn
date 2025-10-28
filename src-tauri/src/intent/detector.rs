#![allow(unused_imports)]
#![allow(dead_code)]
use lru::LruCache;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::context::aggregator::Context;
use crate::intent::llm_client::LLMClient;
use crate::intent::{Intent, IntentType};

/// Intent detector with LLM integration and caching
#[derive(Debug)]
#[allow(dead_code)]
pub struct IntentDetector {
    llm_client: Arc<Mutex<LLMClient>>,
    intent_cache: LruCache<String, CachedIntent>,
    confidence_threshold: f32,
    stats: IntentDetectorStats,
}

#[derive(Debug, Clone)]
struct CachedIntent {
    intent: Intent,
    timestamp: Instant,
    cache_key: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct IntentDetectorStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub llm_calls: u64,
    pub llm_errors: u64,
    pub average_response_time_ms: f64,
    pub total_response_time_ms: u64,
}

#[allow(dead_code)]
impl IntentDetector {
    pub fn new(llm_client: Arc<Mutex<LLMClient>>) -> Self {
        Self {
            llm_client,
            intent_cache: LruCache::new(NonZeroUsize::new(500).unwrap()),
            confidence_threshold: 0.5,
            stats: IntentDetectorStats {
                total_requests: 0,
                cache_hits: 0,
                cache_misses: 0,
                llm_calls: 0,
                llm_errors: 0,
                average_response_time_ms: 0.0,
                total_response_time_ms: 0,
            },
        }
    }

    /// Detect intent from context
    pub async fn detect_intent(&mut self, ctx: &Context) -> Result<Intent, String> {
        self.stats.total_requests += 1;

        // Generate cache key
        let cache_key = self.generate_cache_key(ctx);

        // Check cache first
        let cached_intent = self.intent_cache.get(&cache_key).cloned();
        if let Some(cached) = cached_intent {
            let cache_age = cached.timestamp.elapsed();
            let ttl = self.get_cache_ttl(&cached.intent);

            if cache_age < ttl {
                self.stats.cache_hits += 1;
                debug!(
                    "[INTENT] Cache hit: {} (age: {:?}, ttl: {:?})",
                    cached.intent.intent_type.as_str(),
                    cache_age,
                    ttl
                );
                return Ok(cached.intent.clone());
            }
        }

        self.stats.cache_misses += 1;

        // Generate fingerprint for context analysis
        let fingerprint = self.generate_context_fingerprint(ctx);

        // Build prompt
        let prompt = self.build_prompt(ctx, &fingerprint);

        // Call LLM with timeout
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(30);

        let response = match tokio::time::timeout(timeout_duration, async {
            self.llm_client.lock().unwrap().generate(&prompt, 200).await
        })
        .await
        {
            Ok(Ok(response)) => {
                self.stats.llm_calls += 1;
                let duration = start_time.elapsed();
                self.stats.total_response_time_ms += duration.as_millis() as u64;
                self.stats.average_response_time_ms =
                    self.stats.total_response_time_ms as f64 / self.stats.llm_calls as f64;
                response
            }
            Ok(Err(e)) => {
                self.stats.llm_errors += 1;
                error!("[INTENT] LLM call failed: {}", e);
                return self.create_fallback_intent(ctx);
            }
            Err(_timeout) => {
                self.stats.llm_errors += 1;
                warn!("[INTENT] LLM timeout after 30s, using fallback");
                return self.create_fallback_intent(ctx);
            }
        };

        // Parse response
        let intent = self.parse_response(&response)?;

        // Cache result
        let cached_intent = CachedIntent {
            intent: intent.clone(),
            timestamp: Instant::now(),
            cache_key: cache_key.clone(),
        };
        self.intent_cache.put(cache_key, cached_intent);

        info!(
            "[INTENT] Detected: {} (confidence: {:.2}, time: {:?})",
            intent.intent_type.as_str(),
            intent.confidence,
            start_time.elapsed()
        );

        Ok(intent)
    }

    /// Check if intent should proceed (confidence threshold)
    pub fn should_proceed(&self, intent: &Intent) -> bool {
        intent.confidence >= self.confidence_threshold
    }

    /// Generate cache key from context
    fn generate_cache_key(&self, ctx: &Context) -> String {
        // Use app name, window title, and clipboard hash for cache key
        let clipboard_hash = ctx
            .clipboard
            .as_ref()
            .map(|c| {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                c.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            })
            .unwrap_or_else(|| "none".to_string());

        format!(
            "{}|{}|{}",
            ctx.app.name, ctx.app.window_title, clipboard_hash
        )
    }

    /// Get cache TTL based on confidence
    fn get_cache_ttl(&self, intent: &Intent) -> Duration {
        if intent.confidence >= 0.9 {
            Duration::from_secs(600) // 10 minutes for high confidence
        } else if intent.confidence >= 0.7 {
            Duration::from_secs(300) // 5 minutes for medium confidence
        } else {
            Duration::from_secs(120) // 2 minutes for low confidence
        }
    }

    /// Generate context fingerprint for analysis
    fn generate_context_fingerprint(&self, ctx: &Context) -> ContextFingerprint {
        ContextFingerprint {
            app_name: ctx.app.name.clone(),
            window_title: ctx.app.window_title.clone(),
            bundle_id: ctx.app.bundle_id.clone(),
            idle_seconds: ctx.idle_seconds as f32,
            clipboard_length: ctx.clipboard.as_ref().map(|c| c.len()).unwrap_or(0),
            clipboard_keywords: self.extract_clipboard_keywords(ctx),
            domain_hints: self.extract_domain_hints(ctx),
        }
    }

    /// Extract keywords from clipboard content
    fn extract_clipboard_keywords(&self, ctx: &Context) -> Vec<String> {
        if let Some(clipboard) = &ctx.clipboard {
            clipboard
                .split_whitespace()
                .filter(|word| word.len() > 3)
                .map(|word| word.to_lowercase())
                .filter(|word| !self.is_stop_word(word))
                .take(5) // Limit to 5 keywords
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract domain hints from context
    fn extract_domain_hints(&self, ctx: &Context) -> Vec<String> {
        let mut hints = Vec::new();

        // App-based hints
        let app_lower = ctx.app.name.to_lowercase();
        if app_lower.contains("code")
            || app_lower.contains("studio")
            || app_lower.contains("editor")
        {
            hints.push("development".to_string());
        }
        if app_lower.contains("browser")
            || app_lower.contains("chrome")
            || app_lower.contains("safari")
        {
            hints.push("web".to_string());
        }
        if app_lower.contains("terminal") || app_lower.contains("console") {
            hints.push("terminal".to_string());
        }

        // Window title hints
        let title_lower = ctx.app.window_title.to_lowercase();
        if title_lower.contains("error")
            || title_lower.contains("exception")
            || title_lower.contains("debug")
        {
            hints.push("debugging".to_string());
        }
        if title_lower.contains("stackoverflow")
            || title_lower.contains("github")
            || title_lower.contains("docs")
        {
            hints.push("research".to_string());
        }

        // Clipboard hints
        if let Some(clipboard) = &ctx.clipboard {
            let clipboard_lower = clipboard.to_lowercase();
            if clipboard_lower.contains("def ")
                || clipboard_lower.contains("class ")
                || clipboard_lower.contains("import ")
            {
                hints.push("coding".to_string());
            }
            if clipboard_lower.contains("error") || clipboard_lower.contains("traceback") {
                hints.push("error_analysis".to_string());
            }
        }

        hints
    }

    /// Check if word is a stop word
    fn is_stop_word(&self, word: &str) -> bool {
        let stop_words = [
            "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by", "a",
            "an", "is", "are", "was", "were", "be", "been", "have", "has", "had", "this", "that",
            "these", "those", "will", "would", "could", "should",
        ];
        stop_words.contains(&word)
    }

    /// Build prompt for LLM
    fn build_prompt(&self, ctx: &Context, fingerprint: &ContextFingerprint) -> String {
        let domain_hints = fingerprint.domain_hints.join(", ");
        let clipboard_keywords = fingerprint.clipboard_keywords.join(", ");

        format!(
            r#"You are analyzing user intent in a software application context.

Context:
- App: {} ({})
- Window: {}
- Idle time: {:.1} seconds
- Clipboard length: {} characters
- Clipboard keywords: {}
- Domain hints: {}

Additional signals:
- Idle > 20s suggests user might be stuck
- Clipboard contains code suggests debugging/learning
- Browser + StackOverflow suggests researching
- Terminal suggests system administration

Respond with JSON only:
{{
  "intent": "one of: debugging, learning, creating, researching, stuck",
  "confidence": 0.0-1.0,
  "reason": "brief explanation of why this intent was detected"
}}

Response:"#,
            ctx.app.name,
            ctx.app.bundle_id,
            ctx.app.window_title,
            ctx.idle_seconds,
            fingerprint.clipboard_length,
            if clipboard_keywords.is_empty() {
                "none"
            } else {
                &clipboard_keywords
            },
            if domain_hints.is_empty() {
                "none"
            } else {
                &domain_hints
            }
        )
    }

    /// Parse LLM response
    fn parse_response(&self, response: &str) -> Result<Intent, String> {
        // Clean response (remove markdown, extra whitespace)
        let cleaned_response = response
            .trim()
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .collect::<Vec<_>>()
            .join(" ");

        // Try to find JSON in response
        let json_start = cleaned_response.find('{');
        let json_end = cleaned_response.rfind('}');

        let json_str = if let (Some(start), Some(end)) = (json_start, json_end) {
            &cleaned_response[start..=end]
        } else {
            &cleaned_response
        };

        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse failed: {} (response: {})", e, json_str))?;

        let intent_str = json["intent"].as_str().ok_or("Missing intent field")?;
        let confidence = json["confidence"]
            .as_f64()
            .ok_or("Missing confidence field")? as f32;
        let reason = json["reason"]
            .as_str()
            .unwrap_or("No reason provided")
            .to_string();

        let intent_type = IntentType::from_str(intent_str);

        Ok(Intent {
            intent_type,
            confidence: confidence.clamp(0.0, 1.0),
            reason,
            detected_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    /// Clear intent cache
    pub fn clear_cache(&mut self) {
        self.intent_cache.clear();
    }

    /// Get cache hit rate
    pub fn get_cache_hit_rate(&self) -> f64 {
        let total_requests = self.stats.cache_hits + self.stats.cache_misses;
        if total_requests == 0 {
            return 0.0;
        }
        self.stats.cache_hits as f64 / total_requests as f64
    }

    /// Create fallback intent when LLM fails
    fn create_fallback_intent(&self, ctx: &Context) -> Result<Intent, String> {
        // Heuristic-based intent detection as fallback
        let intent_type = self.heuristic_intent_detection(ctx);
        let confidence = 0.3; // Low confidence for fallback

        let reason = format!("Fallback heuristic: {}", intent_type.as_str());

        Ok(Intent {
            intent_type,
            confidence,
            reason,
            detected_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    /// Heuristic intent detection based on context patterns
    fn heuristic_intent_detection(&self, ctx: &Context) -> IntentType {
        let app_lower = ctx.app.name.to_lowercase();
        let title_lower = ctx.app.window_title.to_lowercase();

        // Debugging patterns
        if title_lower.contains("error")
            || title_lower.contains("exception")
            || title_lower.contains("debug")
        {
            return IntentType::Debugging;
        }

        // Learning patterns
        if app_lower.contains("browser")
            && (title_lower.contains("stackoverflow") || title_lower.contains("docs"))
        {
            return IntentType::Learning;
        }

        // Creating patterns
        if app_lower.contains("code") || app_lower.contains("editor") {
            return IntentType::Creating;
        }

        // Stuck patterns
        if ctx.idle_seconds > 60.0 {
            return IntentType::Stuck;
        }

        // Default to unknown
        IntentType::Unknown
    }

    /// Get detector statistics
    pub fn get_stats(&self) -> &IntentDetectorStats {
        &self.stats
    }

    /// Get cache size
    pub fn get_cache_size(&self) -> usize {
        self.intent_cache.len()
    }
}

/// Context fingerprint for intent analysis
#[derive(Debug, Clone)]
struct ContextFingerprint {
    app_name: String,
    window_title: String,
    bundle_id: String,
    idle_seconds: f32,
    clipboard_length: usize,
    clipboard_keywords: Vec<String>,
    domain_hints: Vec<String>,
}
