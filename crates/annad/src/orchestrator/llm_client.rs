//! Ollama LLM Client v0.10.0

use anna_common::{
    LlmAResponse, LlmBResponse, OllamaChatRequest, OllamaChatResponse, OllamaMessage,
    LLM_A_SYSTEM_PROMPT, LLM_B_SYSTEM_PROMPT,
};
use anyhow::{Context, Result};
use std::time::Duration;
use tracing::{debug, warn};

const OLLAMA_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_MODEL: &str = "llama3.2:3b";

/// Ollama LLM client
pub struct OllamaClient {
    http_client: reqwest::Client,
    model: String,
}

impl OllamaClient {
    pub fn new(model: Option<String>) -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .unwrap_or_default(),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
        }
    }

    /// Call LLM-A with the given prompt
    pub async fn call_llm_a(&self, user_prompt: &str) -> Result<LlmAResponse> {
        let response_text = self
            .call_ollama(LLM_A_SYSTEM_PROMPT, user_prompt)
            .await
            .context("LLM-A call failed")?;

        // Try to parse as JSON
        self.parse_llm_a_response(&response_text)
    }

    /// Call LLM-B with the given prompt
    pub async fn call_llm_b(&self, user_prompt: &str) -> Result<LlmBResponse> {
        let response_text = self
            .call_ollama(LLM_B_SYSTEM_PROMPT, user_prompt)
            .await
            .context("LLM-B call failed")?;

        // Try to parse as JSON
        self.parse_llm_b_response(&response_text)
    }

    /// Raw Ollama API call
    async fn call_ollama(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let url = format!("{}/api/chat", OLLAMA_URL);

        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            stream: false,
            format: Some("json".to_string()),
        };

        debug!("Calling Ollama model: {}", self.model);

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Ollama returned error {}: {}", status, error_text);
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(chat_response.message.content)
    }

    /// Parse LLM-A response, handling common JSON issues
    fn parse_llm_a_response(&self, text: &str) -> Result<LlmAResponse> {
        // Try direct parse first
        if let Ok(response) = serde_json::from_str::<LlmAResponse>(text) {
            return Ok(response);
        }

        // Try to extract JSON from text (LLM sometimes adds prose)
        if let Some(json_start) = text.find('{') {
            if let Some(json_end) = text.rfind('}') {
                let json_str = &text[json_start..=json_end];
                if let Ok(response) = serde_json::from_str::<LlmAResponse>(json_str) {
                    return Ok(response);
                }
            }
        }

        // Log the problematic response for debugging
        warn!("Failed to parse LLM-A response: {}", text);

        // Return a default response indicating parse failure
        Ok(LlmAResponse {
            plan: anna_common::LlmAPlan {
                intent: "parse_error".to_string(),
                probe_requests: vec![],
                can_answer_without_more_probes: false,
            },
            draft_answer: None,
            self_scores: None,
            needs_more_probes: false,
            refuse_to_answer: true,
            refusal_reason: Some(format!("Failed to parse LLM response: {}", text)),
        })
    }

    /// Parse LLM-B response, handling common JSON issues
    fn parse_llm_b_response(&self, text: &str) -> Result<LlmBResponse> {
        // Try direct parse first
        if let Ok(response) = serde_json::from_str::<LlmBResponse>(text) {
            return Ok(response);
        }

        // Try to extract JSON from text
        if let Some(json_start) = text.find('{') {
            if let Some(json_end) = text.rfind('}') {
                let json_str = &text[json_start..=json_end];
                if let Ok(response) = serde_json::from_str::<LlmBResponse>(json_str) {
                    return Ok(response);
                }
            }
        }

        warn!("Failed to parse LLM-B response: {}", text);

        // Return a refuse verdict on parse failure
        Ok(LlmBResponse {
            verdict: anna_common::AuditVerdict::Refuse,
            scores: anna_common::AuditScores::new(0.0, 0.0, 0.0),
            probe_requests: vec![],
            problems: vec![format!("Failed to parse audit response: {}", text)],
            suggested_fix: None,
        })
    }

    /// Check if Ollama is available
    pub async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", OLLAMA_URL);
        self.http_client.get(&url).send().await.is_ok()
    }

    /// Get the model being used
    pub fn model(&self) -> &str {
        &self.model
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        assert_eq!(client.model(), DEFAULT_MODEL);
    }

    #[test]
    fn test_ollama_client_custom_model() {
        let client = OllamaClient::new(Some("qwen2.5:7b".to_string()));
        assert_eq!(client.model(), "qwen2.5:7b");
    }
}
