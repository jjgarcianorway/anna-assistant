// v0.0.650: Settings Converter Config (Phase 226)
// Configuration for settings conversion

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::formats::{SourceFormat, TargetFormat};

/// Converter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConverterConfig {
    /// Source format
    pub source: SourceFormat,
    /// Target format
    pub target: TargetFormat,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Preserve comments
    pub preserve_comments: bool,
    /// Pretty output
    pub pretty: bool,
}

impl ConverterConfig {
    /// Create new config
    pub fn new(source: SourceFormat, target: TargetFormat) -> Self {
        Self {
            source,
            target,
            category: None,
            preserve_comments: false,
            pretty: true,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set preserve comments
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }

    /// Set pretty output
    pub fn pretty(mut self, pretty: bool) -> Self {
        self.pretty = pretty;
        self
    }
}

impl Default for ConverterConfig {
    fn default() -> Self {
        Self::new(SourceFormat::Json, TargetFormat::Toml)
    }
}
