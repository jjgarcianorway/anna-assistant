//! Log Scan State - v5.4.1

use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Log scan state path
pub const LOG_SCAN_STATE_PATH: &str = "/var/lib/anna/knowledge/log_scan_state.json";

/// State of the log scanner daemon task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogScanState {
    /// Last scan timestamp
    pub last_scan_at: u64,

    /// New errors found in last scan
    pub new_errors: u64,

    /// New warnings found in last scan
    pub new_warnings: u64,

    /// Scanner running state
    pub running: bool,

    /// Total scans performed
    pub total_scans: u64,

    /// Created at timestamp
    pub created_at: u64,

    /// v5.4.1: Journalctl cursor for incremental scanning
    #[serde(default)]
    pub journal_cursor: Option<String>,
}

impl Default for LogScanState {
    fn default() -> Self {
        Self::new()
    }
}

impl LogScanState {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            last_scan_at: now,
            new_errors: 0,
            new_warnings: 0,
            running: false,
            total_scans: 0,
            created_at: now,
            journal_cursor: None,
        }
    }

    /// Load from disk
    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(LOG_SCAN_STATE_PATH) {
            serde_json::from_str(&content).unwrap_or_else(|_| Self::new())
        } else {
            Self::new()
        }
    }

    /// Save to disk using atomic write
    /// v5.5.2: Uses atomic write (temp file + rename) to prevent corruption
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        crate::atomic_write::atomic_write(LOG_SCAN_STATE_PATH, &json)
    }

    /// Record a scan
    pub fn record_scan(&mut self, new_errors: u64, new_warnings: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.last_scan_at = now;
        self.new_errors = new_errors;
        self.new_warnings = new_warnings;
        self.total_scans += 1;
    }

    /// Format last scan timestamp
    pub fn format_last_scan(&self) -> String {
        chrono::DateTime::from_timestamp(self.last_scan_at as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Get scanner state as string
    pub fn state_string(&self) -> &'static str {
        if self.running {
            "running (background)"
        } else {
            "idle"
        }
    }
}
