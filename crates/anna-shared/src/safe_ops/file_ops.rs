//! Safe file operations with backup support.

use anyhow::{anyhow, Result};
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::warn;

use super::backup_types::{BackupLedger, FileBackup};
use super::backup_utils::backup_dir;

/// Create a backup of a file before modifying it.
pub fn backup_single_file(file_path: &str, reason: &str) -> Result<FileBackup> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", file_path));
    }

    let backups = backup_dir();
    fs::create_dir_all(&backups)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let backup_name = format!("{}_{}.bak", file_name, timestamp);
    let backup_path = backups.join(&backup_name);

    fs::copy(path, &backup_path)?;

    let backup = FileBackup {
        original_path: file_path.to_string(),
        backup_path: backup_path.to_string_lossy().to_string(),
        created_at: Utc::now().to_rfc3339(),
        reason: reason.to_string(),
        can_rollback: true,
    };

    let mut ledger = BackupLedger::load().unwrap_or_default();
    ledger.add_backup(backup.clone());
    ledger.save()?;

    Ok(backup)
}

/// Rollback a file to its backup.
pub fn rollback_file(backup: &FileBackup) -> Result<()> {
    let backup_path = Path::new(&backup.backup_path);
    let original_path = Path::new(&backup.original_path);

    if !backup_path.exists() {
        return Err(anyhow!("Backup file does not exist: {}", backup.backup_path));
    }

    if let Some(parent) = original_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(backup_path, original_path)?;

    let mut ledger = BackupLedger::load().unwrap_or_default();
    if let Some(entry) = ledger.backups.iter_mut().find(|b| b.backup_path == backup.backup_path) {
        entry.can_rollback = false;
    }
    ledger.save()?;

    Ok(())
}

/// Safe write operation with backup.
pub fn safe_write(file_path: &str, content: &str, reason: &str) -> Result<FileBackup> {
    let file_exists = Path::new(file_path).exists();

    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    if file_exists {
        let backup = backup_single_file(file_path, reason)?;
        fs::write(file_path, content)?;
        return Ok(backup);
    }

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

/// Safe append operation with backup.
pub fn safe_append(file_path: &str, content: &str, reason: &str) -> Result<FileBackup> {
    let backup = if Path::new(file_path).exists() {
        backup_single_file(file_path, reason)?
    } else {
        FileBackup {
            original_path: file_path.to_string(),
            backup_path: String::new(),
            created_at: Utc::now().to_rfc3339(),
            reason: format!("Created new file: {}", reason),
            can_rollback: false,
        }
    };

    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    writeln!(file, "{}", content)?;

    Ok(backup)
}

/// Verify a file contains expected content.
pub fn verify_file_contains(file_path: &str, expected: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    Ok(content.contains(expected))
}

/// Verify a file does not contain specific content.
pub fn verify_file_not_contains(file_path: &str, not_expected: &str) -> Result<bool> {
    let content = fs::read_to_string(file_path)?;
    Ok(!content.contains(not_expected))
}

/// List all recent backups (last 7 days).
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

/// Clean up old backups (older than 30 days).
pub fn cleanup_old_backups() -> Result<usize> {
    let mut ledger = BackupLedger::load()?;
    let cutoff = Utc::now() - chrono::Duration::days(30);
    let mut cleaned = 0;

    for backup in &mut ledger.backups {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&backup.created_at) {
            if created < cutoff && backup.can_rollback {
                if let Err(e) = fs::remove_file(&backup.backup_path) {
                    warn!("Failed to delete old backup {}: {}", backup.backup_path, e);
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
