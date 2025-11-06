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
    ApplyAction {
        advice_id: String,
        dry_run: bool,
        stream: bool, // Enable live output streaming
    },

    /// Get configuration
    GetConfig,

    /// Set configuration value
    SetConfig { key: String, value: String },

    /// Refresh system facts and advice
    Refresh,

    /// Update Arch Wiki cache
    UpdateWikiCache,

    /// Ping daemon (health check)
    Ping,

    /// Check for Anna updates (delegated to daemon)
    CheckUpdate,

    /// Perform Anna update (delegated to daemon)
    /// This allows the daemon (running as root) to handle the update
    /// without requiring sudo from the user
    PerformUpdate {
        /// Skip download and just restart with current binaries
        restart_only: bool,
    },

    /// List rollbackable actions (Beta.91)
    ListRollbackable,

    /// Rollback a specific action by advice ID (Beta.91)
    RollbackAction {
        advice_id: String,
        dry_run: bool,
    },

    /// Rollback last N actions (Beta.91)
    RollbackLast {
        count: usize,
        dry_run: bool,
    },
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

    /// Streaming chunk (for live command output)
    StreamChunk {
        chunk_type: StreamChunkType,
        data: String,
    },

    /// Stream end marker
    StreamEnd {
        success: bool,
        message: String,
    },

    /// Update check result
    UpdateCheck {
        current_version: String,
        latest_version: String,
        is_update_available: bool,
        download_url: Option<String>,
        release_notes: Option<String>,
    },

    /// Update result
    UpdateResult {
        success: bool,
        message: String,
        old_version: String,
        new_version: String,
    },

    /// List of rollbackable actions (Beta.91)
    RollbackableActions(Vec<RollbackableAction>),

    /// Rollback result (Beta.91)
    RollbackResult {
        success: bool,
        message: String,
        actions_reversed: Vec<String>, // List of advice IDs that were rolled back
    },
}

/// Type of streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamChunkType {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// Status update
    Status,
}

/// Rollbackable action information (Beta.91)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackableAction {
    /// Advice ID that was applied
    pub advice_id: String,
    /// Human-readable title
    pub title: String,
    /// When it was executed (ISO 8601 timestamp)
    pub executed_at: String,
    /// Original command that was executed
    pub command: String,
    /// Generated rollback command
    pub rollback_command: Option<String>,
    /// Whether this action can be rolled back
    pub can_rollback: bool,
    /// Reason why rollback is not available (if applicable)
    pub rollback_unavailable_reason: Option<String>,
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
