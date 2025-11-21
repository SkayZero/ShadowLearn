use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize)]
struct VisionMessage {
    role: String,
    content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
}

#[derive(Debug, Clone, Serialize)]
struct ImageSource {
    #[serde(rename = "type")]
    source_type: String, // "base64"
    media_type: String,  // "image/jpeg"
    data: String,        // base64 data
}

#[derive(Debug, Clone, Serialize)]
struct VisionRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<VisionMessage>,
}

#[derive(Debug, Clone, Deserialize)]
struct VisionResponse {
    content: Vec<ResponseContent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum ResponseContent {
    #[serde(rename = "text")]
    Text { text: String },
}

pub struct ClaudeVisionClient {
    api_key: String,
    base_url: String,
}

impl ClaudeVisionClient {
    pub fn new() -> Result<Self, String> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "ANTHROPIC_API_KEY not found in environment".to_string())?;

        Ok(Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        })
    }

    /// Analyse une image avec Claude Vision
    pub async fn analyze_screenshot(
        &self,
        image_base64: &str,
        prompt: &str,
    ) -> Result<String, String> {
        let start = std::time::Instant::now();

        let message = VisionMessage {
            role: "user".to_string(),
            content: vec![
                ContentBlock::Image {
                    source: ImageSource {
                        source_type: "base64".to_string(),
                        media_type: "image/jpeg".to_string(),
                        data: image_base64.to_string(),
                    },
                },
                ContentBlock::Text {
                    text: prompt.to_string(),
                },
            ],
        };

        let request_body = VisionRequest {
            model: "claude-3-haiku-20240307".to_string(), // Utilise Haiku pour la rapidité
            max_tokens: 1024,
            messages: vec![message],
        };

        let client = reqwest::Client::new();
        let url = format!("{}/messages", self.base_url);

        info!("🔍 Sending image to Claude Vision API...");

        let response = timeout(
            Duration::from_secs(30),
            client
                .post(&url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&request_body)
                .send(),
        )
        .await
        .map_err(|_| "Request timeout after 30s".to_string())?
        .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("❌ Claude Vision API error {}: {}", status, text);
            return Err(format!("API error {}: {}", status, text));
        }

        let vision_response: VisionResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        let analysis = vision_response
            .content
            .iter()
            .find_map(|block| match block {
                ResponseContent::Text { text } => Some(text.clone()),
            })
            .ok_or_else(|| "No text response from API".to_string())?;

        let duration = start.elapsed();
        info!("✅ Claude Vision analysis completed in {}ms", duration.as_millis());

        Ok(analysis)
    }

    /// Génère une suggestion basée sur le contexte visuel
    pub async fn suggest_action(&self, image_base64: &str) -> Result<String, String> {
        let prompt = r#"Analyze this screenshot and suggest helpful actions the user might want to take.

Focus on:
1. What application or task is the user working on?
2. What could be automated or improved?
3. Are there any learning opportunities?

Respond with 1-3 concise, actionable suggestions. Be specific and helpful."#;

        self.analyze_screenshot(image_base64, prompt).await
    }

    /// Détecte le domaine/contexte de travail
    pub async fn detect_context(&self, image_base64: &str) -> Result<String, String> {
        let prompt = r#"Analyze this screenshot and identify:
1. The domain (coding, design, music, writing, etc.)
2. The current task or focus
3. The user's apparent goal

Respond in 1-2 sentences."#;

        self.analyze_screenshot(image_base64, prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation_without_key() {
        // Should fail if no API key
        std::env::remove_var("ANTHROPIC_API_KEY");
        let result = ClaudeVisionClient::new();
        assert!(result.is_err());
    }

    #[test]
    fn test_client_creation_with_key() {
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        let result = ClaudeVisionClient::new();
        assert!(result.is_ok());
    }
}
