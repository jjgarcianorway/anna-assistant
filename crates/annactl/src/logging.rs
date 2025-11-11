//! Logging for annactl operations
//!
//! Phase 0.3d: Append-only JSONL logging to /var/log/anna/ctl.jsonl
//! Citation: [archwiki:system_maintenance]

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

const LOG_PATH: &str = "/var/log/anna/ctl.jsonl";

/// Log entry for each annactl invocation
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp
    pub ts: String,

    /// Request ID (UUID)
    pub req_id: String,

    /// System state at invocation time
    pub state: String,

    /// Command name
    pub command: String,

    /// Whether command was allowed (null if daemon unavailable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<bool>,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Exit code
    pub exit_code: i32,

    /// Wiki citation
    pub citation: String,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Success flag
    pub ok: bool,

    /// Error details if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}

impl LogEntry {
    /// Write log entry to file
    pub fn write(&self) -> Result<(), std::io::Error> {
        // Create parent directory if needed
        if let Some(parent) = std::path::Path::new(LOG_PATH).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_PATH)?;

        let json = serde_json::to_string(self)?;
        writeln!(file, "{}", json)?;

        Ok(())
    }

    /// Generate request ID
    pub fn generate_req_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Get current timestamp in ISO 8601 format
    pub fn now() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}
