// v0.0.558: Settings Export/Import (Phase 134)
// Type definitions for settings export/import

use serde::{Deserialize, Serialize};

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
