// v0.0.649: Settings Decoder Config (Phase 225)
// Configuration for decoder behavior

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{DecodingFormat, DecodingMode};

/// Decoder config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoderConfig {
    /// Decoding format
    pub format: DecodingFormat,
    /// Decoding mode
    pub mode: DecodingMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Allow unknown keys
    pub allow_unknown: bool,
    /// Collect errors
    pub collect_errors: bool,
}

impl DecoderConfig {
    /// Create new config
    pub fn new(format: DecodingFormat) -> Self {
        Self {
            format,
            mode: DecodingMode::Strict,
            category: None,
            allow_unknown: false,
            collect_errors: true,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: DecodingMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set allow unknown
    pub fn allow_unknown(mut self, allow: bool) -> Self {
        self.allow_unknown = allow;
        self
    }

    /// Set collect errors
    pub fn collect_errors(mut self, collect: bool) -> Self {
        self.collect_errors = collect;
        self
    }
}

impl Default for DecoderConfig {
    fn default() -> Self {
        Self::new(DecodingFormat::Json)
    }
}
