//! LLM Client Abstraction - v6.42.0
//!
//! Provides a generic interface for calling LLM backends with strict JSON schemas.
//! Supports both real implementations (Ollama, OpenAI-compatible) and fake clients for testing.

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
            model: "llama3.2:3b".to_string(),
            api_key: None,
            timeout_secs: 30,
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

    /// Check if endpoint is Ollama-style
    fn is_ollama_endpoint(&self) -> bool {
        self.config.endpoint.contains("11434") || self.config.endpoint.contains("ollama")
    }
}

impl LlmClient for HttpLlmClient {
    fn call_json(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        schema_description: &str,
    ) -> Result<serde_json::Value, LlmError> {
        if !self.config.enabled {
            return Err(LlmError::Disabled);
        }

        // Build the full prompt with schema
        let full_prompt = format!(
            "{}\n\nUser question: {}\n\nYou must respond with valid JSON matching this schema:\n{}",
            system_prompt, user_prompt, schema_description
        );

        // Try Ollama-style API first
        if self.is_ollama_endpoint() {
            match self.call_ollama(&full_prompt) {
                Ok(json) => return Ok(json),
                Err(e) => {
                    tracing::debug!("Ollama API failed, trying OpenAI-compatible: {}", e);
                }
            }
        }

        // Fall back to OpenAI-compatible API
        self.call_openai_compatible(system_prompt, &full_prompt)
    }
}

impl HttpLlmClient {
    /// Call Ollama-style API
    fn call_ollama(&self, prompt: &str) -> Result<serde_json::Value, LlmError> {
        let url = format!("{}/api/generate", self.config.endpoint);

        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "format": "json",
        });

        let response = self.client
            .post(&url)
            .json(&request_body)
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

        // Extract response from Ollama format
        let text = response_json
            .get("response")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::EmptyResponse)?;

        // Parse the response as JSON
        serde_json::from_str(text)
            .map_err(|e| LlmError::InvalidJson(format!("LLM output is not valid JSON: {}", e)))
    }

    /// Call OpenAI-compatible API
    fn call_openai_compatible(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, LlmError> {
        let url = format!("{}/v1/chat/completions", self.config.endpoint);

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt},
            ],
            "response_format": {"type": "json_object"},
        });

        let mut request = self.client.post(&url).json(&request_body);

        if let Some(api_key) = &self.config.api_key {
            request = request.bearer_auth(api_key);
        }

        let response = request.send().map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout(self.config.timeout_secs)
            } else {
                LlmError::HttpError(format!("Request failed: {}", e))
            }
        })?;

        if !response.status().is_success() {
            return Err(LlmError::HttpError(format!(
                "HTTP {} from OpenAI-compatible API",
                response.status()
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .map_err(|e| LlmError::InvalidJson(format!("Failed to parse response: {}", e)))?;

        // Extract content from OpenAI format
        let text = response_json
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| LlmError::EmptyResponse)?;

        // Parse the content as JSON
        serde_json::from_str(text)
            .map_err(|e| LlmError::InvalidJson(format!("LLM output is not valid JSON: {}", e)))
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
            // If no more responses, return the last one repeatedly
            return Err(LlmError::EmptyResponse);
        }

        if responses.len() == 1 {
            // Keep returning the same response
            responses[0].clone()
        } else {
            // Pop and return next response
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
        assert_eq!(config.model, "llama3.2:3b");
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_fake_client_always_valid() {
        let json = serde_json::json!({"test": "data"});
        let client = FakeLlmClient::always_valid(json.clone());

        let result = client.call_json("system", "user", "schema");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json);
        assert_eq!(client.call_count(), 1);

        // Call again, should return same response
        let result2 = client.call_json("system", "user", "schema");
        assert!(result2.is_ok());
        assert_eq!(client.call_count(), 2);
    }

    #[test]
    fn test_fake_client_always_error() {
        let client = FakeLlmClient::always_error(LlmError::Disabled);

        let result = client.call_json("system", "user", "schema");
        assert!(result.is_err());
        assert_eq!(client.call_count(), 1);
    }

    #[test]
    fn test_fake_client_multiple_responses() {
        let client = FakeLlmClient::new(vec![
            Ok(serde_json::json!({"response": 1})),
            Ok(serde_json::json!({"response": 2})),
            Err(LlmError::Timeout(30)),
        ]);

        let r1 = client.call_json("", "", "");
        assert!(r1.is_ok());
        assert_eq!(r1.unwrap()["response"], 1);

        let r2 = client.call_json("", "", "");
        assert!(r2.is_ok());
        assert_eq!(r2.unwrap()["response"], 2);

        let r3 = client.call_json("", "", "");
        assert!(r3.is_err());
        assert_eq!(client.call_count(), 3);
    }
}
