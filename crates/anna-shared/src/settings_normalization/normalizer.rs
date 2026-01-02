// v0.0.667: Settings Normalization (Phase 243)
// Core normalizer implementation

use std::collections::HashMap;
use crate::settings_normalization::{
    NormalizerConfig, NormalizerStats, NormalizationResult, CaseStyle,
};

/// Settings normalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsNormalizer {
    /// Config
    config: NormalizerConfig,
    /// Stats
    stats: NormalizerStats,
}

impl SettingsNormalizer {
    /// Create new normalizer
    pub fn new(config: NormalizerConfig) -> Self {
        Self {
            config,
            stats: NormalizerStats::default(),
        }
    }

    /// Normalize key
    fn normalize_key(&self, key: &str) -> String {
        let mut result = key.to_string();

        if self.config.trim_whitespace {
            result = result.trim().to_string();
        }

        result = match self.config.key_case {
            CaseStyle::Lower => result.to_lowercase(),
            CaseStyle::Upper => result.to_uppercase(),
            CaseStyle::Snake => result.replace('-', "_").replace(' ', "_").to_lowercase(),
            CaseStyle::Kebab => result.replace('_', "-").replace(' ', "-").to_lowercase(),
            CaseStyle::Camel => result, // Would need more complex logic
        };

        result
    }

    /// Normalize value
    fn normalize_value(&self, value: &str) -> String {
        let mut result = value.to_string();

        if self.config.trim_whitespace {
            result = result.trim().to_string();
        }

        if self.config.collapse_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        result
    }

    /// Normalize settings
    pub fn normalize(&mut self, settings: &HashMap<String, String>) -> NormalizationResult {
        let mut result_settings = HashMap::new();
        let mut keys_normalized = 0;
        let mut values_normalized = 0;
        let mut keys_removed = 0;

        for (key, value) in settings {
            let normalized_key = self.normalize_key(key);
            let normalized_value = self.normalize_value(value);

            if self.config.remove_empty && normalized_value.is_empty() {
                keys_removed += 1;
                continue;
            }

            if normalized_key != *key {
                keys_normalized += 1;
            }
            if normalized_value != *value {
                values_normalized += 1;
            }

            result_settings.insert(normalized_key, normalized_value);
        }

        self.stats.record_type(self.config.normalization_type);

        let result = NormalizationResult::success(result_settings)
            .with_counts(keys_normalized, values_normalized, keys_removed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &NormalizerStats {
        &self.stats
    }

    /// Get config
    pub fn config(&self) -> &NormalizerConfig {
        &self.config
    }
}
