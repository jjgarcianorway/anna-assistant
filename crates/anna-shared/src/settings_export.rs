// v0.0.558: Settings Export/Import (Phase 134)
// Handles exporting and importing settings in various formats

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::settings_persistence::{SettingsError, SettingsResult};
use crate::unified_settings::UnifiedSettings;

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExportFormat {
    /// JSON format (default, human-readable)
    #[default]
    Json,
    /// TOML format (config-style)
    Toml,
    /// Minimal JSON (compact)
    JsonCompact,
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
            Self::Toml => write!(f, "TOML"),
            Self::JsonCompact => write!(f, "JSON (compact)"),
        }
    }
}

impl ExportFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json | Self::JsonCompact => "json",
            Self::Toml => "toml",
        }
    }

    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "toml" => Self::Toml,
            _ => Self::Json,
        }
    }

    /// All available formats
    pub fn all() -> Vec<Self> {
        vec![Self::Json, Self::Toml, Self::JsonCompact]
    }
}

/// Export options
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Format to export in
    pub format: ExportFormat,
    /// Include metadata (version, timestamp)
    pub include_metadata: bool,
    /// Include comments (where supported)
    pub include_comments: bool,
    /// Obfuscate sensitive values
    pub obfuscate_sensitive: bool,
}

impl ExportOptions {
    /// Create new export options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set format
    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }

    /// Include metadata
    pub fn with_metadata(mut self) -> Self {
        self.include_metadata = true;
        self
    }

    /// Obfuscate sensitive data
    pub fn obfuscate(mut self) -> Self {
        self.obfuscate_sensitive = true;
        self
    }
}

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export timestamp
    pub exported_at: chrono::DateTime<chrono::Utc>,
    /// Anna version that created export
    pub anna_version: String,
    /// Export format used
    pub format: String,
    /// Optional description
    pub description: Option<String>,
}

impl Default for ExportMetadata {
    fn default() -> Self {
        Self {
            exported_at: chrono::Utc::now(),
            anna_version: env!("CARGO_PKG_VERSION").to_string(),
            format: ExportFormat::default().to_string(),
            description: None,
        }
    }
}

/// Settings export with optional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsExport {
    /// Export metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ExportMetadata>,
    /// The settings
    pub settings: UnifiedSettings,
}

impl SettingsExport {
    /// Create new settings export
    pub fn new(settings: UnifiedSettings) -> Self {
        Self {
            metadata: None,
            settings,
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self) -> Self {
        self.metadata = Some(ExportMetadata::default());
        self
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        if let Some(ref mut meta) = self.metadata {
            meta.description = Some(description.into());
        } else {
            let mut meta = ExportMetadata::default();
            meta.description = Some(description.into());
            self.metadata = Some(meta);
        }
        self
    }
}

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

/// Quick export helper
pub fn export_settings(settings: &UnifiedSettings, format: ExportFormat) -> SettingsResult<String> {
    SettingsExporter::new()
        .with_options(ExportOptions::new().format(format))
        .export_string(settings)
}

/// Quick import helper
pub fn import_settings(content: &str) -> SettingsResult<UnifiedSettings> {
    SettingsImporter::new().import_string(content)
}

/// Detect format from file path
pub fn detect_format(path: &PathBuf) -> ExportFormat {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(ExportFormat::from_extension)
        .unwrap_or_default()
}

/// Format export info for display
pub fn format_export_info(format: ExportFormat, path: Option<&PathBuf>) -> String {
    let mut output = String::new();
    output.push_str(&format!("Format: {}\n", format));
    if let Some(p) = path {
        output.push_str(&format!("Path: {}\n", p.display()));
    }
    output
}

/// Fun fact about settings export
pub fn settings_export_fun_fact() -> &'static str {
    "Anna can export your settings to share with friends or backup to a USB drive!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format_display() {
        assert_eq!(format!("{}", ExportFormat::Json), "JSON");
        assert_eq!(format!("{}", ExportFormat::Toml), "TOML");
        assert_eq!(format!("{}", ExportFormat::JsonCompact), "JSON (compact)");
    }

    #[test]
    fn test_export_format_extension() {
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Toml.extension(), "toml");
    }

    #[test]
    fn test_export_format_from_extension() {
        assert_eq!(ExportFormat::from_extension("toml"), ExportFormat::Toml);
        assert_eq!(ExportFormat::from_extension("json"), ExportFormat::Json);
        assert_eq!(ExportFormat::from_extension("unknown"), ExportFormat::Json);
    }

    #[test]
    fn test_export_options_default() {
        let options = ExportOptions::new();
        assert_eq!(options.format, ExportFormat::Json);
        assert!(!options.include_metadata);
    }

    #[test]
    fn test_export_options_builder() {
        let options = ExportOptions::new()
            .format(ExportFormat::Toml)
            .with_metadata()
            .obfuscate();
        assert_eq!(options.format, ExportFormat::Toml);
        assert!(options.include_metadata);
        assert!(options.obfuscate_sensitive);
    }

    #[test]
    fn test_export_metadata_default() {
        let meta = ExportMetadata::default();
        assert!(!meta.anna_version.is_empty());
        assert!(meta.description.is_none());
    }

    #[test]
    fn test_settings_export_new() {
        let settings = UnifiedSettings::default();
        let export = SettingsExport::new(settings);
        assert!(export.metadata.is_none());
    }

    #[test]
    fn test_settings_export_with_metadata() {
        let settings = UnifiedSettings::default();
        let export = SettingsExport::new(settings).with_metadata();
        assert!(export.metadata.is_some());
    }

    #[test]
    fn test_exporter_json() {
        let settings = UnifiedSettings::default();
        let exporter = SettingsExporter::new();
        let result = exporter.export_string(&settings);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("personality"));
    }

    #[test]
    fn test_importer_json() {
        let settings = UnifiedSettings::default();
        let exported = export_settings(&settings, ExportFormat::Json).unwrap();
        let imported = import_settings(&exported).unwrap();
        // Basic check that import worked
        assert_eq!(
            imported.personality.formality,
            settings.personality.formality
        );
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(detect_format(&PathBuf::from("test.toml")), ExportFormat::Toml);
        assert_eq!(detect_format(&PathBuf::from("test.json")), ExportFormat::Json);
    }

    #[test]
    fn test_default_filename() {
        let exporter = SettingsExporter::new();
        let filename = exporter.default_filename();
        assert!(filename.contains("anna_settings"));
        assert!(filename.contains(".json"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_export_fun_fact();
        assert!(fact.contains("export"));
    }
}
