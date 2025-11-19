use serde::{Deserialize, Serialize};

/// Configuration globale de ShadowLearn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub apps: AppsConfig,
    pub triggers: TriggersConfig,
    pub llm: LLMConfig,
    pub shortcuts: ShortcutsConfig,
    pub captures: CapturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConfig {
    pub allowlist: Vec<String>,
    pub blocklist: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggersConfig {
    pub idle_threshold_s: u64,
    pub cooldown_accept_s: u64,
    pub cooldown_dismiss_s: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: String, // "openai" | "anthropic" | "ollama"
    pub model: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutsConfig {
    pub show: String,        // Default: "CmdOrCtrl+Shift+S"
    pub chat: String,        // Default: "CmdOrCtrl+K"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturesConfig {
    pub enabled: bool,
    pub min_interval_s: u64,
}

/// Statistiques de confidentialité
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStats {
    pub screenshots_count: usize,
    pub screenshots_size_bytes: u64,
    pub contexts_count: usize,
    pub db_size_bytes: u64,
    pub data_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            apps: AppsConfig {
                allowlist: vec![],
                blocklist: vec![],
            },
            triggers: TriggersConfig {
                idle_threshold_s: 12,
                cooldown_accept_s: 45,
                cooldown_dismiss_s: 90,
            },
            llm: LLMConfig {
                provider: "ollama".to_string(),
                model: "llama3".to_string(),
                api_key: None,
            },
            shortcuts: ShortcutsConfig {
                show: "CmdOrCtrl+Shift+S".to_string(),
                chat: "CmdOrCtrl+K".to_string(),
            },
            captures: CapturesConfig {
                enabled: false,
                min_interval_s: 60,
            },
        }
    }
}

