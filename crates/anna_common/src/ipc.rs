//! IPC protocol definitions for Anna Assistant
//!
//! Defines message types and communication protocol between daemon and client.

use crate::types::{Advice, SystemFacts};
use serde::{Deserialize, Serialize};

/// IPC Request from client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    pub method: Method,
}

/// IPC Response from daemon to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    pub result: Result<ResponseData, String>,
}

/// Request methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Method {
    /// Get daemon status
    Status,

    /// Get system facts
    GetFacts,

    /// Get recommendations
    GetAdvice,

    /// Get recommendations with user context (for multi-user systems)
    GetAdviceWithContext {
        username: String,
        desktop_env: Option<String>,
        shell: String,
        display_server: Option<String>,
    },

    /// Apply an action by advice ID
    ApplyAction { advice_id: String, dry_run: bool },

    /// Get configuration
    GetConfig,

    /// Set configuration value
    SetConfig { key: String, value: String },

    /// Refresh system facts and advice
    Refresh,

    /// Ping daemon (health check)
    Ping,
}

/// Response data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResponseData {
    /// Status information
    Status(StatusData),

    /// System facts
    Facts(SystemFacts),

    /// List of advice
    Advice(Vec<Advice>),

    /// Action result
    ActionResult { success: bool, message: String },

    /// Configuration data
    Config(ConfigData),

    /// Simple success/pong
    Ok,
}

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusData {
    pub version: String,
    pub uptime_seconds: u64,
    pub last_telemetry_check: String,
    pub pending_recommendations: usize,
}

/// Configuration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub autonomy_tier: u8,
    pub auto_update_check: bool,
    pub wiki_cache_path: String,
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            autonomy_tier: 0, // AdviseOnly
            auto_update_check: true,
            wiki_cache_path: "~/.local/share/anna/wiki".to_string(),
        }
    }
}
