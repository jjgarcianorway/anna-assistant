//! LLM client - communicates with Ollama

use anna_common::{OllamaChatRequest, OllamaChatResponse, OllamaMessage};
use anyhow::{Context, Result};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";

/// Client for communicating with Ollama
pub struct LlmClient {
    client: reqwest::Client,
    base_url: String,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: OLLAMA_URL.to_string(),
        }
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.base_url);
        self.client.get(&url).send().await.is_ok()
    }

    /// Send a chat request to a model
    pub async fn chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: user_message.to_string(),
                },
            ],
            stream: false,
            format: Some("json".to_string()),
        };

        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama request failed ({}): {}", status, text);
        }

        let chat_resp: OllamaChatResponse = resp
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(chat_resp.message.content)
    }

    /// Send a chat with history
    pub async fn chat_with_history(
        &self,
        model: &str,
        messages: Vec<OllamaMessage>,
    ) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages,
            stream: false,
            format: Some("json".to_string()),
        };

        let resp = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Ollama request failed ({}): {}", status, text);
        }

        let chat_resp: OllamaChatResponse = resp
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(chat_resp.message.content)
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}
