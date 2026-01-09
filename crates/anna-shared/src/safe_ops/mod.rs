//! Safe Operations - Backup, verify, and rollback system changes.
//!
//! This module provides safe file operations for Anna:
//! - Automatic backups before modifications
//! - Verification of changes
//! - Rollback capability
//!
//! All backups are stored in ~/.local/share/anna/backups/

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::anna_data_dir;

/// A backup of a file before modification
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

/// Record of all backups
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackupLedger {
    /// All backups
    pub backups: Vec<FileBackup>,
}

impl BackupLedger {
    /// Load backup ledger from disk
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

    /// Save backup ledger to disk
    pub fn save(&self) -> Result<()> {
        let path = ledger_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Add a backup to the ledger
    pub fn add_backup(&mut self, backup: FileBackup) {
        self.backups.push(backup);
    }

    /// Find backups for a specific file
    pub fn find_backups(&self, original_path: &str) -> Vec<&FileBackup> {
        self.backups
            .iter()
            .filter(|b| b.original_path == original_path && b.can_rollback)
            .collect()
    }

    /// Get the most recent backup for a file
    pub fn latest_backup(&self, original_path: &str) -> Option<&FileBackup> {
        self.find_backups(original_path)
            .into_iter()
            .max_by_key(|b| &b.created_at)
    }
}

/// Create a backup of a file before modifying it
pub fn backup_file(file_path: &str, reason: &str) -> Result<FileBackup> {
    let path = Path::new(file_path);

    // Ensure source exists
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", file_path));
    }

    // Create backup directory
    let backup_dir = backups_dir();
    fs::create_dir_all(&backup_dir)?;

    // Generate backup filename with timestamp
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let backup_name = format!("{}_{}.bak", file_name, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    // Copy file to backup location
    fs::copy(path, &backup_path)?;

    let backup = FileBackup {
        original_path: file_path.to_string(),
        backup_path: backup_path.to_string_lossy().to_string(),
        created_at: Utc::now().to_rfc3339(),
        reason: reason.to_string(),
        can_rollback: true,
    };

    // Add to ledger
    let mut ledger = BackupLedger::load().unwrap_or_default();
    ledger.add_backup(backup.clone());
    ledger.save()?;

    Ok(backup)
}

/// Rollback a file to its backup
pub fn rollback_file(backup: &FileBackup) -> Result<()> {
    let backup_path = Path::new(&backup.backup_path);
    let original_path = Path::new(&backup.original_path);

    if !backup_path.exists() {
        return Err(anyhow!("Backup file does not exist: {}", backup.backup_path));
    }

    // Create parent directory if needed
    if let Some(parent) = original_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Restore the backup
    fs::copy(backup_path, original_path)?;

    // Mark as rolled back in ledger
    let mut ledger = BackupLedger::load().unwrap_or_default();
    if let Some(entry) = ledger.backups.iter_mut().find(|b| b.backup_path == backup.backup_path) {
        entry.can_rollback = false;
    }
    ledger.save()?;

    Ok(())
}

/// Verify a file contains expected content
pub fn verify_file_contains(file_path: &str, expected: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    Ok(content.contains(expected))
}

/// Verify a file does not contain specific content
pub fn verify_file_not_contains(file_path: &str, not_expected: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    Ok(!content.contains(not_expected))
}

/// Safe write operation with backup
pub fn safe_write(file_path: &str, content: &str, reason: &str) -> Result<FileBackup> {
    // v0.0.891: Restructured to avoid unwrap
    let file_exists = Path::new(file_path).exists();

    // Write new content (create parent dirs if needed)
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Handle existing file: backup first
    if file_exists {
        let backup = backup_file(file_path, reason)?;
        fs::write(file_path, content)?;
        return Ok(backup);
    }

    // New file: write and create record (no backup to restore to)
    fs::write(file_path, content)?;
    let backup = FileBackup {
        original_path: file_path.to_string(),
        backup_path: String::new(),
        created_at: Utc::now().to_rfc3339(),
        reason: format!("Created new file: {}", reason),
        can_rollback: false,
    };
    let mut ledger = BackupLedger::load().unwrap_or_default();
    ledger.add_backup(backup.clone());
    ledger.save()?;
    Ok(backup)
}

/// Safe append operation with backup
pub fn safe_append(file_path: &str, content: &str, reason: &str) -> Result<FileBackup> {
    // Backup existing file if it exists
    let backup = if Path::new(file_path).exists() {
        backup_file(file_path, reason)?
    } else {
        // Create an empty backup record for new files
        FileBackup {
            original_path: file_path.to_string(),
            backup_path: String::new(),
            created_at: Utc::now().to_rfc3339(),
            reason: format!("Created new file: {}", reason),
            can_rollback: false,
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Append content
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    use std::io::Write;
    writeln!(file, "{}", content)?;

    Ok(backup)
}

/// Get backups directory
pub fn backups_dir() -> PathBuf {
    anna_data_dir().join("backups")
}

/// Get ledger path
fn ledger_path() -> PathBuf {
    anna_data_dir().join("backup_ledger.json")
}

/// List all recent backups (last 7 days)
pub fn list_recent_backups() -> Result<Vec<FileBackup>> {
    let ledger = BackupLedger::load()?;
    let cutoff = Utc::now() - chrono::Duration::days(7);

    Ok(ledger
        .backups
        .into_iter()
        .filter(|b| {
            b.can_rollback
                && chrono::DateTime::parse_from_rfc3339(&b.created_at)
                    .map(|dt| dt > cutoff)
                    .unwrap_or(false)
        })
        .collect())
}

/// Clean up old backups (older than 30 days)
pub fn cleanup_old_backups() -> Result<usize> {
    let mut ledger = BackupLedger::load()?;
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let mut cleaned = 0;

    for backup in &mut ledger.backups {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&backup.created_at) {
            if created < cutoff && backup.can_rollback {
                // Delete the backup file
                if let Err(e) = fs::remove_file(&backup.backup_path) {
                    tracing::warn!("Failed to delete old backup {}: {}", backup.backup_path, e);
                } else {
                    backup.can_rollback = false;
                    cleaned += 1;
                }
            }
        }
    }

    ledger.save()?;
    Ok(cleaned)
}
