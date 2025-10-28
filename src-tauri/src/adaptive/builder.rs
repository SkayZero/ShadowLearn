#![allow(unused_imports)]
#![allow(dead_code)]

use super::templates::{PromptTemplates, PromptStyle};
use tracing::{debug, info};

/// Builder for adaptive prompts
#[derive(Debug)]
pub struct PromptBuilder;

impl PromptBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build an adaptive prompt based on context
    pub fn build(
        &self,
        domain: String,
        intent: String,
        trust_score: f32,
        idle_time: f32,
    ) -> Result<String, String> {
        debug!("[PROMPT BUILDER] Building prompt - domain={}, intent={}, trust={:.2}, idle={:.1}s",
               domain, intent, trust_score, idle_time);

        // Validate inputs
        if trust_score < 0.0 || trust_score > 1.0 {
            return Err("Trust score must be between 0 and 1".to_string());
        }

        // Build adaptive prompt
        let prompt = PromptTemplates::build_prompt(&domain, &intent, trust_score, idle_time);
        
        info!("[PROMPT BUILDER] Generated prompt with {} characters", prompt.len());
        
        Ok(prompt)
    }

    /// Adjust prompt based on trust level
    fn adjust_for_trust(prompt: &str, trust: f32) -> String {
        if trust < 0.3 {
            format!("[SAFE MODE] {}\n\n⚠️ Low trust detected - using conservative approach", prompt)
        } else if trust > 0.9 {
            format!("[CREATIVE MODE] {}\n\n✨ High trust - creative solutions encouraged", prompt)
        } else {
            prompt.to_string()
        }
    }
}

