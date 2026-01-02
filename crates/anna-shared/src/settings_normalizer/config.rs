// v0.0.645: Settings Normalizer Config (Phase 221)
// Configuration for settings normalization

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{NormalizationType, NormalizationRule};

/// Normalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizerConfig {
    /// Normalization type
    pub normalization_type: NormalizationType,
    /// Normalization rule
    pub rule: NormalizationRule,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Preserve original
    pub preserve_original: bool,
    /// Enabled
    pub enabled: bool,
}

impl NormalizerConfig {
    /// Create new config
    pub fn new(normalization_type: NormalizationType) -> Self {
        Self {
            normalization_type,
            rule: NormalizationRule::None,
            category: None,
            preserve_original: true,
            enabled: true,
        }
    }

    /// Set rule
    pub fn rule(mut self, rule: NormalizationRule) -> Self {
        self.rule = rule;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set preserve original
    pub fn preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self::new(NormalizationType::String)
    }
}
