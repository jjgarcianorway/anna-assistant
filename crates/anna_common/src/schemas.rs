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
