//! Steward logging
//!
//! Phase 0.9: Structured logging for lifecycle operations
//! Citation: [archwiki:System_maintenance]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;

const STEWARD_LOG_DIR: &str = "/var/log/anna";
const STEWARD_LOG_FILE: &str = "steward.jsonl";

/// Steward log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardLogEntry {
    /// Timestamp (ISO 8601)
    pub ts: DateTime<Utc>,
    /// Operation type (health, update, audit)
    pub operation: String,
    /// Success status
    pub success: bool,
    /// Details
    pub details: String,
    /// Items affected (e.g., packages updated, services restarted)
    pub items: Vec<String>,
    /// Arch Wiki citation
    pub citation: String,
}

impl StewardLogEntry {
    /// Create new log entry
    pub fn new(
        operation: String,
        success: bool,
        details: String,
        items: Vec<String>,
        citation: String,
    ) -> Self {
        Self {
            ts: Utc::now(),
            operation,
            success,
            details,
            items,
            citation,
        }
    }

    /// Write log entry to steward.jsonl
    pub async fn write(&self) -> Result<()> {
        let log_dir = Path::new(STEWARD_LOG_DIR);
        create_dir_all(log_dir)
            .await
            .context("Failed to create steward log directory")?;

        let log_path = log_dir.join(STEWARD_LOG_FILE);
        let json = serde_json::to_string(self)? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .context("Failed to open steward log")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write steward log entry")?;

        file.sync_all()
            .await
            .context("Failed to sync steward log")?;

        Ok(())
    }
}
