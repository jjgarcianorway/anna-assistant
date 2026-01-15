//! Safe reset with automatic backup.
//! v0.3.21: Reset modes with automatic backups per reliability sprint spec.

use crate::config::{anna_data_dir, AnnaConfig};
use crate::memory::{memory_path, Memory};
use crate::rpc::{ResetMode, ResetResult};
use crate::stats::{PersistentStats, StatsAudit, StatsAuditEntry, StatsEventType};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

use super::backup_types::BackupInfo;
use super::backup_utils::{backup_file, create_backup_dir};

/// Safe reset with automatic backup.
pub struct SafeReset;

impl SafeReset {
    /// Perform reset with the specified mode.
    /// Always creates a backup before modifying anything.
    pub fn execute(mode: ResetMode) -> Result<ResetResult> {
        info!("Starting safe reset with mode: {:?}", mode);

        let backup_path = create_backup_dir(&format!("reset_{:?}", mode).to_lowercase())?;
        info!("Backup directory: {:?}", backup_path);

        let mut cleared = Vec::new();
        Self::backup_all(&backup_path)?;

        match mode {
            ResetMode::Memory => cleared.extend(Self::reset_memory()?),
            ResetMode::Config => cleared.extend(Self::reset_config()?),
            ResetMode::Models => cleared.extend(Self::reset_models()?),
            ResetMode::Helpers => cleared.extend(Self::reset_helpers()?),
            ResetMode::Everything => {
                cleared.extend(Self::reset_memory()?);
                cleared.extend(Self::reset_config()?);
                cleared.extend(Self::reset_models()?);
                cleared.extend(Self::reset_helpers()?);
                cleared.extend(Self::reset_stats()?);
                cleared.extend(Self::reset_tickets()?);
            }
        }

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

    /// Backup all Anna data files.
    fn backup_all(backup_dir: &PathBuf) -> Result<()> {
        let data_dir = anna_data_dir();
        backup_file(&memory_path(), backup_dir, "memory.json")?;
        backup_file(&data_dir.join("config.toml"), backup_dir, "config.toml")?;
        backup_file(&data_dir.join("stats.json"), backup_dir, "stats.json")?;
        backup_file(&data_dir.join("stats_audit.jsonl"), backup_dir, "stats_audit.jsonl")?;
        backup_file(&data_dir.join("installed_deps.txt"), backup_dir, "installed_deps.txt")?;
        backup_file(&data_dir.join("tickets.json"), backup_dir, "tickets.json")?;
        backup_file(&data_dir.join("fix_history.json"), backup_dir, "fix_history.json")?;
        backup_file(&data_dir.join("model_prefs.json"), backup_dir, "model_prefs.json")?;
        Ok(())
    }

    /// Reset memory (experiences, patterns, clusters).
    fn reset_memory() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let (exp_count, pattern_count, cluster_count) = match Memory::load() {
            Ok(memory) => {
                let counts = (
                    memory.experiences.len(),
                    memory.patterns.len(),
                    memory.clusters.len(),
                );
                let fresh = Memory::default();
                fresh.save()?;
                counts
            }
            Err(_) => {
                let fresh = Memory::default();
                fresh.save()?;
                (0, 0, 0)
            }
        };
        cleared.push(format!(
            "Memory ({} experiences, {} patterns, {} clusters)",
            exp_count, pattern_count, cluster_count
        ));
        Ok(cleared)
    }

    /// Reset config to defaults.
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

    /// Reset model preferences.
    fn reset_models() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let prefs_path = anna_data_dir().join("model_prefs.json");
        if prefs_path.exists() {
            fs::remove_file(&prefs_path)?;
            cleared.push("Model preferences (will re-detect on next start)".to_string());
        }
        Ok(cleared)
    }

    /// Reset helper tracking (does NOT uninstall packages).
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

    /// Reset stats (XP, questions answered).
    /// v0.3.30: TRANSACTIONAL - single atomic pass, no retry loops.
    fn reset_stats() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let legacy_migrated = Self::migrate_legacy_xp()?;

        // TRANSACTIONAL RESET: One pass, no retries
        let (questions, xp) = match PersistentStats::load() {
            Ok(stats) => (stats.rpg.total_questions, stats.rpg.xp),
            Err(_) => (0, 0),
        };

        let stats_path = anna_data_dir().join("stats.json");
        if stats_path.exists() {
            fs::remove_file(&stats_path)
                .context("Failed to remove stats.json - reset aborted")?;
        }

        let fresh = PersistentStats::fresh();
        fresh.save().context("Failed to write fresh stats - reset incomplete")?;

        // NO VERIFICATION LOOP - if save() returns Ok, we trust it
        cleared.push(format!("Stats ({} questions, {} XP)", questions, xp));

        if let Some(msg) = legacy_migrated {
            cleared.push(msg);
        }

        let audit_path = anna_data_dir().join("stats_audit.jsonl");
        let audit_existed = audit_path.exists();
        if audit_existed {
            fs::remove_file(&audit_path)?;
        }
        cleared.push(format!(
            "Stats audit trail ({})",
            if audit_existed { "cleared" } else { "none" }
        ));

        Ok(cleared)
    }

    /// Migrate legacy xp.json into PersistentStats (one-time).
    /// NOTE: Legacy user-local paths are no longer supported. System paths only.
    pub(crate) fn migrate_legacy_xp() -> Result<Option<String>> {
        // System paths only - no legacy user-local data to migrate
        Ok(None)
    }

    /// Reset ticket tracker.
    /// v0.3.30: TRANSACTIONAL - single atomic pass, no retry loops.
    fn reset_tickets() -> Result<Vec<String>> {
        let mut cleared = Vec::new();
        let tickets_path = anna_data_dir().join("tickets.json");

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
            fs::remove_file(&tickets_path)
                .context("Failed to remove tickets.json - reset aborted")?;
            counts
        } else {
            (0, 0, 0)
        };

        // NO VERIFICATION LOOP - if remove_file() returns Ok, file is gone
        cleared.push(format!(
            "Tickets ({} resolved, {} failed, {} escalated)",
            resolved, failed, escalated
        ));

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

        cleared.push(format!("Fix history ({} fixes)", fixes_count));
        Ok(cleared)
    }

    /// List available backups.
    pub fn list_backups() -> Result<Vec<BackupInfo>> {
        let backup_path = super::backup_utils::backup_dir();
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

                let file_count = fs::read_dir(&path)?.count();
                let size = Self::dir_size(&path)?;

                let created = fs::metadata(&path)?
                    .created()
                    .ok()
                    .map(|t| chrono::DateTime::<Utc>::from(t).to_rfc3339());

                backups.push(BackupInfo {
                    name,
                    path: path.display().to_string(),
                    file_count,
                    size_bytes: size,
                    created,
                });
            }
        }

        backups.sort_by(|a, b| b.name.cmp(&a.name));
        Ok(backups)
    }

    /// Calculate directory size.
    fn dir_size(path: &PathBuf) -> Result<u64> {
        let mut size = 0;
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            size += entry.metadata()?.len();
        }
        Ok(size)
    }

    /// Restore from a backup.
    pub fn restore_backup(backup_name: &str) -> Result<Vec<String>> {
        let backup_path = super::backup_utils::backup_dir().join(backup_name);
        if !backup_path.exists() {
            anyhow::bail!("Backup not found: {}", backup_name);
        }

        let data_dir = anna_data_dir();
        let mut restored = Vec::new();

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

    /// Delete old backups, keeping only the most recent N.
    pub fn cleanup_backups(keep_count: usize) -> Result<usize> {
        let mut backups = Self::list_backups()?;
        if backups.len() <= keep_count {
            return Ok(0);
        }

        let mut deleted = 0;
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
