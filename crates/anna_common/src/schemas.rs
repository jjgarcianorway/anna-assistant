//! JSON schemas for Anna API

use serde::{Deserialize, Serialize};

/// Request to run a probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunProbeRequest {
    pub probe_id: String,
    #[serde(default)]
    pub force_refresh: bool,
}

/// Request to set state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetStateRequest {
    pub key: String,
    pub value: serde_json::Value,
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
}

/// Request to get state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStateRequest {
    pub key: String,
}

/// Response for state operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateResponse {
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub found: bool,
}

/// Request to invalidate state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateRequest {
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub pattern: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub probes_available: usize,
    /// v0.16.5: List of available probe IDs
    #[serde(default)]
    pub probe_names: Vec<String>,
}

/// List probes response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProbesResponse {
    pub probes: Vec<ProbeInfo>,
}

/// Probe information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInfo {
    pub id: String,
    pub parser: String,
    pub cache_policy: String,
}

/// Ollama chat request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    /// How long to keep model loaded in memory after request.
    /// Use "0" to unload immediately, "5m" for 5 minutes, etc.
    /// Default is "5m" to balance responsiveness and resource usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

/// Ollama message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

/// Ollama chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatResponse {
    pub model: String,
    pub message: OllamaMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
}

/// Orchestrator request (LLM-A processing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorRequest {
    pub user_query: String,
    #[serde(default)]
    pub context: Vec<ProbeEvidence>,
}

/// Evidence from a probe for LLM context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeEvidence {
    pub probe_id: String,
    pub data: serde_json::Value,
    pub confidence: f64,
}

/// Final response to user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnaResponse {
    pub answer: String,
    pub confidence: f64,
    pub sources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// v0.9.0: Update state response for status command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStateResponse {
    /// Latest known version
    pub latest_version: Option<String>,
    /// Current update status description
    pub status: String,
    /// Last check timestamp (RFC 3339)
    pub last_check: Option<String>,
    /// Is an update download in progress?
    pub download_in_progress: bool,
    /// Downloaded bytes if in progress
    pub download_progress_bytes: Option<u64>,
    /// Total bytes if known
    pub download_total_bytes: Option<u64>,
    /// Is update ready to apply?
    pub ready_to_apply: bool,
    /// Is daemon currently busy (serving request)?
    pub daemon_busy: bool,
    /// Next retry time if update failed
    pub next_retry: Option<String>,
    /// Last failure reason if any
    pub last_failure: Option<String>,
}

impl Default for UpdateStateResponse {
    fn default() -> Self {
        Self {
            latest_version: None,
            status: "unknown".to_string(),
            last_check: None,
            download_in_progress: false,
            download_progress_bytes: None,
            download_total_bytes: None,
            ready_to_apply: false,
            daemon_busy: false,
            next_retry: None,
            last_failure: None,
        }
    }
}
