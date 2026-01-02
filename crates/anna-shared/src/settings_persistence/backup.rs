// v0.0.555: Settings Persistence - Backup operations (Phase 131)
// Handles backup and restore operations for settings

use std::fs;
use std::path::PathBuf;

use super::error::{SettingsError, SettingsResult};
use super::manager::SettingsPersistence;

impl SettingsPersistence {
    /// Get backup directory path
    pub fn backup_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("anna").join("settings_backups"))
    }

    /// Create a backup of current settings
    pub fn create_backup(&self) -> SettingsResult<PathBuf> {
        let backup_dir = Self::backup_dir().ok_or(SettingsError::PathUnavailable)?;
        fs::create_dir_all(&backup_dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = backup_dir.join(format!("settings_{}.json.bak", timestamp));

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| SettingsError::Serde(e.to_string()))?;
        fs::write(&backup_path, content)?;

        self.cleanup_old_backups()?;

        Ok(backup_path)
    }

    /// Restore from latest backup
    pub fn restore_latest() -> SettingsResult<Self> {
        let backup_dir = Self::backup_dir().ok_or(SettingsError::PathUnavailable)?;

        if !backup_dir.exists() {
            return Err(SettingsError::RestoreFailed("No backups found".into()));
        }

        let mut backups: Vec<_> = fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "bak").unwrap_or(false))
            .collect();

        backups.sort_by_key(|e| e.path());
        backups.reverse();

        let latest = backups
            .first()
            .ok_or(SettingsError::RestoreFailed("No backups found".into()))?;

        let content = fs::read_to_string(latest.path())?;
        serde_json::from_str(&content).map_err(|e| SettingsError::Serde(e.to_string()))
    }

    /// Restore from specific backup file
    pub fn restore_from(path: &PathBuf) -> SettingsResult<Self> {
        if !path.exists() {
            return Err(SettingsError::RestoreFailed("Backup file not found".into()));
        }

        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| SettingsError::Serde(e.to_string()))
    }

    /// List available backups
    pub fn list_backups() -> SettingsResult<Vec<PathBuf>> {
        let backup_dir = Self::backup_dir().ok_or(SettingsError::PathUnavailable)?;

        if !backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups: Vec<_> = fs::read_dir(&backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "bak").unwrap_or(false))
            .map(|e| e.path())
            .collect();

        backups.sort();
        backups.reverse();

        Ok(backups)
    }

    /// Clean up old backups beyond max_backups
    pub(super) fn cleanup_old_backups(&self) -> SettingsResult<()> {
        let backups = Self::list_backups()?;

        if backups.len() > self.max_backups as usize {
            for backup in backups.iter().skip(self.max_backups as usize) {
                fs::remove_file(backup)?;
            }
        }

        Ok(())
    }
}
