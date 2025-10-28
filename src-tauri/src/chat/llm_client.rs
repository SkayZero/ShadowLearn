use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub include_context: bool,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub provider: String,
    pub used_fallback: bool,
    pub ttfr_ms: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LLMClientStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub fallback_used: u64,
    pub avg_ttfr_ms: f64,
}

pub struct LLMChatClient {
    primary_provider: LLMProvider,
    fallback_provider: LLMProvider,
    stats: Arc<tokio::sync::Mutex<LLMClientStats>>,
}

#[derive(Debug, Clone)]
pub enum LLMProvider {
    OpenAI { api_key: String, base_url: String },
    Ollama { base_url: String },
}

impl LLMChatClient {
    pub fn new() -> Self {
        Self {
            primary_provider: Self::detect_primary_provider(),
            fallback_provider: LLMProvider::Ollama {
                base_url: "http://localhost:11434".to_string(),
            },
            stats: Arc::new(tokio::sync::Mutex::new(LLMClientStats {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                fallback_used: 0,
                avg_ttfr_ms: 0.0,
            })),
        }
    }

    fn detect_primary_provider() -> LLMProvider {
        // Try to detect from environment or use Ollama as default
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            LLMProvider::OpenAI {
                api_key,
                base_url: "https://api.openai.com/v1".to_string(),
            }
        } else {
            LLMProvider::Ollama {
                base_url: "http://localhost:11434".to_string(),
            }
        }
    }

    /// Check health of both providers
    pub async fn check_health(&self) -> (bool, bool) {
        let primary_ok = self.check_provider_health(&self.primary_provider).await;
        let fallback_ok = self.check_provider_health(&self.fallback_provider).await;
        (primary_ok, fallback_ok)
    }

    async fn check_provider_health(&self, provider: &LLMProvider) -> bool {
        match provider {
            LLMProvider::OpenAI { .. } => {
                // For OpenAI, we just check if API key exists
                true
            }
            LLMProvider::Ollama { base_url } => {
                let health_url = format!("{}/api/tags", base_url);
                let client = reqwest::Client::new();
                match timeout(Duration::from_secs(2), client.get(&health_url).send()).await {
                    Ok(Ok(resp)) => resp.status().is_success(),
                    Ok(Err(_)) => false,
                    Err(_) => false,
                }
            }
        }
    }

    /// Send chat request with timeout, retries, and fallback
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, String> {
        let start = std::time::Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_requests += 1;
        }

        // Try primary provider with retries
        let mut last_error = None;
        for attempt in 0..3 {
            match timeout(
                Duration::from_secs(12),
                self.send_to_provider(&self.primary_provider, &request, attempt),
            )
            .await
            {
                Ok(Ok(response)) => {
                    let ttfr = start.elapsed().as_millis() as u64;
                    self.record_success(false, ttfr).await;
                    return Ok(ChatResponse {
                        content: response,
                        provider: self.provider_name(&self.primary_provider),
                        used_fallback: false,
                        ttfr_ms: ttfr,
                    });
                }
                Ok(Err(e)) => {
                    last_error = Some(e);
                    warn!("Attempt {} failed: {}", attempt + 1, last_error.as_ref().unwrap());
                    // Exponential backoff: 2s, 4s, 8s
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt + 1))).await;
                }
                Err(_) => {
                    last_error = Some("Timeout after 12s".to_string());
                }
            }
        }

        // If primary failed, try fallback
        warn!("Primary provider failed, trying fallback...");
        match timeout(
            Duration::from_secs(12),
            self.send_to_provider(&self.fallback_provider, &request, 0),
        )
        .await
        {
            Ok(Ok(response)) => {
                let ttfr = start.elapsed().as_millis() as u64;
                self.record_success(true, ttfr).await;
                info!("✅ Fallback successful");
                Ok(ChatResponse {
                    content: response,
                    provider: self.provider_name(&self.fallback_provider),
                    used_fallback: true,
                    ttfr_ms: ttfr,
                })
            }
            Ok(Err(e)) => {
                let ttfr = start.elapsed().as_millis() as u64;
                self.record_failure().await;
                error!("❌ Both providers failed. Last error: {}", e);
                Err(format!("LLM unavailable (both providers failed): {}", e))
            }
            Err(_) => {
                let ttfr = start.elapsed().as_millis() as u64;
                self.record_failure().await;
                error!("❌ Fallback timeout after 12s");
                Err("LLM timeout (both providers)".to_string())
            }
        }
    }

    async fn send_to_provider(
        &self,
        provider: &LLMProvider,
        request: &ChatRequest,
        _attempt: u32,
    ) -> Result<String, String> {
        match provider {
            LLMProvider::OpenAI { api_key, base_url } => {
                self.send_to_openai(api_key, base_url, request).await
            }
            LLMProvider::Ollama { base_url } => self.send_to_ollama(base_url, request).await,
        }
    }

    async fn send_to_openai(
        &self,
        api_key: &str,
        base_url: &str,
        request: &ChatRequest,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", base_url);

        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<ChatMessage>,
            temperature: f32,
        }

        let body = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: request.messages.clone(),
            temperature: request.temperature,
        };

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, text));
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<Choice>,
        }

        #[derive(Deserialize)]
        struct Choice {
            message: ChatMessage,
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(openai_response.choices[0].message.content.clone())
    }

    async fn send_to_ollama(&self, base_url: &str, request: &ChatRequest) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/chat", base_url);

        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            messages: Vec<ChatMessage>,
            stream: bool,
        }

        // Try models in order: llama3, qwen2.5:3b, llama3.2 (fallback)
        let models = vec!["llama3", "qwen2.5:3b", "llama3.2"];
        let mut last_error = None;
        
        for model in models {
            let body = OllamaRequest {
                model: model.to_string(),
                messages: request.messages.clone(),
                stream: false,
            };

            match client.post(&url).json(&body).send().await {
                Ok(response) if response.status().is_success() => {
                    #[derive(Deserialize)]
                    struct OllamaResponse {
                        message: ChatMessage,
                    }
                    
                    let ollama_response: OllamaResponse = response
                        .json()
                        .await
                        .map_err(|e| format!("Parse error: {}", e))?;
                    info!("✅ Used model: {}", model);
                    return Ok(ollama_response.message.content.clone());
                }
                Ok(response) => {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    last_error = Some(format!("API error {}: {}", status, text));
                    warn!("❌ Model '{}' failed: {}", model, last_error.as_ref().unwrap());
                    continue;
                }
                Err(e) => {
                    last_error = Some(format!("Network error: {}", e));
                    warn!("❌ Model '{}' network error: {}", model, e);
                    continue;
                }
            }
        }

        return Err(last_error.unwrap_or("All models failed".to_string()));
    }

    async fn _parse_ollama_response(&self, response: reqwest::Response) -> Result<String, String> {
        #[derive(Deserialize)]
        struct OllamaResponse {
            message: ChatMessage,
        }

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        Ok(ollama_response.message.content.clone())
    }

    fn provider_name(&self, provider: &LLMProvider) -> String {
        match provider {
            LLMProvider::OpenAI { .. } => "openai".to_string(),
            LLMProvider::Ollama { .. } => "ollama".to_string(),
        }
    }

    async fn record_success(&self, used_fallback: bool, ttfr: u64) {
        let mut stats = self.stats.lock().await;
        stats.successful_requests += 1;
        if used_fallback {
            stats.fallback_used += 1;
        }
        // Simple moving average
        let n = stats.successful_requests;
        stats.avg_ttfr_ms = (stats.avg_ttfr_ms * (n - 1) as f64 + ttfr as f64) / n as f64;
    }

    async fn record_failure(&self) {
        let mut stats = self.stats.lock().await;
        stats.failed_requests += 1;
    }

    pub async fn get_stats(&self) -> LLMClientStats {
        self.stats.lock().await.clone()
    }
}

