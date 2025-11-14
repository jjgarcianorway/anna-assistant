//! LLM Abstraction Layer - Task 12
//!
//! Provides a clean abstraction for Language Model interactions.
//! Default: Disabled (no network calls)
//!
//! Safety guarantees:
//! - LLM output is text only, never executed
//! - Backend disabled by default
//! - No network calls unless explicitly configured
//! - API keys handled via environment variables

use serde::{Deserialize, Serialize};
use std::env;

/// LLM backend type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendKind {
    /// No LLM backend (default)
    Disabled,

    /// HTTP OpenAI-compatible API (e.g., local LLaMA, OpenAI, etc.)
    HttpOpenAiCompatible,
}

impl Default for LlmBackendKind {
    fn default() -> Self {
        Self::Disabled
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Backend type
    pub backend: LlmBackendKind,

    /// Base URL for HTTP backend (e.g., "http://localhost:8080/v1")
    pub base_url: Option<String>,

    /// Environment variable name containing API key (e.g., "OPENAI_API_KEY")
    pub api_key_env: Option<String>,

    /// Model name (e.g., "gpt-4.1-mini", "llama-3")
    pub model: Option<String>,

    /// Maximum tokens in response
    pub max_tokens: Option<u32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend: LlmBackendKind::Disabled,
            base_url: None,
            api_key_env: None,
            model: None,
            max_tokens: Some(500), // Conservative default
        }
    }
}

/// LLM prompt structure
#[derive(Debug, Clone)]
pub struct LlmPrompt {
    /// System message (Anna's role and constraints)
    pub system: String,

    /// User's question/input
    pub user: String,
}

/// LLM response
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Generated text response
    pub text: String,
}

/// LLM errors
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("LLM backend is disabled")]
    Disabled,

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// LLM backend trait
pub trait LlmBackend: Send + Sync {
    /// Send a chat request and get a response
    fn chat(&self, prompt: &LlmPrompt) -> Result<LlmResponse, LlmError>;
}

/// Dummy backend (always returns disabled error)
pub struct DummyBackend;

impl LlmBackend for DummyBackend {
    fn chat(&self, _prompt: &LlmPrompt) -> Result<LlmResponse, LlmError> {
        Err(LlmError::Disabled)
    }
}

/// HTTP OpenAI-compatible backend
pub struct HttpOpenAiBackend {
    base_url: String,
    api_key: Option<String>,
    model: String,
    max_tokens: u32,
}

impl HttpOpenAiBackend {
    /// Create a new HTTP OpenAI backend
    pub fn new(config: &LlmConfig) -> Result<Self, LlmError> {
        let base_url = config.base_url.clone()
            .ok_or_else(|| LlmError::ConfigError("base_url is required for HTTP backend".to_string()))?;

        let model = config.model.clone()
            .ok_or_else(|| LlmError::ConfigError("model is required for HTTP backend".to_string()))?;

        // Try to read API key from environment if configured
        let api_key = if let Some(env_var) = &config.api_key_env {
            match env::var(env_var) {
                Ok(key) if !key.is_empty() => Some(key),
                Ok(_) => return Err(LlmError::ConfigError(format!("API key env var {} is empty", env_var))),
                Err(_) => {
                    // API key not required for local servers, optional
                    None
                }
            }
        } else {
            None
        };

        let max_tokens = config.max_tokens.unwrap_or(500);

        Ok(Self {
            base_url,
            api_key,
            model,
            max_tokens,
        })
    }
}

impl LlmBackend for HttpOpenAiBackend {
    fn chat(&self, prompt: &LlmPrompt) -> Result<LlmResponse, LlmError> {
        // Build the request
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let mut request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": prompt.system
                },
                {
                    "role": "user",
                    "content": prompt.user
                }
            ],
            "max_tokens": self.max_tokens,
            "temperature": 0.7
        });

        // Use blocking reqwest for simplicity (Task 12 is plumbing, not production-ready)
        let client = reqwest::blocking::Client::new();
        let mut req = client.post(&url)
            .header("Content-Type", "application/json");

        // Add Authorization header if API key is present
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }

        let response = req
            .json(&request_body)
            .send()
            .map_err(|e| LlmError::HttpError(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_else(|_| "".to_string());
            return Err(LlmError::HttpError(format!("HTTP {}: {}", status, body)));
        }

        // Parse response
        let response_json: serde_json::Value = response.json()
            .map_err(|e| LlmError::HttpError(format!("Failed to parse response: {}", e)))?;

        // Extract text from first choice
        let text = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| LlmError::Unexpected("No content in response".to_string()))?
            .to_string();

        Ok(LlmResponse { text })
    }
}

/// LLM client (high-level interface)
pub struct LlmClient {
    backend: Box<dyn LlmBackend>,
}

impl LlmClient {
    /// Create LLM client from configuration
    pub fn from_config(config: &LlmConfig) -> Result<Self, LlmError> {
        let backend: Box<dyn LlmBackend> = match config.backend {
            LlmBackendKind::Disabled => Box::new(DummyBackend),
            LlmBackendKind::HttpOpenAiCompatible => {
                Box::new(HttpOpenAiBackend::new(config)?)
            }
        };

        Ok(Self { backend })
    }

    /// Send a chat request
    pub fn chat(&self, prompt: &LlmPrompt) -> Result<LlmResponse, LlmError> {
        self.backend.chat(prompt)
    }

    /// Get Anna's system prompt
    pub fn anna_system_prompt() -> String {
        "You are Anna, a local system administrator assistant for Arch Linux.\n\n\
         Your role:\n\
         - You help users understand and maintain their Arch Linux system\n\
         - You explain technical concepts in plain English\n\
         - You suggest commands but never claim to execute them\n\
         - You cite the Arch Wiki when relevant\n\n\
         Constraints:\n\
         - Only answer questions about Linux, system administration, and Arch Linux\n\
         - Do not answer off-topic questions (weather, sports, entertainment, etc.)\n\
         - Do not pretend you can run commands - you can only suggest them\n\
         - Be concise and helpful\n\
         - If you don't know something, say so and suggest checking the Arch Wiki".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_disabled() {
        let config = LlmConfig::default();
        assert_eq!(config.backend, LlmBackendKind::Disabled);
    }

    #[test]
    fn test_dummy_backend_returns_disabled() {
        let backend = DummyBackend;
        let prompt = LlmPrompt {
            system: "test".to_string(),
            user: "hello".to_string(),
        };

        let result = backend.chat(&prompt);
        assert!(matches!(result, Err(LlmError::Disabled)));
    }

    #[test]
    fn test_client_from_disabled_config() {
        let config = LlmConfig::default();
        let client = LlmClient::from_config(&config).unwrap();

        let prompt = LlmPrompt {
            system: "test".to_string(),
            user: "hello".to_string(),
        };

        let result = client.chat(&prompt);
        assert!(matches!(result, Err(LlmError::Disabled)));
    }

    #[test]
    fn test_http_backend_requires_base_url() {
        let config = LlmConfig {
            backend: LlmBackendKind::HttpOpenAiCompatible,
            base_url: None,
            api_key_env: None,
            model: Some("test-model".to_string()),
            max_tokens: Some(100),
        };

        let result = HttpOpenAiBackend::new(&config);
        assert!(matches!(result, Err(LlmError::ConfigError(_))));
    }

    #[test]
    fn test_http_backend_requires_model() {
        let config = LlmConfig {
            backend: LlmBackendKind::HttpOpenAiCompatible,
            base_url: Some("http://localhost:8080/v1".to_string()),
            api_key_env: None,
            model: None,
            max_tokens: Some(100),
        };

        let result = HttpOpenAiBackend::new(&config);
        assert!(matches!(result, Err(LlmError::ConfigError(_))));
    }

    #[test]
    fn test_anna_system_prompt_contains_key_concepts() {
        let prompt = LlmClient::anna_system_prompt();

        assert!(prompt.contains("Anna"));
        assert!(prompt.contains("Arch Linux"));
        assert!(prompt.contains("never claim to execute"));
        assert!(prompt.contains("Arch Wiki"));
    }
}
