//! LLM Abstraction Layer - Tasks 12 & A
//!
//! Provides a clean abstraction for Language Model interactions.
//! Default: Not configured (prompts user to set up LLM)
//!
//! Safety guarantees:
//! - LLM output is text only, never executed
//! - Backend disabled by default
//! - No network calls unless explicitly configured
//! - API keys handled via environment variables
//! - Local LLM preferred, remote with warnings

use serde::{Deserialize, Serialize};
use std::env;

/// LLM operational mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmMode {
    /// LLM is not configured yet
    NotConfigured,

    /// Using a local LLM (Ollama, etc.) - preferred
    Local,

    /// Using a remote API (OpenAI, etc.) - with privacy/cost warnings
    Remote,

    /// Explicitly disabled by user
    Disabled,
}

impl Default for LlmMode {
    fn default() -> Self {
        Self::NotConfigured
    }
}

/// LLM backend type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LlmBackendKind {
    /// No LLM backend (default)
    Disabled,

    /// Local HTTP backend (e.g., Ollama)
    LocalHttp,

    /// Remote OpenAI-compatible API
    RemoteOpenAiCompatible,
}

impl Default for LlmBackendKind {
    fn default() -> Self {
        Self::Disabled
    }
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Operational mode
    pub mode: LlmMode,

    /// Backend type
    pub backend: LlmBackendKind,

    /// Base URL for HTTP backend
    /// Examples:
    /// - Local: "http://localhost:11434/v1" (Ollama)
    /// - Remote: "https://api.openai.com/v1"
    pub base_url: Option<String>,

    /// Environment variable name containing API key (e.g., "OPENAI_API_KEY")
    /// Not required for local backends
    pub api_key_env: Option<String>,

    /// Model name (e.g., "llama3", "gpt-4o-mini")
    pub model: Option<String>,

    /// Maximum tokens in response
    pub max_tokens: Option<u32>,

    /// Estimated cost per 1000 tokens (USD) - for remote backends
    /// Used to show approximate cost to user
    pub cost_per_1k_tokens: Option<f64>,

    /// Safety notes shown to user for this config
    /// e.g., "Using remote API may send system info to provider"
    pub safety_notes: Vec<String>,

    /// Human-readable description of this config
    pub description: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            mode: LlmMode::NotConfigured,
            backend: LlmBackendKind::Disabled,
            base_url: None,
            api_key_env: None,
            model: None,
            max_tokens: Some(500), // Conservative default
            cost_per_1k_tokens: None,
            safety_notes: Vec::new(),
            description: "LLM not configured".to_string(),
        }
    }
}

impl LlmConfig {
    /// Create a local LLM configuration (Ollama-style)
    pub fn local(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            mode: LlmMode::Local,
            backend: LlmBackendKind::LocalHttp,
            base_url: Some(base_url.into()),
            api_key_env: None,
            model: Some(model.into()),
            max_tokens: Some(2000), // Local can handle more
            cost_per_1k_tokens: None, // Local is free
            safety_notes: vec![
                "Using local LLM - your data stays on this machine".to_string(),
            ],
            description: "Local LLM (privacy-first)".to_string(),
        }
    }

    /// Create a remote LLM configuration (OpenAI-compatible)
    pub fn remote(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key_env: impl Into<String>,
        cost_per_1k: f64,
    ) -> Self {
        Self {
            mode: LlmMode::Remote,
            backend: LlmBackendKind::RemoteOpenAiCompatible,
            base_url: Some(base_url.into()),
            api_key_env: Some(api_key_env.into()),
            model: Some(model.into()),
            max_tokens: Some(1000), // Conservative for cost control
            cost_per_1k_tokens: Some(cost_per_1k),
            safety_notes: vec![
                "Using remote API - system info may be sent to provider".to_string(),
                "You may be charged per token by your provider".to_string(),
            ],
            description: "Remote API (privacy trade-off)".to_string(),
        }
    }

    /// Explicitly disable LLM
    pub fn disabled() -> Self {
        Self {
            mode: LlmMode::Disabled,
            backend: LlmBackendKind::Disabled,
            description: "LLM explicitly disabled by user".to_string(),
            ..Default::default()
        }
    }

    /// Check if LLM is usable
    pub fn is_usable(&self) -> bool {
        matches!(self.mode, LlmMode::Local | LlmMode::Remote)
    }

    /// Get estimated cost for a conversation (tokens)
    pub fn estimated_cost(&self, tokens: u32) -> Option<f64> {
        self.cost_per_1k_tokens.map(|cost_per_1k| {
            (tokens as f64 / 1000.0) * cost_per_1k
        })
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

/// HTTP OpenAI-compatible backend (supports both local and remote)
pub struct HttpOpenAiBackend {
    base_url: String,
    api_key: Option<String>,
    model: String,
    max_tokens: u32,
    is_local: bool,
}

impl HttpOpenAiBackend {
    /// Create a new HTTP backend from config
    pub fn new(config: &LlmConfig) -> Result<Self, LlmError> {
        let base_url = config.base_url.clone()
            .ok_or_else(|| LlmError::ConfigError("base_url is required for HTTP backend".to_string()))?;

        let model = config.model.clone()
            .ok_or_else(|| LlmError::ConfigError("model is required for HTTP backend".to_string()))?;

        let is_local = config.mode == LlmMode::Local;

        // Try to read API key from environment if configured
        let api_key = if let Some(env_var) = &config.api_key_env {
            match env::var(env_var) {
                Ok(key) if !key.is_empty() => Some(key),
                Ok(_) => {
                    if is_local {
                        // Local servers don't need API keys
                        None
                    } else {
                        return Err(LlmError::ConfigError(format!("API key env var {} is empty", env_var)));
                    }
                }
                Err(_) => {
                    if is_local {
                        // API key not required for local servers
                        None
                    } else {
                        return Err(LlmError::ConfigError(format!("API key env var {} not found", env_var)));
                    }
                }
            }
        } else {
            if !is_local {
                return Err(LlmError::ConfigError("API key env var required for remote backend".to_string()));
            }
            None
        };

        let max_tokens = config.max_tokens.unwrap_or(500);

        Ok(Self {
            base_url,
            api_key,
            model,
            max_tokens,
            is_local,
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
        // Check if LLM is configured and usable
        if !config.is_usable() {
            return Err(LlmError::Disabled);
        }

        let backend: Box<dyn LlmBackend> = match config.backend {
            LlmBackendKind::Disabled => {
                return Err(LlmError::Disabled);
            }
            LlmBackendKind::LocalHttp | LlmBackendKind::RemoteOpenAiCompatible => {
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

        // Creating a client from a disabled/not configured config should fail
        let result = LlmClient::from_config(&config);
        assert!(matches!(result, Err(LlmError::Disabled)));
    }

    #[test]
    fn test_http_backend_requires_base_url() {
        let mut config = LlmConfig::local("", "test-model");
        config.base_url = None;

        let result = HttpOpenAiBackend::new(&config);
        assert!(matches!(result, Err(LlmError::ConfigError(_))));
    }

    #[test]
    fn test_http_backend_requires_model() {
        let mut config = LlmConfig::local("http://localhost:8080/v1", "");
        config.model = None;

        let result = HttpOpenAiBackend::new(&config);
        assert!(matches!(result, Err(LlmError::ConfigError(_))));
    }

    #[test]
    fn test_local_config_creation() {
        let config = LlmConfig::local("http://localhost:11434/v1", "llama3");

        assert_eq!(config.mode, LlmMode::Local);
        assert_eq!(config.backend, LlmBackendKind::LocalHttp);
        assert_eq!(config.base_url, Some("http://localhost:11434/v1".to_string()));
        assert_eq!(config.model, Some("llama3".to_string()));
        assert!(config.cost_per_1k_tokens.is_none()); // Local is free
        assert!(config.is_usable());
    }

    #[test]
    fn test_remote_config_creation() {
        let config = LlmConfig::remote(
            "https://api.openai.com/v1",
            "gpt-4o-mini",
            "OPENAI_API_KEY",
            0.00015,
        );

        assert_eq!(config.mode, LlmMode::Remote);
        assert_eq!(config.backend, LlmBackendKind::RemoteOpenAiCompatible);
        assert_eq!(config.cost_per_1k_tokens, Some(0.00015));
        assert!(config.is_usable());

        // Test cost estimation
        let cost = config.estimated_cost(10000);
        assert!(cost.is_some());
        assert!((cost.unwrap() - 0.0015).abs() < 0.0001);
    }

    #[test]
    fn test_disabled_config() {
        let config = LlmConfig::disabled();

        assert_eq!(config.mode, LlmMode::Disabled);
        assert!(!config.is_usable());
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
