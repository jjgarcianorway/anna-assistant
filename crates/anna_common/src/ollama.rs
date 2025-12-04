//! Ollama Local LLM Client v0.0.5
//!
//! HTTP client for local Ollama API - used for Translator and Junior.
//! No cloud calls - all local.
//!
//! Endpoints used:
//! - GET / - health check
//! - GET /api/tags - list available models
//! - POST /api/generate - generate response
//! - POST /api/pull - pull/download a model (streaming)
//! - DELETE /api/delete - delete a model

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::mpsc;

/// Default Ollama API endpoint
pub const OLLAMA_DEFAULT_URL: &str = "http://127.0.0.1:11434";

/// Default timeout for health checks (ms)
pub const HEALTH_CHECK_TIMEOUT_MS: u64 = 2000;

/// Default timeout for generation (ms)
pub const GENERATE_TIMEOUT_MS: u64 = 60000;

/// Ollama client for local LLM calls
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    timeout_ms: u64,
}

/// Ollama availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    /// Whether Ollama service is reachable
    pub available: bool,
    /// Base URL being used
    pub url: String,
    /// List of available models (empty if not available)
    pub models: Vec<String>,
    /// Selected model for Junior (if configured)
    pub selected_model: Option<String>,
    /// Whether selected model is downloaded
    pub model_ready: bool,
    /// Last check timestamp (epoch seconds)
    pub last_check: u64,
    /// Error message if not available
    pub error: Option<String>,
}

impl Default for OllamaStatus {
    fn default() -> Self {
        Self {
            available: false,
            url: OLLAMA_DEFAULT_URL.to_string(),
            models: Vec::new(),
            selected_model: None,
            model_ready: false,
            last_check: 0,
            error: None,
        }
    }
}

/// Model info from Ollama API
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub digest: String,
    #[serde(default)]
    pub modified_at: String,
}

/// Response from /api/tags
#[derive(Debug, Clone, Deserialize)]
pub struct TagsResponse {
    #[serde(default)]
    pub models: Vec<OllamaModel>,
}

/// Request for /api/generate
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// Generation options
#[derive(Debug, Clone, Serialize, Default)]
pub struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

/// Response from /api/generate (non-streaming)
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    #[serde(default)]
    pub done: bool,
    #[serde(default)]
    pub total_duration: u64,
    #[serde(default)]
    pub load_duration: u64,
    #[serde(default)]
    pub prompt_eval_count: u32,
    #[serde(default)]
    pub eval_count: u32,
    #[serde(default)]
    pub eval_duration: u64,
}

/// Request for /api/pull
#[derive(Debug, Clone, Serialize)]
pub struct PullRequest {
    pub name: String,
    #[serde(default)]
    pub stream: bool,
}

/// Progress from /api/pull (streaming)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PullProgress {
    pub status: String,
    #[serde(default)]
    pub digest: Option<String>,
    #[serde(default)]
    pub total: u64,
    #[serde(default)]
    pub completed: u64,
    #[serde(default)]
    pub error: Option<String>,
}

impl PullProgress {
    /// Calculate download percentage
    pub fn percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if this is a downloading status
    pub fn is_downloading(&self) -> bool {
        self.status.contains("pulling") || self.status.contains("download")
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.status == "success" || self.status.contains("verifying")
    }
}

/// Error from Ollama operations
#[derive(Debug, Clone)]
pub enum OllamaError {
    /// Service not reachable
    NotAvailable(String),
    /// Model not found
    ModelNotFound(String),
    /// Request timeout
    Timeout,
    /// HTTP error
    HttpError(String),
    /// Parse error
    ParseError(String),
    /// Generation error
    GenerateError(String),
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::NotAvailable(msg) => write!(f, "Ollama not available: {}", msg),
            OllamaError::ModelNotFound(model) => write!(f, "Model not found: {}", model),
            OllamaError::Timeout => write!(f, "Request timed out"),
            OllamaError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            OllamaError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            OllamaError::GenerateError(msg) => write!(f, "Generate error: {}", msg),
        }
    }
}

impl std::error::Error for OllamaError {}

impl OllamaClient {
    /// Create a new client with default URL
    pub fn new() -> Self {
        Self {
            base_url: OLLAMA_DEFAULT_URL.to_string(),
            timeout_ms: GENERATE_TIMEOUT_MS,
        }
    }

    /// Create a client with custom URL
    pub fn with_url(url: &str) -> Self {
        Self {
            base_url: url.to_string(),
            timeout_ms: GENERATE_TIMEOUT_MS,
        }
    }

    /// Set timeout in milliseconds
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Check if Ollama is available (health check)
    pub async fn is_available(&self) -> bool {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(HEALTH_CHECK_TIMEOUT_MS))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        match client.get(&self.base_url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, OllamaError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(HEALTH_CHECK_TIMEOUT_MS))
            .build()
            .map_err(|e| OllamaError::HttpError(e.to_string()))?;

        let url = format!("{}/api/tags", self.base_url);
        let resp = client.get(&url).send().await.map_err(|e| {
            if e.is_timeout() {
                OllamaError::Timeout
            } else if e.is_connect() {
                OllamaError::NotAvailable(e.to_string())
            } else {
                OllamaError::HttpError(e.to_string())
            }
        })?;

        if !resp.status().is_success() {
            return Err(OllamaError::HttpError(format!("Status: {}", resp.status())));
        }

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

        Ok(tags.models)
    }

    /// Check if a specific model is available
    pub async fn has_model(&self, model: &str) -> Result<bool, OllamaError> {
        let models = self.list_models().await?;
        // Model names may include :latest or other tags
        let model_base = model.split(':').next().unwrap_or(model);
        Ok(models.iter().any(|m| {
            let m_base = m.name.split(':').next().unwrap_or(&m.name);
            m_base == model_base || m.name == model
        }))
    }

    /// Generate a response (non-streaming)
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<GenerateResponse, OllamaError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| OllamaError::HttpError(e.to_string()))?;

        let request = GenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            stream: false,
            options: Some(GenerateOptions {
                temperature: Some(0.3), // Low temperature for consistent verification
                num_predict: Some(500), // Limit output length
                top_p: Some(0.9),
            }),
        };

        let url = format!("{}/api/generate", self.base_url);
        let resp = client.post(&url).json(&request).send().await.map_err(|e| {
            if e.is_timeout() {
                OllamaError::Timeout
            } else if e.is_connect() {
                OllamaError::NotAvailable(e.to_string())
            } else {
                OllamaError::HttpError(e.to_string())
            }
        })?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.as_u16() == 404 {
                return Err(OllamaError::ModelNotFound(model.to_string()));
            }
            return Err(OllamaError::GenerateError(format!(
                "Status {}: {}",
                status, body
            )));
        }

        let gen_resp: GenerateResponse = resp
            .json()
            .await
            .map_err(|e| OllamaError::ParseError(e.to_string()))?;

        Ok(gen_resp)
    }

    /// Pull a model with progress callback
    /// Returns a receiver that will receive progress updates
    pub async fn pull_model(
        &self,
        model: &str,
    ) -> Result<mpsc::Receiver<PullProgress>, OllamaError> {
        let (tx, rx) = mpsc::channel(100);
        let base_url = self.base_url.clone();
        let model = model.to_string();

        // Spawn task to handle streaming pull
        tokio::spawn(async move {
            let client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(3600)) // 1 hour timeout for large models
                .build()
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx
                        .send(PullProgress {
                            status: "error".to_string(),
                            digest: None,
                            total: 0,
                            completed: 0,
                            error: Some(e.to_string()),
                        })
                        .await;
                    return;
                }
            };

            let url = format!("{}/api/pull", base_url);
            let request = PullRequest {
                name: model.clone(),
                stream: true,
            };

            let resp = match client.post(&url).json(&request).send().await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx
                        .send(PullProgress {
                            status: "error".to_string(),
                            digest: None,
                            total: 0,
                            completed: 0,
                            error: Some(e.to_string()),
                        })
                        .await;
                    return;
                }
            };

            // Stream the response
            let mut stream = resp.bytes_stream();
            use futures_util::StreamExt;

            let mut buffer = String::new();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                            buffer.push_str(&text);
                            // Parse newline-delimited JSON
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].trim().to_string();
                                buffer = buffer[pos + 1..].to_string();

                                if !line.is_empty() {
                                    if let Ok(progress) =
                                        serde_json::from_str::<PullProgress>(&line)
                                    {
                                        if tx.send(progress).await.is_err() {
                                            return; // Receiver dropped
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(PullProgress {
                                status: "error".to_string(),
                                digest: None,
                                total: 0,
                                completed: 0,
                                error: Some(e.to_string()),
                            })
                            .await;
                        return;
                    }
                }
            }

            // Send final success if no error
            let _ = tx
                .send(PullProgress {
                    status: "success".to_string(),
                    digest: None,
                    total: 0,
                    completed: 0,
                    error: None,
                })
                .await;
        });

        Ok(rx)
    }

    /// Pull a model synchronously (blocking until complete)
    pub async fn pull_model_sync(&self, model: &str) -> Result<(), OllamaError> {
        let mut rx = self.pull_model(model).await?;

        while let Some(progress) = rx.recv().await {
            if let Some(err) = progress.error {
                return Err(OllamaError::HttpError(err));
            }
            if progress.status == "success" {
                return Ok(());
            }
        }

        Ok(())
    }

    /// Get full status including model availability
    pub async fn get_status(&self, selected_model: Option<&str>) -> OllamaStatus {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut status = OllamaStatus {
            url: self.base_url.clone(),
            last_check: now,
            selected_model: selected_model.map(|s| s.to_string()),
            ..Default::default()
        };

        // Check availability
        if !self.is_available().await {
            status.error = Some("Ollama service not reachable".to_string());
            return status;
        }

        status.available = true;

        // List models
        match self.list_models().await {
            Ok(models) => {
                status.models = models.iter().map(|m| m.name.clone()).collect();

                // Check if selected model is available
                if let Some(model) = selected_model {
                    let model_base = model.split(':').next().unwrap_or(model);
                    status.model_ready = status.models.iter().any(|m| {
                        let m_base = m.split(':').next().unwrap_or(m);
                        m_base == model_base || m == model
                    });
                    if !status.model_ready {
                        status.error = Some(format!("Model '{}' not downloaded", model));
                    }
                }
            }
            Err(e) => {
                status.error = Some(format!("Failed to list models: {}", e));
            }
        }

        status
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Select best model for Junior based on available models and hardware
/// Preference order (lightweight models first for verification tasks):
/// 1. qwen2.5:1.5b - Very fast, good for verification
/// 2. qwen2.5:3b - Good balance
/// 3. llama3.2:1b - Compact
/// 4. llama3.2:3b - Good balance
/// 5. phi3:mini - Microsoft's compact model
/// 6. gemma2:2b - Google's compact model
/// 7. mistral:7b - Fallback to larger model
/// 8. Any available model
pub fn select_junior_model(available_models: &[String]) -> Option<String> {
    let preference_order = [
        "qwen2.5:1.5b",
        "qwen2.5:3b",
        "llama3.2:1b",
        "llama3.2:3b",
        "phi3:mini",
        "gemma2:2b",
        "mistral:7b",
        "mistral",
        "llama3.2",
        "qwen2.5",
        "phi3",
        "gemma2",
    ];

    for preferred in preference_order {
        for model in available_models {
            let model_base = model.split(':').next().unwrap_or(model);
            if model == preferred || model_base == preferred || model.starts_with(preferred) {
                return Some(model.clone());
            }
        }
    }

    // Fallback: return first available model
    available_models.first().cloned()
}

/// Check if Ollama is installed on the system
pub fn is_ollama_installed() -> bool {
    std::process::Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get Ollama version if installed
pub fn get_ollama_version() -> Option<String> {
    std::process::Command::new("ollama")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_junior_model_prefers_small() {
        let models = vec![
            "mistral:7b".to_string(),
            "qwen2.5:1.5b".to_string(),
            "llama3.2:3b".to_string(),
        ];
        let selected = select_junior_model(&models);
        assert_eq!(selected, Some("qwen2.5:1.5b".to_string()));
    }

    #[test]
    fn test_select_junior_model_fallback() {
        let models = vec!["custom-model:latest".to_string()];
        let selected = select_junior_model(&models);
        assert_eq!(selected, Some("custom-model:latest".to_string()));
    }

    #[test]
    fn test_select_junior_model_empty() {
        let models: Vec<String> = vec![];
        let selected = select_junior_model(&models);
        assert_eq!(selected, None);
    }

    #[test]
    fn test_ollama_status_default() {
        let status = OllamaStatus::default();
        assert!(!status.available);
        assert!(status.models.is_empty());
        assert!(!status.model_ready);
    }
}
