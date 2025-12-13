// v0.0.555: Settings Persistence (Phase 131)
// Handles saving/loading unified settings to/from disk

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use crate::unified_settings::UnifiedSettings;

/// Result type for settings operations
pub type SettingsResult<T> = Result<T, SettingsError>;

/// Settings persistence error
#[derive(Debug)]
pub enum SettingsError {
    /// IO error during read/write
    Io(io::Error),
    /// Serialization/deserialization error
    Serde(String),
    /// Path not available
    PathUnavailable,
    /// Backup failed
    BackupFailed(String),
    /// Restore failed
    RestoreFailed(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::Serde(e) => write!(f, "Serialization error: {}", e),
            Self::PathUnavailable => write!(f, "Settings path unavailable"),
            Self::BackupFailed(e) => write!(f, "Backup failed: {}", e),
            Self::RestoreFailed(e) => write!(f, "Restore failed: {}", e),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<io::Error> for SettingsError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Settings file format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SettingsFormat {
    /// JSON format (default)
    #[default]
    Json,
    /// TOML format
    Toml,
}

impl std::fmt::Display for SettingsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
        }
    }
}

/// Settings persistence manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPersistence {
    /// Active settings
    pub settings: UnifiedSettings,
    /// Preferred format
    pub format: SettingsFormat,
    /// Auto-save on change
    pub auto_save: bool,
    /// Create backup before save
    pub backup_on_save: bool,
    /// Maximum backup count
    pub max_backups: u8,
}

impl Default for SettingsPersistence {
    fn default() -> Self {
        Self {
            settings: UnifiedSettings::default(),
            format: SettingsFormat::default(),
            auto_save: true,
            backup_on_save: true,
            max_backups: 5,
        }
    }
}

impl SettingsPersistence {
    /// Create new settings persistence manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from disk
    pub fn load() -> SettingsResult<Self> {
        let path = Self::settings_path().ok_or(SettingsError::PathUnavailable)?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;

        // Try JSON first
        if let Ok(persistence) = serde_json::from_str::<Self>(&content) {
            return Ok(persistence);
        }

        // Try TOML
        if let Ok(persistence) = toml::from_str::<Self>(&content) {
            return Ok(persistence);
        }

        // Try loading just settings (legacy format)
        if let Ok(settings) = serde_json::from_str::<UnifiedSettings>(&content) {
            return Ok(Self {
                settings,
                ..Default::default()
            });
        }

        Err(SettingsError::Serde("Failed to parse settings file".into()))
    }

    /// Save settings to disk
    pub fn save(&self) -> SettingsResult<()> {
        let path = Self::settings_path().ok_or(SettingsError::PathUnavailable)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if self.backup_on_save && path.exists() {
            self.create_backup()?;
        }

        let content = match self.format {
            SettingsFormat::Json => serde_json::to_string_pretty(self)
                .map_err(|e| SettingsError::Serde(e.to_string()))?,
            SettingsFormat::Toml => {
                toml::to_string_pretty(self).map_err(|e| SettingsError::Serde(e.to_string()))?
            }
        };

        fs::write(&path, content)?;
        Ok(())
    }

    /// Get settings file path
    pub fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("anna").join("settings.json"))
    }

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
    fn cleanup_old_backups(&self) -> SettingsResult<()> {
        let backups = Self::list_backups()?;

        if backups.len() > self.max_backups as usize {
            for backup in backups.iter().skip(self.max_backups as usize) {
                fs::remove_file(backup)?;
            }
        }

        Ok(())
    }

    /// Apply a natural language change
    pub fn apply_change(&mut self, request: &str) -> Option<String> {
        let result = self.settings.apply_change(request);

        if result.is_some() && self.auto_save {
            let _ = self.save();
        }

        result
    }

    /// Reset all settings to defaults
    pub fn reset_all(&mut self) {
        self.settings.reset_all();
        if self.auto_save {
            let _ = self.save();
        }
    }

    /// Check if settings file exists
    pub fn exists() -> bool {
        Self::settings_path().map(|p| p.exists()).unwrap_or(false)
    }

    /// Delete settings file
    pub fn delete() -> SettingsResult<()> {
        let path = Self::settings_path().ok_or(SettingsError::PathUnavailable)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Export settings to specific path
    pub fn export_to(&self, path: &PathBuf, format: SettingsFormat) -> SettingsResult<()> {
        let content = match format {
            SettingsFormat::Json => serde_json::to_string_pretty(&self.settings)
                .map_err(|e| SettingsError::Serde(e.to_string()))?,
            SettingsFormat::Toml => toml::to_string_pretty(&self.settings)
                .map_err(|e| SettingsError::Serde(e.to_string()))?,
        };
        fs::write(path, content)?;
        Ok(())
    }

    /// Import settings from specific path
    pub fn import_from(path: &PathBuf) -> SettingsResult<UnifiedSettings> {
        let content = fs::read_to_string(path)?;

        // Try JSON first
        if let Ok(settings) = serde_json::from_str(&content) {
            return Ok(settings);
        }

        // Try TOML
        if let Ok(settings) = toml::from_str(&content) {
            return Ok(settings);
        }

        Err(SettingsError::Serde("Failed to parse settings file".into()))
    }

    /// Enable auto-save
    pub fn enable_auto_save(&mut self) {
        self.auto_save = true;
    }

    /// Disable auto-save
    pub fn disable_auto_save(&mut self) {
        self.auto_save = false;
    }

    /// Is auto-save enabled?
    pub fn is_auto_save(&self) -> bool {
        self.auto_save
    }
}

/// Format settings summary for display
pub fn format_persistence_status() -> String {
    let mut output = String::new();

    output.push_str("=== Settings Persistence ===\n\n");

    if let Some(path) = SettingsPersistence::settings_path() {
        output.push_str(&format!("Config path: {}\n", path.display()));
        output.push_str(&format!(
            "Settings exist: {}\n",
            if path.exists() { "Yes" } else { "No" }
        ));
    }

    if let Ok(backups) = SettingsPersistence::list_backups() {
        output.push_str(&format!("Backup count: {}\n", backups.len()));
    }

    output
}

/// Check if settings persistence is available
pub fn is_persistence_available() -> bool {
    SettingsPersistence::settings_path().is_some()
}

/// Fun fact about settings persistence
pub fn settings_persistence_fun_fact() -> &'static str {
    "Anna keeps up to 5 backup copies of your settings - you can always restore to a previous configuration!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_settings_format_display() {
        assert_eq!(format!("{}", SettingsFormat::Json), "JSON");
        assert_eq!(format!("{}", SettingsFormat::Toml), "TOML");
    }

    #[test]
    fn test_default_persistence() {
        let persistence = SettingsPersistence::default();
        assert!(persistence.auto_save);
        assert!(persistence.backup_on_save);
        assert_eq!(persistence.max_backups, 5);
    }

    #[test]
    fn test_settings_error_display() {
        let err = SettingsError::PathUnavailable;
        assert!(format!("{}", err).contains("unavailable"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(ErrorKind::NotFound, "test");
        let settings_err = SettingsError::from(io_err);
        assert!(matches!(settings_err, SettingsError::Io(_)));
    }

    #[test]
    fn test_settings_path() {
        // Should return Some on most systems
        let path = SettingsPersistence::settings_path();
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("anna"));
        }
    }

    #[test]
    fn test_backup_dir() {
        let path = SettingsPersistence::backup_dir();
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("backups"));
        }
    }

    #[test]
    fn test_apply_change() {
        let mut persistence = SettingsPersistence::new();
        persistence.auto_save = false; // Don't actually save during test

        let result = persistence.apply_change("enable learning mode");
        assert!(result.is_some());
    }

    #[test]
    fn test_enable_disable_auto_save() {
        let mut persistence = SettingsPersistence::new();
        assert!(persistence.is_auto_save());

        persistence.disable_auto_save();
        assert!(!persistence.is_auto_save());

        persistence.enable_auto_save();
        assert!(persistence.is_auto_save());
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_persistence_fun_fact();
        assert!(fact.contains("backup"));
    }

    #[test]
    fn test_persistence_status() {
        let status = format_persistence_status();
        assert!(status.contains("Persistence"));
    }

    #[test]
    fn test_is_persistence_available() {
        // Should usually be true
        let _ = is_persistence_available();
    }
}
