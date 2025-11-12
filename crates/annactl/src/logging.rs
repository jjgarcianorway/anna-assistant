//! Logging for annactl operations
//!
//! v1.16.3: XDG-compliant logging with fallback chain
//! Citation: [archwiki:system_maintenance][archwiki:XDG_Base_Directory]

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

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
    /// Discover log file path with fallback chain
    ///
    /// Priority:
    /// 1. $ANNACTL_LOG_FILE environment variable (explicit override)
    /// 2. $XDG_STATE_HOME/anna/ctl.jsonl (XDG standard)
    /// 3. ~/.local/state/anna/ctl.jsonl (XDG fallback)
    ///
    /// Never defaults to /var/log/anna for non-root users
    fn discover_log_path() -> Option<String> {
        // 1. Explicit override
        if let Ok(path) = std::env::var("ANNACTL_LOG_FILE") {
            return Some(path);
        }

        // 2. XDG_STATE_HOME
        if let Ok(xdg_state) = std::env::var("XDG_STATE_HOME") {
            return Some(format!("{}/anna/ctl.jsonl", xdg_state));
        }

        // 3. HOME/.local/state fallback
        if let Ok(home) = std::env::var("HOME") {
            return Some(format!("{}/.local/state/anna/ctl.jsonl", home));
        }

        None
    }

    /// Write log entry to file, falling back to stdout on failure
    pub fn write(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(self)?;

        // Try to get log path
        let log_path = Self::discover_log_path();

        if let Some(path) = log_path {
            // Try to write to file
            match Self::write_to_file(&json, &path) {
                Ok(()) => return Ok(()),
                Err(_) => {
                    // Silently fall back to stdout
                    println!("{}", json);
                    return Ok(());
                }
            }
        }

        // No path available, write to stdout
        println!("{}", json);
        Ok(())
    }

    /// Attempt to write log entry to file
    fn write_to_file(json: &str, path: &str) -> Result<(), std::io::Error> {
        // Create parent directory if needed
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

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
