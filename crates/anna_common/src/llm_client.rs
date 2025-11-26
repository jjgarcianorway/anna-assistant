//! LLM Client Abstraction - v10.0.2
//!
//! Provides a generic interface for calling LLM backends with proper multi-turn conversations.
//! Uses Ollama's chat API for better context handling and JSON compliance.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "http://localhost:11434".to_string(),
            model: "qwen2.5:14b".to_string(),
            api_key: None,
            timeout_secs: 60,
        }
    }
}

/// LLM errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum LlmError {
    #[error("LLM is disabled in configuration")]
    Disabled,

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Invalid JSON response: {0}")]
    InvalidJson(String),

    #[error("Request timeout after {0} seconds")]
    Timeout(u64),

    #[error("LLM returned empty response")]
    EmptyResponse,
}

/// Generic LLM client trait
pub trait LlmClient: Send + Sync {
    /// Call LLM with a prompt and expect JSON response
    fn call_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema_description: &str,
    ) -> Result<serde_json::Value, LlmError>;
}

/// Real LLM client implementation using HTTP
pub struct HttpLlmClient {
    config: LlmConfig,
    client: reqwest::blocking::Client,
}

impl HttpLlmClient {
    pub fn new(config: LlmConfig) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { config, client })
    }
}

impl LlmClient for HttpLlmClient {
    fn call_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        _schema_description: &str,
    ) -> Result<serde_json::Value, LlmError> {
        if !self.config.enabled {
            return Err(LlmError::Disabled);
        }

        // Use chat API for better multi-turn handling
        let url = format!("{}/api/chat", self.config.endpoint);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "stream": false,
            "format": "json",
            "options": {
                "temperature": 0.1,  // Low temperature for consistent JSON
                "num_predict": 1024, // Enough tokens for response
            }
        });

        // Try up to 3 times for valid JSON
        let mut last_error = LlmError::EmptyResponse;
        for attempt in 0..3 {
            match self.call_ollama_chat(&url, &request_body) {
                Ok(json) => return Ok(json),
                Err(e) => {
                    tracing::debug!("LLM attempt {} failed: {}", attempt + 1, e);
                    last_error = e;
                    // Continue to retry
                }
            }
        }

        Err(last_error)
    }
}

impl HttpLlmClient {
    /// Call Ollama chat API
    fn call_ollama_chat(&self, url: &str, body: &serde_json::Value) -> Result<serde_json::Value, LlmError> {
        let response = self.client
            .post(url)
            .json(body)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout(self.config.timeout_secs)
                } else {
                    LlmError::HttpError(format!("Request failed: {}", e))
                }
            })?;

        if !response.status().is_success() {
            return Err(LlmError::HttpError(format!(
                "HTTP {} from Ollama",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .map_err(|e| LlmError::InvalidJson(format!("Failed to parse response: {}", e)))?;

        // Extract response from Ollama chat format
        let text = response_json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::EmptyResponse)?;

        // Parse the response as JSON
        self.parse_json_response(text)
    }

    /// Parse JSON from LLM response, handling common issues
    fn parse_json_response(&self, text: &str) -> Result<serde_json::Value, LlmError> {
        let text = text.trim();

        // Try direct parse first
        if let Ok(json) = serde_json::from_str(text) {
            return Ok(json);
        }

        // Try to extract JSON from markdown code blocks
        if let Some(start) = text.find("```json") {
            if let Some(end) = text[start + 7..].find("```") {
                let json_str = &text[start + 7..start + 7 + end].trim();
                if let Ok(json) = serde_json::from_str(json_str) {
                    return Ok(json);
                }
            }
        }

        // Try to extract JSON from generic code blocks
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            if let Some(end) = after_start.find("```") {
                // Skip the language identifier if present
                let content = &after_start[..end];
                let json_str = content.lines().skip(1).collect::<Vec<_>>().join("\n");
                if let Ok(json) = serde_json::from_str(&json_str) {
                    return Ok(json);
                }
                // Try without skipping
                if let Ok(json) = serde_json::from_str(content.trim()) {
                    return Ok(json);
                }
            }
        }

        // Try to find JSON object in the text
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                let json_str = &text[start..=end];
                if let Ok(json) = serde_json::from_str(json_str) {
                    return Ok(json);
                }
            }
        }

        Err(LlmError::InvalidJson(format!("Could not parse JSON from: {}", &text[..text.len().min(200)])))
    }
}

/// Fake LLM client for testing
pub struct FakeLlmClient {
    responses: std::sync::Mutex<Vec<Result<serde_json::Value, LlmError>>>,
    call_count: std::sync::Mutex<usize>,
}

impl FakeLlmClient {
    /// Create a fake client with pre-defined responses
    pub fn new(responses: Vec<Result<serde_json::Value, LlmError>>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
            call_count: std::sync::Mutex::new(0),
        }
    }

    /// Create a fake client that always returns valid JSON
    pub fn always_valid(json: serde_json::Value) -> Self {
        Self::new(vec![Ok(json)])
    }

    /// Create a fake client that always returns an error
    pub fn always_error(error: LlmError) -> Self {
        Self::new(vec![Err(error)])
    }

    /// Get the number of calls made
    pub fn call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

impl LlmClient for FakeLlmClient {
    fn call_json(
        &self,
        _system_prompt: &str,
        _user_prompt: &str,
        _schema_description: &str,
    ) -> Result<serde_json::Value, LlmError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            return Err(LlmError::EmptyResponse);
        }

        if responses.len() == 1 {
            responses[0].clone()
        } else {
            responses.remove(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(config.enabled);
        assert_eq!(config.endpoint, "http://localhost:11434");
        assert_eq!(config.model, "qwen2.5:14b");
    }

    #[test]
    fn test_parse_json_response() {
        let client = HttpLlmClient::new(LlmConfig::default()).unwrap();

        // Direct JSON
        let result = client.parse_json_response(r#"{"step_type": "final_answer"}"#);
        assert!(result.is_ok());

        // JSON in code block
        let result = client.parse_json_response("```json\n{\"step_type\": \"final_answer\"}\n```");
        assert!(result.is_ok());

        // JSON with surrounding text
        let result = client.parse_json_response("Here is my answer: {\"step_type\": \"final_answer\"}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_fake_client() {
        let json = serde_json::json!({"test": "data"});
        let client = FakeLlmClient::always_valid(json.clone());

        let result = client.call_json("system", "user", "schema");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json);
    }
}
