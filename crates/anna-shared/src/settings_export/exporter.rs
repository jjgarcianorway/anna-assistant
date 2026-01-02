// v0.0.558: Settings Export/Import (Phase 134)
// Settings export functionality

use std::fs;
use std::path::PathBuf;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

use super::types::{ExportFormat, ExportOptions, SettingsExport};

/// Settings exporter
#[derive(Debug, Clone, Default)]
pub struct SettingsExporter {
    /// Export options
    pub options: ExportOptions,
}

impl SettingsExporter {
    /// Create new exporter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set options
    pub fn with_options(mut self, options: ExportOptions) -> Self {
        self.options = options;
        self
    }

    /// Export settings to string
    pub fn export_string(&self, settings: &UnifiedSettings) -> SettingsResult<String> {
        let export = if self.options.include_metadata {
            SettingsExport::new(settings.clone()).with_metadata()
        } else {
            SettingsExport::new(settings.clone())
        };

        match self.options.format {
            ExportFormat::Json => serde_json::to_string_pretty(&export)
                .map_err(|e| SettingsError::Serde(e.to_string())),
            ExportFormat::JsonCompact => {
                serde_json::to_string(&export).map_err(|e| SettingsError::Serde(e.to_string()))
            }
            ExportFormat::Toml => {
                toml::to_string_pretty(&export).map_err(|e| SettingsError::Serde(e.to_string()))
            }
        }
    }

    /// Export settings to file
    pub fn export_file(&self, settings: &UnifiedSettings, path: &PathBuf) -> SettingsResult<()> {
        let content = self.export_string(settings)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate default export filename
    pub fn default_filename(&self) -> String {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        format!("anna_settings_{}.{}", timestamp, self.options.format.extension())
    }
}

/// Quick export helper
pub fn export_settings(settings: &UnifiedSettings, format: ExportFormat) -> SettingsResult<String> {
    SettingsExporter::new()
        .with_options(ExportOptions::new().format(format))
        .export_string(settings)
}
