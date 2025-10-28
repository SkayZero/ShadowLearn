#![allow(unused_imports)]
#![allow(dead_code)]
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error, info};

/// LLM client supporting multiple providers
#[derive(Debug)]
#[allow(dead_code)]
pub struct LLMClient {
    client: Client,
    provider: LLMProvider,
    api_key: Option<String>,
    base_url: String,
    timeout: Duration,
    stats: LLMClientStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LLMProvider {
    Ollama,
    OpenAI,
    Anthropic,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LLMClientStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub total_response_time_ms: u64,
    pub provider: LLMProvider,
}

#[allow(dead_code)]
impl LLMClient {
    pub fn new(provider: LLMProvider, api_key: Option<String>) -> Self {
        let base_url = match provider {
            LLMProvider::Ollama => "http://localhost:11434".to_string(),
            LLMProvider::OpenAI => "https://api.openai.com/v1".to_string(),
            LLMProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            provider,
            api_key,
            base_url,
            timeout: Duration::from_secs(30),
            stats: LLMClientStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                total_response_time_ms: 0,
                provider,
            },
        }
    }

    /// Generate text using LLM
    pub async fn generate(&mut self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let start_time = std::time::Instant::now();
        self.stats.total_requests += 1;

        let result = match self.provider {
            LLMProvider::Ollama => self.generate_ollama(prompt, max_tokens).await,
            LLMProvider::OpenAI => self.generate_openai(prompt, max_tokens).await,
            LLMProvider::Anthropic => self.generate_anthropic(prompt, max_tokens).await,
        };

        let duration = start_time.elapsed();
        self.stats.total_response_time_ms += duration.as_millis() as u64;
        self.stats.average_response_time_ms =
            self.stats.total_response_time_ms as f64 / self.stats.total_requests as f64;

        match result {
            Ok(response) => {
                self.stats.successful_requests += 1;
                debug!(
                    "[LLM] Generated response in {:?} (provider: {:?})",
                    duration, self.provider
                );
                Ok(response)
            }
            Err(e) => {
                self.stats.failed_requests += 1;
                error!(
                    "[LLM] Generation failed: {} (provider: {:?})",
                    e, self.provider
                );
                Err(e)
            }
        }
    }

    /// Generate using Ollama (local)
    async fn generate_ollama(&self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let payload = json!({
            "model": "llama3.2",
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": max_tokens,
                "temperature": 0.3,
                "top_p": 0.9,
                "stop": ["```", "\n\n\n"]
            }
        });

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama API error: {}", response.status()));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Response parse failed: {}", e))?;

        json["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No response field in Ollama response".into())
    }

    /// Generate using OpenAI API
    async fn generate_openai(&self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or("OpenAI API key not configured")?;

        let payload = json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": "You are an AI assistant that analyzes user intent from software context. Respond only with valid JSON."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": max_tokens,
            "temperature": 0.3,
            "top_p": 0.9,
            "stop": ["```", "\n\n\n"]
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error: {} - {}", status, error_text));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Response parse failed: {}", e))?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in OpenAI response".into())
    }

    /// Generate using Anthropic Claude API
    async fn generate_anthropic(&self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or("Anthropic API key not configured")?;

        let payload = json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": max_tokens,
            "temperature": 0.3,
            "messages": [
                {
                    "role": "user",
                    "content": format!("{}\n\nRespond only with valid JSON.", prompt)
                }
            ]
        });

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error: {} - {}", status, error_text));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Response parse failed: {}", e))?;

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in Anthropic response".into())
    }

    /// Switch provider dynamically
    pub fn switch_provider(&mut self, new_provider: LLMProvider, api_key: Option<String>) {
        self.provider = new_provider;
        self.api_key = api_key;

        self.base_url = match new_provider {
            LLMProvider::Ollama => "http://localhost:11434".to_string(),
            LLMProvider::OpenAI => "https://api.openai.com/v1".to_string(),
            LLMProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
        };

        info!("[LLM] Switched to provider: {:?}", new_provider);
    }

    /// Check if provider is available
    pub async fn check_availability(&self) -> bool {
        match self.provider {
            LLMProvider::Ollama => self.check_ollama_availability().await,
            LLMProvider::OpenAI => self.check_openai_availability().await,
            LLMProvider::Anthropic => self.check_anthropic_availability().await,
        }
    }

    async fn check_ollama_availability(&self) -> bool {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        response.is_ok()
    }

    async fn check_openai_availability(&self) -> bool {
        if self.api_key.is_none() {
            return false;
        }

        let payload = json!({
            "model": "gpt-4o-mini",
            "messages": [{"role": "user", "content": "test"}],
            "max_tokens": 1
        });

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
            .json(&payload)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        response.is_ok()
    }

    async fn check_anthropic_availability(&self) -> bool {
        if self.api_key.is_none() {
            return false;
        }

        let payload = json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "test"}]
        });

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        response.is_ok()
    }

    /// Get client statistics
    pub fn get_stats(&self) -> &LLMClientStats {
        &self.stats
    }

    /// Get success rate
    pub fn get_success_rate(&self) -> f64 {
        if self.stats.total_requests == 0 {
            return 0.0;
        }
        self.stats.successful_requests as f64 / self.stats.total_requests as f64
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = LLMClientStats {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            total_response_time_ms: 0,
            provider: self.provider,
        };
    }
}

#[allow(dead_code)]
impl LLMProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            LLMProvider::Ollama => "ollama",
            LLMProvider::OpenAI => "openai",
            LLMProvider::Anthropic => "anthropic",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ollama" => LLMProvider::Ollama,
            "openai" => LLMProvider::OpenAI,
            "anthropic" => LLMProvider::Anthropic,
            _ => LLMProvider::Ollama, // Default to Ollama
        }
    }
}
