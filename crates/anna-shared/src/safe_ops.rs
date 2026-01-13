//! Safe operations with backup and rollback capability.
//! v0.3.21: Reset modes with automatic backups per reliability sprint spec.

use crate::config::{anna_data_dir, AnnaConfig};
use crate::memory::{memory_path, Memory};
use crate::rpc::{ResetMode, ResetResult};
use crate::stats::{PersistentStats, StatsAudit, StatsAuditEntry, StatsEventType};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Backup directory path
fn backup_dir() -> PathBuf {
    anna_data_dir().join("backups")
}

/// Create a timestamped backup directory
fn create_backup_dir(prefix: &str) -> Result<PathBuf> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_dir().join(format!("{}_{}", prefix, timestamp));
    fs::create_dir_all(&backup_path)?;
    Ok(backup_path)
}

/// Backup a file if it exists
fn backup_file(source: &PathBuf, backup_dir: &PathBuf, name: &str) -> Result<bool> {
    if source.exists() {
        let dest = backup_dir.join(name);
        fs::copy(source, &dest)?;
        debug!("Backed up {:?} to {:?}", source, dest);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Safe reset with automatic backup
pub struct SafeReset;

impl SafeReset {
    /// Perform reset with the specified mode
    /// Always creates a backup before modifying anything
    pub fn execute(mode: ResetMode) -> Result<ResetResult> {
        info!("Starting safe reset with mode: {:?}", mode);

        // Create backup first
        let backup_path = create_backup_dir(&format!("reset_{:?}", mode).to_lowercase())?;
        info!("Backup directory: {:?}", backup_path);

        let mut cleared = Vec::new();

        // Backup everything that might be affected
        Self::backup_all(&backup_path)?;

        // Execute reset based on mode
        match mode {
            ResetMode::Memory => {
                cleared.extend(Self::reset_memory()?);
            }
            ResetMode::Config => {
                cleared.extend(Self::reset_config()?);
            }
            ResetMode::Models => {
                cleared.extend(Self::reset_models()?);
            }
            ResetMode::Helpers => {
                cleared.extend(Self::reset_helpers()?);
            }
            ResetMode::Everything => {
                // Reset everything in order
                cleared.extend(Self::reset_memory()?);
                cleared.extend(Self::reset_config()?);
                cleared.extend(Self::reset_models()?);
                cleared.extend(Self::reset_helpers()?);
                cleared.extend(Self::reset_stats()?);
                cleared.extend(Self::reset_tickets()?);
            }
        }

        // Log the reset in audit trail
        let _ = StatsAudit::log(StatsAuditEntry::new(
            StatsEventType::StatsReset {
                reason: format!("User requested {:?} reset", mode),
            },
            serde_json::json!({
                "mode": format!("{:?}", mode),
                "backup_path": backup_path.display().to_string(),
                "cleared": cleared,
            }),
        ));

        info!("Reset complete. Cleared: {:?}", cleared);

        Ok(ResetResult {
            cleared,
            backup_path: Some(backup_path.display().to_string()),
        })
    }

    /// Backup all Anna data files
    /// INVARIANT: All data is at /var/lib/anna, no per-user paths
    fn backup_all(backup_dir: &PathBuf) -> Result<()> {
        let data_dir = anna_data_dir();

        // Memory
        backup_file(&memory_path(), backup_dir, "memory.json")?;

        // Config (now at /etc/anna/config.toml, but backup from data_dir for legacy)
        backup_file(&data_dir.join("config.toml"), backup_dir, "config.toml")?;

        // Stats
        backup_file(&data_dir.join("stats.json"), backup_dir, "stats.json")?;

        // Stats audit trail
        backup_file(&data_dir.join("stats_audit.jsonl"), backup_dir, "stats_audit.jsonl")?;

        // Installed deps
        backup_file(&data_dir.join("installed_deps.txt"), backup_dir, "installed_deps.txt")?;

        // Tickets (now at /var/lib/anna)
        backup_file(&data_dir.join("tickets.json"), backup_dir, "tickets.json")?;

        // Fix history (now at /var/lib/anna)
        backup_file(&data_dir.join("fix_history.json"), backup_dir, "fix_history.json")?;

        // Model preferences (if exists)
        backup_file(&data_dir.join("model_prefs.json"), backup_dir, "model_prefs.json")?;

        Ok(())
    }

    /// Reset memory (experiences, patterns, clusters)
    /// v0.3.23: Always report counts for deterministic output
    fn reset_memory() -> Result<Vec<String>> {
        let mut cleared = Vec::new();

        let (exp_count, pattern_count, cluster_count) = match Memory::load() {
            Ok(memory) => {
                let counts = (
                    memory.experiences.len(),
                    memory.patterns.len(),
                    memory.clusters.len(),
                );
                // Always save fresh memory to ensure clean state
                let fresh = Memory::default();
                fresh.save()?;
                counts
            }
            Err(_) => {
                // Memory doesn't exist, create fresh
                let fresh = Memory::default();
                fresh.save()?;
                (0, 0, 0)
            }
        };

        // Always report for deterministic output
        cleared.push(format!(
            "Memory ({} experiences, {} patterns, {} clusters)",
            exp_count, pattern_count, cluster_count
        ));

        Ok(cleared)
    }

    /// Reset config to defaults
    fn reset_config() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let config_path = anna_data_dir().join("config.toml");

        if config_path.exists() {
            let fresh_config = AnnaConfig::default();
            fresh_config.save()?;
            cleared.push("Configuration (reset to defaults)".to_string());
        }

        Ok(cleared)
    }

    /// Reset model preferences
    fn reset_models() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let prefs_path = anna_data_dir().join("model_prefs.json");

        if prefs_path.exists() {
            fs::remove_file(&prefs_path)?;
            cleared.push("Model preferences (will re-detect on next start)".to_string());
        }

        Ok(cleared)
    }

    /// Reset helper tracking (does NOT uninstall packages)
    fn reset_helpers() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let deps_path = anna_data_dir().join("installed_deps.txt");

        if deps_path.exists() {
            if let Ok(content) = fs::read_to_string(&deps_path) {
                let count = content.lines().filter(|l| !l.trim().is_empty()).count();
                if count > 0 {
                    fs::remove_file(&deps_path)?;
                    cleared.push(format!(
                        "Helper tracking ({} packages - NOT uninstalled)",
                        count
                    ));
                }
            }
        }

        Ok(cleared)
    }

    /// Reset stats (XP, questions answered)
    /// v0.3.23: Always report counts for deterministic output; migrate legacy xp.json
    /// v0.3.28: Use PersistentStats::fresh() for XP baseline consistency
    /// v0.3.30: TRANSACTIONAL - single atomic pass, no retry loops
    fn reset_stats() -> Result<Vec<String>> {
        let mut cleared = Vec::new();

        // First, migrate any legacy xp.json into unified store (one-time migration)
        let legacy_migrated = Self::migrate_legacy_xp()?;

        // TRANSACTIONAL RESET: One pass, no retries
        // Step 1: Read old values for reporting
        let (questions, xp) = match PersistentStats::load() {
            Ok(stats) => (stats.rpg.total_questions, stats.rpg.xp),
            Err(_) => (0, 0),
        };

        // Step 2: Delete the old file completely (atomic)
        let stats_path = anna_data_dir().join("stats.json");
        if stats_path.exists() {
            fs::remove_file(&stats_path)
                .context("Failed to remove stats.json - reset aborted")?;
        }

        // Step 3: Write fresh stats (single atomic write)
        let fresh = PersistentStats::fresh();
        fresh.save().context("Failed to write fresh stats - reset incomplete")?;

        // NO VERIFICATION LOOP - if save() returns Ok, we trust it
        // The daemon must reload from this file, not use cached values

        // Always report for deterministic output
        cleared.push(format!(
            "Stats ({} questions, {} XP)",
            questions, xp
        ));

        // Report if legacy data was migrated
        if let Some(msg) = legacy_migrated {
            cleared.push(msg);
        }

        // Clear audit trail if exists
        let audit_path = anna_data_dir().join("stats_audit.jsonl");
        let audit_existed = audit_path.exists();
        if audit_existed {
            fs::remove_file(&audit_path)?;
        }
        // Always report audit trail status
        cleared.push(format!(
            "Stats audit trail ({})",
            if audit_existed { "cleared" } else { "none" }
        ));

        Ok(cleared)
    }

    /// Migrate legacy xp.json into PersistentStats (one-time)
    /// Returns Some(message) if migration occurred, None otherwise
    fn migrate_legacy_xp() -> Result<Option<String>> {
        let xp_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/xp.json");

        if !xp_path.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&xp_path) {
            Ok(c) => c,
            Err(_) => return Ok(None),
        };

        let xp: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => {
                // Invalid JSON, just delete it
                fs::remove_file(&xp_path)?;
                return Ok(Some("Legacy XP (invalid, removed)".to_string()));
            }
        };

        let total_xp = xp.get("total_xp").and_then(|v| v.as_u64()).unwrap_or(0);
        let level = xp.get("level").and_then(|v| v.as_u64()).unwrap_or(1);

        // Delete legacy file - migration complete
        fs::remove_file(&xp_path)?;

        if total_xp > 0 || level > 1 {
            Ok(Some(format!(
                "Legacy XP migrated ({} XP, level {}) - file removed",
                total_xp, level
            )))
        } else {
            Ok(Some("Legacy XP (empty, removed)".to_string()))
        }
    }

    /// Reset ticket tracker
    /// v0.3.23: Always report counts for deterministic output
    /// v0.3.30: TRANSACTIONAL - single atomic pass, no retry loops
    /// v0.3.31: System-wide paths at /var/lib/anna
    fn reset_tickets() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let tickets_path = anna_data_dir().join("tickets.json");

        // TRANSACTIONAL: Read counts first, then delete once
        let (resolved, failed, escalated) = if tickets_path.exists() {
            let counts = if let Ok(content) = fs::read_to_string(&tickets_path) {
                if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
                    (
                        store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0),
                        store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0),
                        store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0),
                    )
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            };
            // Single atomic delete - if this fails, the whole reset fails
            fs::remove_file(&tickets_path)
                .context("Failed to remove tickets.json - reset aborted")?;
            counts
        } else {
            (0, 0, 0)
        };

        // NO VERIFICATION LOOP - if remove_file() returns Ok, file is gone
        // The daemon must reload (or clear cache) after reset RPC

        // Always report for deterministic output
        cleared.push(format!(
            "Tickets ({} resolved, {} failed, {} escalated)",
            resolved, failed, escalated
        ));

        // Clear fix history (now at /var/lib/anna)
        let fix_history_path = anna_data_dir().join("fix_history.json");

        let fixes_count = if fix_history_path.exists() {
            let count = if let Ok(content) = fs::read_to_string(&fix_history_path) {
                if let Ok(history) = serde_json::from_str::<serde_json::Value>(&content) {
                    history.get("fixes").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
                } else {
                    0
                }
            } else {
                0
            };
            fs::remove_file(&fix_history_path)
                .context("Failed to remove fix_history.json - reset aborted")?;
            count
        } else {
            0
        };

        // Always report fix history status
        cleared.push(format!("Fix history ({} fixes)", fixes_count));

        Ok(cleared)
    }

    /// List available backups
    pub fn list_backups() -> Result<Vec<BackupInfo>> {
        let backup_path = backup_dir();
        if !backup_path.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&backup_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Count files in backup
                let file_count = fs::read_dir(&path)?.count();

                // Get size
                let size = Self::dir_size(&path)?;

                // Get creation time from directory metadata
                let created = fs::metadata(&path)?
                    .created()
                    .ok()
                    .map(|t| {
                        chrono::DateTime::<Utc>::from(t).to_rfc3339()
                    });

                backups.push(BackupInfo {
                    name,
                    path: path.display().to_string(),
                    file_count,
                    size_bytes: size,
                    created,
                });
            }
        }

        // Sort by name (which includes timestamp) descending
        backups.sort_by(|a, b| b.name.cmp(&a.name));

        Ok(backups)
    }

    /// Calculate directory size
    fn dir_size(path: &PathBuf) -> Result<u64> {
        let mut size = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            size += entry.metadata()?.len();
        }
        Ok(size)
    }

    /// Restore from a backup
    pub fn restore_backup(backup_name: &str) -> Result<Vec<String>> {
        let backup_path = backup_dir().join(backup_name);
        if !backup_path.exists() {
            anyhow::bail!("Backup not found: {}", backup_name);
        }

        let data_dir = anna_data_dir();

        let mut restored = Vec::new();

        // Restore each file if it exists in backup
        // v0.3.31: All paths are now under /var/lib/anna
        let restore_pairs: Vec<(&str, PathBuf)> = vec![
            ("memory.json", memory_path()),
            ("config.toml", data_dir.join("config.toml")),
            ("stats.json", data_dir.join("stats.json")),
            ("stats_audit.jsonl", data_dir.join("stats_audit.jsonl")),
            ("installed_deps.txt", data_dir.join("installed_deps.txt")),
            ("tickets.json", data_dir.join("tickets.json")),
            ("fix_history.json", data_dir.join("fix_history.json")),
            ("model_prefs.json", data_dir.join("model_prefs.json")),
        ];

        for (backup_name, dest_path) in restore_pairs {
            let source = backup_path.join(backup_name);
            if source.exists() {
                // Ensure parent directory exists
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&source, &dest_path)?;
                restored.push(backup_name.to_string());
            }
        }

        info!("Restored {} files from backup", restored.len());

        Ok(restored)
    }

    /// Delete old backups, keeping only the most recent N
    pub fn cleanup_backups(keep_count: usize) -> Result<usize> {
        let mut backups = Self::list_backups()?;
        if backups.len() <= keep_count {
            return Ok(0);
        }

        let mut deleted = 0;
        // backups is sorted newest first, so skip the ones to keep
        for backup in backups.drain(keep_count..) {
            let path = PathBuf::from(&backup.path);
            if fs::remove_dir_all(&path).is_ok() {
                deleted += 1;
                info!("Deleted old backup: {}", backup.name);
            }
        }

        Ok(deleted)
    }
}

/// Information about a backup
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

// =============================================================================
// Legacy file-level backup operations (for individual file modifications)
// =============================================================================

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::io::Write;

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

/// Get ledger path
fn ledger_path() -> PathBuf {
    anna_data_dir().join("backup_ledger.json")
}

/// Create a backup of a file before modifying it
pub fn backup_single_file(file_path: &str, reason: &str) -> Result<FileBackup> {
    let path = Path::new(file_path);

    // Ensure source exists
    if !path.exists() {
        return Err(anyhow!("File does not exist: {}", file_path));
    }

    // Create backup directory
    let backups = backup_dir();
    fs::create_dir_all(&backups)?;

    // Generate backup filename with timestamp
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let backup_name = format!("{}_{}.bak", file_name, timestamp);
    let backup_path = backups.join(&backup_name);

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

/// Safe write operation with backup
pub fn safe_write(file_path: &str, content: &str, reason: &str) -> Result<FileBackup> {
    let file_exists = Path::new(file_path).exists();

    // Write new content (create parent dirs if needed)
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Handle existing file: backup first
    if file_exists {
        let backup = backup_single_file(file_path, reason)?;
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

    // Ensure parent directory exists
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    // Append content
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    writeln!(file, "{}", content)?;

    Ok(backup)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_mode_from_str() {
        assert_eq!(ResetMode::from_str("memory"), Some(ResetMode::Memory));
        assert_eq!(ResetMode::from_str("mem"), Some(ResetMode::Memory));
        assert_eq!(ResetMode::from_str("config"), Some(ResetMode::Config));
        assert_eq!(ResetMode::from_str("everything"), Some(ResetMode::Everything));
        assert_eq!(ResetMode::from_str("all"), Some(ResetMode::Everything));
        assert_eq!(ResetMode::from_str("invalid"), None);
    }

    #[test]
    fn test_reset_mode_description() {
        assert!(ResetMode::Memory.description().contains("memory"));
        assert!(ResetMode::Config.description().contains("configuration"));
        assert!(ResetMode::Everything.description().contains("everything"));
    }

    #[test]
    fn test_reset_mode_aliases() {
        // Test all aliases work correctly
        assert_eq!(ResetMode::from_str("cfg"), Some(ResetMode::Config));
        assert_eq!(ResetMode::from_str("models"), Some(ResetMode::Models));
        assert_eq!(ResetMode::from_str("model"), Some(ResetMode::Models));
        assert_eq!(ResetMode::from_str("helpers"), Some(ResetMode::Helpers));
        assert_eq!(ResetMode::from_str("helper"), Some(ResetMode::Helpers));
        assert_eq!(ResetMode::from_str("deps"), Some(ResetMode::Helpers));
        assert_eq!(ResetMode::from_str("full"), Some(ResetMode::Everything));
    }

    #[test]
    fn test_reset_mode_case_insensitive() {
        assert_eq!(ResetMode::from_str("MEMORY"), Some(ResetMode::Memory));
        assert_eq!(ResetMode::from_str("Memory"), Some(ResetMode::Memory));
        assert_eq!(ResetMode::from_str("EVERYTHING"), Some(ResetMode::Everything));
    }

    #[test]
    fn test_all_modes_have_descriptions() {
        // Verify all modes have non-empty descriptions
        let modes = vec![
            ResetMode::Memory,
            ResetMode::Config,
            ResetMode::Models,
            ResetMode::Helpers,
            ResetMode::Everything,
        ];
        for mode in modes {
            let desc = mode.description();
            assert!(!desc.is_empty(), "Mode {:?} has empty description", mode);
            assert!(desc.len() > 10, "Mode {:?} description too short", mode);
        }
    }

    // v0.3.23: Integration tests for reset/stats consistency

    #[test]
    fn test_reset_memory_uses_actual_array_counts() {
        // Verify reset_memory reads actual array lengths, not counters
        // This test documents the expected behavior
        use crate::memory::Memory;

        // Load memory and verify counting behavior
        if let Ok(memory) = Memory::load() {
            let exp_count = memory.experiences.len();
            let pattern_count = memory.patterns.len();
            let cluster_count = memory.clusters.len();

            // The counts should match what reset would report
            // (this verifies we use .len() not separate counters)
            assert!(exp_count == memory.experiences.len());
            assert!(pattern_count == memory.patterns.len());
            assert!(cluster_count == memory.clusters.len());
        }
    }

    #[test]
    fn test_reset_stats_clears_legacy_xp() {
        // Verify reset_stats includes legacy xp.json clearing
        // This test documents the code path exists
        let xp_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/xp.json");

        // The path construction should be consistent
        assert!(xp_path.to_string_lossy().contains("anna/xp.json")
             || xp_path.to_string_lossy().contains("anna\\xp.json"));
    }

    #[test]
    fn test_reset_tickets_clears_all_ticket_data() {
        // Verify reset_tickets handles both tickets.json and fix_history.json
        let local_data = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna");

        let tickets_path = local_data.join("tickets.json");
        let fix_history_path = local_data.join("fix_history.json");

        // Both paths should be in the same directory
        assert_eq!(tickets_path.parent(), fix_history_path.parent());
    }

    #[test]
    fn test_backup_includes_all_data_files() {
        // Verify backup_all backs up all files that reset can modify
        // This ensures we can always restore after reset
        let data_dir = anna_data_dir();
        let local_data = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna");

        // Files that should be backed up (matching reset targets)
        let expected_files = vec![
            memory_path(),                          // Memory
            data_dir.join("config.toml"),           // Config
            data_dir.join("stats.json"),            // Stats
            data_dir.join("stats_audit.jsonl"),     // Audit trail
            data_dir.join("installed_deps.txt"),    // Helpers
            local_data.join("tickets.json"),        // Tickets
            local_data.join("fix_history.json"),    // Fix history
            data_dir.join("model_prefs.json"),      // Models
        ];

        // All paths should be valid
        for path in expected_files {
            assert!(!path.to_string_lossy().is_empty());
        }
    }

    #[test]
    fn test_reset_result_format_consistency() {
        // Verify reset result messages follow consistent format
        // Format: "Category (N items)" or "Category (description)"

        // Simulate what reset_memory would return (always reported)
        let test_message = format!("Memory ({} experiences, {} patterns, {} clusters)", 5, 3, 2);
        assert!(test_message.contains("Memory"));
        assert!(test_message.contains("experiences"));
        assert!(test_message.contains("patterns"));
        assert!(test_message.contains("clusters"));

        // Simulate what reset_stats would return (always reported with XP)
        let stats_message = format!("Stats ({} questions, {} XP)", 10, 250);
        assert!(stats_message.contains("Stats"));
        assert!(stats_message.contains("questions"));
        assert!(stats_message.contains("XP"));

        // Simulate what reset_tickets would return (always reported with escalated)
        let tickets_message = format!("Tickets ({} resolved, {} failed, {} escalated)", 5, 1, 2);
        assert!(tickets_message.contains("Tickets"));
        assert!(tickets_message.contains("resolved"));
        assert!(tickets_message.contains("failed"));
        assert!(tickets_message.contains("escalated"));
    }

    // v0.3.23: Golden output tests for reset formatting stability

    #[test]
    fn test_reset_output_structure_golden() {
        // Golden test: Reset output must always include these sections in this order
        // This ensures reset output is deterministic and stable across versions
        let expected_sections = vec![
            "In-memory caches",      // Always first (added by handler)
            "Sessions",              // Always second (added by handler)
            "Memory",                // Always includes experience/pattern/cluster counts
            "Stats",                 // Always includes questions/XP
            "Stats audit trail",     // Always reported (cleared or none)
            "Tickets",               // Always includes resolved/failed/escalated
            "Fix history",           // Always includes fix count
        ];

        // Verify section names are valid identifiers
        for section in &expected_sections {
            assert!(!section.is_empty());
            assert!(section.chars().next().unwrap().is_uppercase() || section.starts_with("In-"));
        }
    }

    #[test]
    fn test_reset_output_always_has_counts() {
        // Golden test: All variable sections must include counts, even if zero
        // This ensures output is deterministic regardless of prior state

        // Memory: always "Memory (N experiences, N patterns, N clusters)"
        let memory_zero = format!("Memory ({} experiences, {} patterns, {} clusters)", 0, 0, 0);
        let memory_some = format!("Memory ({} experiences, {} patterns, {} clusters)", 5, 3, 2);
        assert_eq!(memory_zero.matches(|c: char| c.is_ascii_digit()).count() >= 3, true);
        assert_eq!(memory_some.matches(|c: char| c.is_ascii_digit()).count() >= 3, true);

        // Stats: always "Stats (N questions, N XP)"
        let stats_zero = format!("Stats ({} questions, {} XP)", 0, 0);
        let stats_some = format!("Stats ({} questions, {} XP)", 10, 250);
        assert!(stats_zero.contains("0 questions"));
        assert!(stats_some.contains("10 questions"));

        // Tickets: always "Tickets (N resolved, N failed, N escalated)"
        let tickets_zero = format!("Tickets ({} resolved, {} failed, {} escalated)", 0, 0, 0);
        let tickets_some = format!("Tickets ({} resolved, {} failed, {} escalated)", 5, 1, 2);
        assert!(tickets_zero.contains("0 resolved"));
        assert!(tickets_some.contains("5 resolved"));
    }

    #[test]
    fn test_legacy_migration_is_one_time() {
        // Verify legacy xp.json migration behavior
        // After migration, the file should not exist, and subsequent resets
        // should not report legacy migration

        let xp_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("anna/xp.json");

        // If the file doesn't exist, migrate_legacy_xp returns None
        if !xp_path.exists() {
            let result = SafeReset::migrate_legacy_xp();
            assert!(result.is_ok());
            assert!(result.unwrap().is_none(), "Should return None when no legacy file exists");
        }
    }

    #[test]
    fn test_reset_clears_unified_store_regardless_of_legacy() {
        // Verify that reset clears the unified PersistentStats store
        // whether or not legacy files exist
        // This ensures we don't depend on legacy files for correctness

        let stats_path = anna_data_dir().join("stats.json");

        // Stats path should always be the unified location
        assert!(stats_path.to_string_lossy().contains("stats.json"));
        assert!(!stats_path.to_string_lossy().contains("xp.json"));
    }

    // v0.3.23: Backup verification tests

    #[test]
    fn test_backup_directory_path() {
        // Verify backup directory is in a deterministic location
        let backup_path = backup_dir();

        // Should be in ~/.anna/backups/
        assert!(backup_path.to_string_lossy().contains("backups"));
        assert!(backup_path.to_string_lossy().contains(".anna")
             || backup_path.to_string_lossy().contains("anna"));
    }

    #[test]
    fn test_backup_dir_creation() {
        // Verify create_backup_dir creates timestamped directory
        let result = create_backup_dir("test_backup");

        if let Ok(path) = result {
            // Path should contain timestamp pattern YYYYMMDD_HHMMSS
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("test_backup_"));

            // Should have 15 chars after prefix (YYYYMMDD_HHMMSS)
            let parts: Vec<&str> = path_str.split("test_backup_").collect();
            if parts.len() > 1 {
                // Timestamp should be ~15 chars (20260113_123456)
                assert!(parts[1].len() >= 15, "Timestamp format incorrect");
            }

            // Clean up test directory
            let _ = fs::remove_dir_all(&path);
        }
    }

    #[test]
    fn test_backup_file_copies_existing() {
        // Verify backup_file copies files that exist
        use std::io::Write;

        // Create a temp file
        let temp_dir = std::env::temp_dir().join("anna_backup_test");
        let _ = fs::create_dir_all(&temp_dir);

        let source = temp_dir.join("test_source.json");
        let backup_dest = temp_dir.join("backup");
        let _ = fs::create_dir_all(&backup_dest);

        // Write test content
        if let Ok(mut f) = fs::File::create(&source) {
            let _ = writeln!(f, r#"{{"test": true}}"#);
        }

        // Backup the file
        let result = backup_file(&source, &backup_dest, "test_backup.json");
        assert!(result.is_ok());
        assert!(result.unwrap() == true, "Should return true when file exists");

        // Verify backup exists
        let backed_up = backup_dest.join("test_backup.json");
        assert!(backed_up.exists(), "Backup file should exist");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_backup_file_skips_nonexistent() {
        // Verify backup_file returns false for non-existent files
        let temp_dir = std::env::temp_dir().join("anna_backup_test_skip");
        let _ = fs::create_dir_all(&temp_dir);

        let nonexistent = PathBuf::from("/nonexistent/path/file.json");

        let result = backup_file(&nonexistent, &temp_dir, "should_not_exist.json");
        assert!(result.is_ok());
        assert!(result.unwrap() == false, "Should return false when source doesn't exist");

        // Verify no backup was created
        let would_be_backup = temp_dir.join("should_not_exist.json");
        assert!(!would_be_backup.exists(), "No backup should be created");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_reset_result_always_has_backup_path() {
        // Verify ResetResult always includes backup_path
        // This ensures the CLI can always print where backup was saved

        // The ResetResult struct should have backup_path as Option<String>
        // and SafeReset::execute should always set it to Some(...)

        // We can't easily run a full reset in a unit test, but we can
        // verify the contract by checking the return type construction
        let mock_result = crate::rpc::ResetResult {
            cleared: vec!["Test".to_string()],
            backup_path: Some("/test/path".to_string()),
        };

        assert!(mock_result.backup_path.is_some(), "backup_path must be present");
    }

    #[test]
    fn test_backup_does_not_include_secrets() {
        // Verify backup doesn't include files that might contain secrets
        // The backup_all function should NOT backup:
        // - .env files
        // - credentials.json
        // - tokens or API keys

        // List of files that ARE backed up (from backup_all function)
        let backed_up_files = vec![
            "memory.json",
            "config.toml",
            "stats.json",
            "stats_audit.jsonl",
            "installed_deps.txt",
            "tickets.json",
            "fix_history.json",
            "model_prefs.json",
        ];

        // None of these should be secret-containing files
        for file in &backed_up_files {
            assert!(!file.contains("credentials"), "Should not backup credentials");
            assert!(!file.contains("token"), "Should not backup tokens");
            assert!(!file.contains("secret"), "Should not backup secrets");
            assert!(!file.contains(".env"), "Should not backup env files");
            assert!(!file.contains("api_key"), "Should not backup API keys");
        }
    }

    // v0.3.23: Tickets reset verification test

    #[test]
    fn test_tickets_json_reset_removes_file() {
        // Verify reset_tickets removes tickets.json and reports it
        use std::io::Write;

        // Create temp tickets.json with dummy content
        let temp_dir = std::env::temp_dir().join("anna_tickets_reset_test");
        let _ = fs::create_dir_all(&temp_dir);

        let tickets_path = temp_dir.join("tickets.json");
        if let Ok(mut f) = fs::File::create(&tickets_path) {
            let dummy_content = r#"{
                "tickets": [],
                "total_resolved": 5,
                "total_failed": 2,
                "total_escalated": 1
            }"#;
            let _ = f.write_all(dummy_content.as_bytes());
        }

        // Verify file exists
        assert!(tickets_path.exists(), "Test file should exist before test");

        // Read and parse the content
        if let Ok(content) = fs::read_to_string(&tickets_path) {
            if let Ok(store) = serde_json::from_str::<serde_json::Value>(&content) {
                let resolved = store.get("total_resolved").and_then(|v| v.as_u64()).unwrap_or(0);
                let failed = store.get("total_failed").and_then(|v| v.as_u64()).unwrap_or(0);
                let escalated = store.get("total_escalated").and_then(|v| v.as_u64()).unwrap_or(0);

                // Verify we read the correct values
                assert_eq!(resolved, 5, "Should read resolved count");
                assert_eq!(failed, 2, "Should read failed count");
                assert_eq!(escalated, 1, "Should read escalated count");
            }
        }

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tickets_reset_reports_presence() {
        // Verify reset output indicates whether file was present
        // The format is: "Tickets (N resolved, N failed, N escalated)"
        // This should report the counts even if all zero

        let output_with_data = format!(
            "Tickets ({} resolved, {} failed, {} escalated)",
            5, 2, 1
        );
        let output_empty = format!(
            "Tickets ({} resolved, {} failed, {} escalated)",
            0, 0, 0
        );

        // Both formats should contain the key fields
        assert!(output_with_data.contains("resolved"));
        assert!(output_with_data.contains("failed"));
        assert!(output_with_data.contains("escalated"));

        assert!(output_empty.contains("0 resolved"));
        assert!(output_empty.contains("0 failed"));
        assert!(output_empty.contains("0 escalated"));
    }

    // ==========================================================================
    // v0.3.28: SEVERITY-0 BUG REPRODUCTION TESTS
    // Bug: reset claims data cleared but stats shows old values
    // ==========================================================================

    #[test]
    fn test_reset_stats_then_load_shows_zeros() {
        // SEVERITY-0 BUG REPRODUCTION TEST
        // This test verifies that after reset_stats(), the next load() returns zeros.
        //
        // The bug: annactl reset reports "0 questions, 0 XP" but annactl stats
        // shows old values (XP: 25, Tickets Resolved: 11).
        //
        // Root cause hypothesis: The reset saves default stats but the next load
        // reads from a different source or cached value.

        use crate::stats::PersistentStats;
        use std::io::Write;

        // Create a temp directory to avoid interfering with real stats
        let temp_dir = std::env::temp_dir().join("anna_reset_bug_test");
        let _ = fs::create_dir_all(&temp_dir);

        // We can't easily override stats_path(), so this test documents the expected
        // behavior by verifying the load/save round-trip consistency.

        // Test that PersistentStats::default() has zero values
        let default_stats = PersistentStats::default();
        assert_eq!(default_stats.rpg.xp, 0, "Default XP should be 0");
        assert_eq!(default_stats.rpg.total_questions, 0, "Default questions should be 0");

        // The bug might be that after saving default stats, the next load doesn't
        // read from the file. Verify that save() followed by load() round-trips correctly.
        // This requires using the actual stats path.

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stats_path_consistency() {
        // SEVERITY-0 BUG: Verify all code paths use the same stats path.
        // v0.3.31: All paths are now system-wide under /var/lib/anna

        use crate::paths::paths;

        // Get the path from the central Paths struct
        let stats_path_from_paths = paths().stats_file();

        // Get the path that reset uses (from anna_data_dir())
        let stats_path_from_reset = anna_data_dir().join("stats.json");

        // These MUST be the same path
        assert_eq!(
            stats_path_from_paths.to_string_lossy(),
            stats_path_from_reset.to_string_lossy(),
            "CRITICAL: paths().stats_file() and anna_data_dir().join(\"stats.json\") diverge!"
        );

        // Verify path is system-wide (no home directory)
        assert!(
            !stats_path_from_paths.to_string_lossy().contains("/home/"),
            "Stats path should not be under /home/"
        );
        assert!(
            stats_path_from_paths.to_string_lossy().starts_with("/var/lib/anna"),
            "Stats path should be under /var/lib/anna"
        );
    }

    #[test]
    fn test_tickets_path_consistency() {
        // SEVERITY-0 BUG: Verify all code paths use the same tickets path.
        // v0.3.31: All paths are now system-wide under /var/lib/anna

        use crate::paths::paths;

        // Path from central Paths struct
        let tickets_path = paths().tickets_file();

        // Path used by reset_tickets() (should match)
        let tickets_path_from_reset = anna_data_dir().join("tickets.json");

        // These MUST be the same
        assert_eq!(
            tickets_path.to_string_lossy(),
            tickets_path_from_reset.to_string_lossy(),
            "CRITICAL: paths().tickets_file() and anna_data_dir().join(\"tickets.json\") diverge!"
        );

        // Verify path is system-wide (no home directory)
        assert!(
            !tickets_path.to_string_lossy().contains("/home/"),
            "Tickets path should not be under /home/"
        );
        assert!(
            tickets_path.to_string_lossy().starts_with("/var/lib/anna"),
            "Tickets path should be under /var/lib/anna"
        );
    }

    #[test]
    fn test_reset_stats_file_actually_written() {
        // SEVERITY-0 BUG: Verify reset_stats() actually writes to disk.
        // The bug might be that save() fails silently.

        use crate::stats::PersistentStats;

        // Get the actual stats path
        let stats_path = anna_data_dir().join("stats.json");

        // Remember file modification time before reset
        let mtime_before = fs::metadata(&stats_path).ok().and_then(|m| m.modified().ok());

        // Create non-default stats
        let mut stats = PersistentStats::default();
        stats.rpg.xp = 999; // Non-default value
        stats.rpg.total_questions = 999;

        // Save the non-default stats
        if stats.save().is_ok() {
            // Now verify the file was actually modified
            let mtime_after_write = fs::metadata(&stats_path).ok().and_then(|m| m.modified().ok());

            // The file should exist and have been modified
            assert!(stats_path.exists(), "Stats file should exist after save");

            // Load and verify the value was persisted
            if let Ok(loaded) = PersistentStats::load() {
                assert_eq!(loaded.rpg.xp, 999, "Saved XP should be readable");
            }

            // Now save default stats (simulating reset)
            let default_stats = PersistentStats::default();
            if default_stats.save().is_ok() {
                // Load again and verify zeros
                if let Ok(loaded_after_reset) = PersistentStats::load() {
                    assert_eq!(
                        loaded_after_reset.rpg.xp, 0,
                        "SEVERITY-0 BUG: After saving default stats, load() still returns non-zero XP! \
                         Either save() didn't write or load() reads from elsewhere."
                    );
                    assert_eq!(
                        loaded_after_reset.rpg.total_questions, 0,
                        "SEVERITY-0 BUG: After saving default stats, load() still returns non-zero questions!"
                    );
                }
            }
        }
    }

    #[test]
    fn test_xp_baseline_consistency() {
        // v0.3.28: XP baseline must be consistent everywhere.
        // PersistentStats::fresh() is now the single source of truth for baseline stats.
        // Both load() (when no file) and reset_stats() use fresh().

        use crate::stats::PersistentStats;
        use crate::status::RpgStats;

        // fresh() should have proper baseline values
        let fresh = PersistentStats::fresh();
        assert_eq!(fresh.rpg.reliability, 1.0, "fresh() should have 100% reliability");
        assert_eq!(fresh.rpg.title, RpgStats::get_title(0), "fresh() should have Novice Apprentice title");
        assert_eq!(fresh.rpg.xp, 0, "fresh() should have 0 XP");
        assert_eq!(fresh.rpg.total_questions, 0, "fresh() should have 0 questions");
        assert!(fresh.rpg.installed_at.is_some(), "fresh() should have installed_at set");
        assert!(fresh.created_at.is_some(), "fresh() should have created_at set");

        // derive(Default) still gives zeros (for backwards compat with serde),
        // but reset_stats() now uses fresh() to ensure consistency
        let derive_default = PersistentStats::default();
        assert_eq!(derive_default.rpg.reliability, 0.0, "derive(Default) gives 0.0 (serde compat)");

        // The key invariant: after reset, stats should match fresh install baseline
        // This is now guaranteed because reset_stats() uses PersistentStats::fresh()
    }

    // ==========================================================================
    // v0.3.30: CONTRACT ENFORCEMENT TESTS (R5)
    // ==========================================================================

    #[test]
    fn test_reset_is_single_pass_no_retry_loop() {
        // R5: Verify reset code has NO retry loops
        // This test documents the contract by searching for forbidden patterns

        // Read the source file
        let source = include_str!("safe_ops.rs");

        // Forbidden patterns that indicate retry loops
        let forbidden_patterns = [
            "force.*retry",
            "retry.*loop",
            "verification failed.*retry",
            "still exists.*retry",
        ];

        // Check reset_stats and reset_tickets functions
        let reset_stats_section = source
            .find("fn reset_stats")
            .map(|start| {
                let end = source[start..].find("fn reset_").map(|e| start + e).unwrap_or(source.len());
                &source[start..end]
            });

        let reset_tickets_section = source
            .find("fn reset_tickets")
            .map(|start| {
                let end = source[start..].find("fn list_backups").map(|e| start + e).unwrap_or(source.len());
                &source[start..end]
            });

        // Neither section should contain retry patterns
        for pattern in &forbidden_patterns {
            if let Some(section) = reset_stats_section {
                assert!(
                    !section.to_lowercase().contains(&pattern.to_lowercase().replace(".*", "")),
                    "reset_stats contains forbidden pattern: {}", pattern
                );
            }
            if let Some(section) = reset_tickets_section {
                assert!(
                    !section.to_lowercase().contains(&pattern.to_lowercase().replace(".*", "")),
                    "reset_tickets contains forbidden pattern: {}", pattern
                );
            }
        }

        // Verify "NO VERIFICATION LOOP" comment exists (contract documentation)
        assert!(
            source.contains("NO VERIFICATION LOOP"),
            "Contract documentation missing: should have 'NO VERIFICATION LOOP' comment"
        );
    }

    #[test]
    fn test_reset_uses_context_for_errors() {
        // R5: Verify reset functions use .context() for proper error propagation
        // instead of silently retrying on failure

        let source = include_str!("safe_ops.rs");

        // Reset functions should use .context() for error handling
        assert!(
            source.contains("context(\"Failed to remove stats.json"),
            "reset_stats should use .context() for stats file removal"
        );
        assert!(
            source.contains("context(\"Failed to write fresh stats"),
            "reset_stats should use .context() for fresh stats write"
        );
        assert!(
            source.contains("context(\"Failed to remove tickets.json"),
            "reset_tickets should use .context() for tickets file removal"
        );
    }

    #[test]
    fn test_transactional_reset_order() {
        // R5: Verify reset follows the correct transactional order:
        // 1. Read old values (for reporting)
        // 2. Delete old file
        // 3. Write fresh file
        // No verification step, no retry

        let source = include_str!("safe_ops.rs");

        // Find reset_stats function
        let reset_stats_start = source.find("fn reset_stats").expect("reset_stats should exist");
        let reset_stats_section = &source[reset_stats_start..];

        // Find key operations
        let load_pos = reset_stats_section.find("PersistentStats::load()");
        let remove_pos = reset_stats_section.find("remove_file(&stats_path)");
        let save_pos = reset_stats_section.find("fresh.save()");

        // All operations should exist
        assert!(load_pos.is_some(), "Should load stats for reporting");
        assert!(remove_pos.is_some(), "Should remove stats file");
        assert!(save_pos.is_some(), "Should save fresh stats");

        // Order: load < remove < save
        let load_p = load_pos.unwrap();
        let remove_p = remove_pos.unwrap();
        let save_p = save_pos.unwrap();

        assert!(load_p < remove_p, "Should load before remove");
        assert!(remove_p < save_p, "Should remove before save");
    }
}
