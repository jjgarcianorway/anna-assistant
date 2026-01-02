// v0.0.555: Settings Persistence - Manager (Phase 131)
// Core settings persistence manager implementation

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::unified_settings::UnifiedSettings;

use super::error::{SettingsError, SettingsResult};
use super::format::SettingsFormat;

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
