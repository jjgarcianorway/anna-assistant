//! Backup type definitions.

use crate::config::anna_data_dir;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Information about a backup directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    /// Backup directory name
    pub name: String,
    /// Full path to backup
    pub path: String,
    /// Number of files in backup
    pub file_count: usize,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Creation timestamp
    pub created: Option<String>,
}

/// A backup of a file before modification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBackup {
    /// Original file path
    pub original_path: String,
    /// Backup file path
    pub backup_path: String,
    /// When the backup was created
    pub created_at: String,
    /// Description of why the backup was made
    pub reason: String,
    /// Whether this backup can be rolled back
    pub can_rollback: bool,
}

/// Record of all backups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupLedger {
    /// All backups
    pub backups: Vec<FileBackup>,
}

impl BackupLedger {
    /// Load backup ledger from disk.
    pub fn load() -> Result<Self> {
        let path = ledger_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let ledger: BackupLedger = serde_json::from_str(&content)?;
            Ok(ledger)
        } else {
            Ok(BackupLedger::default())
        }
    }

    /// Save backup ledger to disk.
    pub fn save(&self) -> Result<()> {
        let path = ledger_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Add a backup to the ledger.
    pub fn add_backup(&mut self, backup: FileBackup) {
        self.backups.push(backup);
    }

    /// Find backups for a specific file.
    pub fn find_backups(&self, original_path: &str) -> Vec<&FileBackup> {
        self.backups
            .iter()
            .filter(|b| b.original_path == original_path && b.can_rollback)
            .collect()
    }

    /// Get the most recent backup for a file.
    pub fn latest_backup(&self, original_path: &str) -> Option<&FileBackup> {
        self.find_backups(original_path)
            .into_iter()
            .max_by_key(|b| &b.created_at)
    }
}

/// Get ledger path.
pub fn ledger_path() -> PathBuf {
    anna_data_dir().join("backup_ledger.json")
}
