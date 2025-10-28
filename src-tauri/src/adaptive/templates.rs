#![allow(unused_imports)]
#![allow(dead_code)]

use tracing::debug;

/// Domain templates for different application contexts
pub struct PromptTemplates;

impl PromptTemplates {
    /// Get prompt style based on intent type
    pub fn get_style(intent: &str, trust: f32) -> PromptStyle {
        match intent {
            "debugging" => {
                if trust > 0.7 {
                    PromptStyle::Concise
                } else {
                    PromptStyle::SafeConcise
                }
            }
            "learning" => PromptStyle::Pedagogical,
            "creating" => {
                if trust > 0.6 {
                    PromptStyle::Creative
                } else {
                    PromptStyle::SafeCreative
                }
            }
            "researching" => PromptStyle::Analytical,
            "stuck" => PromptStyle::Empathetic,
            _ => PromptStyle::Neutral,
        }
    }

    /// Build prompt based on domain and intent
    pub fn build_prompt(domain: &str, intent: &str, trust: f32, idle_time: f32) -> String {
        let style = Self::get_style(intent, trust);
        let trust_level = if trust > 0.8 { "high" } else if trust > 0.5 { "medium" } else { "low" };
        
        format!(
            "[ADAPTIVE PROMPT]\nDomain: {}\nIntent: {}\nTrust: {:.2} ({})\nIdle: {:.1}s\nStyle: {:?}\n\n",
            domain, intent, trust, trust_level, idle_time, style
        )
    }
}

/// Prompt generation styles
#[derive(Debug, Clone, Copy)]
pub enum PromptStyle {
    Concise,      // Direct, no fluff
    SafeConcise, // Concise but cautious
    Pedagogical, // Educational, step-by-step
    Creative,     // Free-flowing, creative
    SafeCreative, // Creative but boundaries
    Analytical,   // Deep dive, research mode
    Empathetic,   // Understanding, supportive
    Neutral,      // Balanced approach
}

