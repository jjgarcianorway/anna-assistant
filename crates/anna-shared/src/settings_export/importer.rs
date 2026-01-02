// v0.0.558: Settings Export/Import (Phase 134)
// Settings import functionality

use std::fs;
use std::io;
use std::path::PathBuf;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

use super::types::SettingsExport;

/// Settings importer
#[derive(Debug, Clone, Default)]
pub struct SettingsImporter {
    /// Validate after import
    pub validate: bool,
    /// Merge with existing (instead of replace)
    pub merge: bool,
}

impl SettingsImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable validation after import
    pub fn with_validation(mut self) -> Self {
        self.validate = true;
        self
    }

    /// Enable merge mode
    pub fn merge_mode(mut self) -> Self {
        self.merge = true;
        self
    }

    /// Import settings from string
    pub fn import_string(&self, content: &str) -> SettingsResult<UnifiedSettings> {
        // Try to parse as full export first
        if let Ok(export) = serde_json::from_str::<SettingsExport>(content) {
            return Ok(export.settings);
        }
        if let Ok(export) = toml::from_str::<SettingsExport>(content) {
            return Ok(export.settings);
        }

        // Try to parse as bare settings
        if let Ok(settings) = serde_json::from_str::<UnifiedSettings>(content) {
            return Ok(settings);
        }
        if let Ok(settings) = toml::from_str::<UnifiedSettings>(content) {
            return Ok(settings);
        }

        Err(SettingsError::Serde("Failed to parse settings".into()))
    }

    /// Import settings from file
    pub fn import_file(&self, path: &PathBuf) -> SettingsResult<UnifiedSettings> {
        if !path.exists() {
            return Err(SettingsError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Settings file not found",
            )));
        }

        let content = fs::read_to_string(path)?;
        self.import_string(&content)
    }

    /// Import and merge with existing settings
    pub fn import_and_merge(
        &self,
        content: &str,
        _existing: &UnifiedSettings,
    ) -> SettingsResult<UnifiedSettings> {
        let imported = self.import_string(content)?;

        if self.merge {
            // For now, imported settings take precedence
            // Future: selective field merging
            Ok(imported)
        } else {
            Ok(imported)
        }
    }
}

/// Quick import helper
pub fn import_settings(content: &str) -> SettingsResult<UnifiedSettings> {
    SettingsImporter::new().import_string(content)
}
