//! Installation logging
//!
//! Phase 0.8: Structured logging for installation steps
//! Citation: [archwiki:Installation_guide]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;

const INSTALL_LOG_DIR: &str = "/var/log/anna";
const INSTALL_LOG_FILE: &str = "install.jsonl";

/// Installation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallLogEntry {
    /// Timestamp (ISO 8601)
    pub ts: DateTime<Utc>,
    /// Installation step name
    pub step: String,
    /// Action taken
    pub action: String,
    /// Success status
    pub success: bool,
    /// Details or error message
    pub details: String,
    /// Arch Wiki citation
    pub citation: String,
    /// Dry-run mode
    pub dry_run: bool,
}

impl InstallLogEntry {
    /// Create new log entry
    pub fn new(
        step: String,
        action: String,
        success: bool,
        details: String,
        citation: String,
        dry_run: bool,
    ) -> Self {
        Self {
            ts: Utc::now(),
            step,
            action,
            success,
            details,
            citation,
            dry_run,
        }
    }

    /// Write log entry to install.jsonl
    pub async fn write(&self) -> Result<()> {
        let log_dir = Path::new(INSTALL_LOG_DIR);
        create_dir_all(log_dir)
            .await
            .context("Failed to create install log directory")?;

        let log_path = log_dir.join(INSTALL_LOG_FILE);
        let json = serde_json::to_string(self)? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .await
            .context("Failed to open install log")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write install log entry")?;

        file.sync_all()
            .await
            .context("Failed to sync install log")?;

        Ok(())
    }
}
