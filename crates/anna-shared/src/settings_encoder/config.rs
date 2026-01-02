// v0.0.648: Settings Encoder (Phase 224)
// Encoder configuration

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::format::{EncodingFormat, EncodingOptions};

/// Encoder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderConfig {
    /// Encoding format
    pub format: EncodingFormat,
    /// Encoding options
    pub options: EncodingOptions,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Include nulls
    pub include_nulls: bool,
    /// Sort keys
    pub sort_keys: bool,
}

impl EncoderConfig {
    /// Create new config
    pub fn new(format: EncodingFormat) -> Self {
        Self {
            format,
            options: EncodingOptions::Compact,
            category: None,
            include_nulls: false,
            sort_keys: false,
        }
    }

    /// Set options
    pub fn options(mut self, options: EncodingOptions) -> Self {
        self.options = options;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set include nulls
    pub fn include_nulls(mut self, include: bool) -> Self {
        self.include_nulls = include;
        self
    }

    /// Set sort keys
    pub fn sort_keys(mut self, sort: bool) -> Self {
        self.sort_keys = sort;
        self
    }
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self::new(EncodingFormat::Json)
    }
}
