//! Audit Logger - Append-only JSONL logging for all actions

use anna_common::AuditEntry;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::info;

const AUDIT_DIR: &str = "/var/log/anna";
const AUDIT_FILE: &str = "audit.jsonl";

/// Audit logger for recording all actions
pub struct AuditLogger {
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new() -> Result<Self> {
        let audit_dir = Path::new(AUDIT_DIR);
        create_dir_all(audit_dir)
            .await
            .context("Failed to create audit log directory")?;

        let log_path = audit_dir.join(AUDIT_FILE);

        info!("Audit logger initialized: {}", log_path.display());

        Ok(Self { log_path })
    }

    /// Log an audit entry
    pub async fn log(&self, entry: &AuditEntry) -> Result<()> {
        let json = serde_json::to_string(entry)? + "\n";

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .await
            .context("Failed to open audit log")?;

        file.write_all(json.as_bytes())
            .await
            .context("Failed to write audit entry")?;

        file.sync_all().await.context("Failed to sync audit log")?;

        Ok(())
    }

    /// Read all audit entries (for debugging/reports)
    pub async fn read_all(&self) -> Result<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(vec![]);
        }

        let content = tokio::fs::read_to_string(&self.log_path)
            .await
            .context("Failed to read audit log")?;

        let entries: Vec<AuditEntry> = content
            .lines()
            .filter(|line| !line.is_empty())
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        Ok(entries)
    }

    /// Get the path to the audit log
    pub fn path(&self) -> &Path {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_audit_logging() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.jsonl");

        let logger = AuditLogger {
            log_path: log_path.clone(),
        };

        let entry = AuditEntry {
            timestamp: Utc::now(),
            actor: "test".to_string(),
            action_type: "test_action".to_string(),
            details: "Test details".to_string(),
            success: true,
        };

        logger.log(&entry).await.unwrap();

        let entries = logger.read_all().await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].actor, "test");
    }
}
