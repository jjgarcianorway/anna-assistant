// v0.0.668: Settings Denormalizer
// Main denormalizer implementation

use std::collections::HashMap;
use super::config::DenormalizerConfig;
use super::result::DenormalizationResult;
use super::stats::DenormalizerStats;

/// Settings denormalizer
#[derive(Debug, Clone, Default)]
pub struct SettingsDenormalizer {
    /// Config
    config: DenormalizerConfig,
    /// Stats
    stats: DenormalizerStats,
}

impl SettingsDenormalizer {
    /// Create new denormalizer
    pub fn new(config: DenormalizerConfig) -> Self {
        Self {
            config,
            stats: DenormalizerStats::default(),
        }
    }

    /// Denormalize settings
    pub fn denormalize(&mut self, settings: &HashMap<String, String>) -> DenormalizationResult {
        let mut result_settings = HashMap::new();
        let mut keys_prefixed = 0;
        let mut keys_suffixed = 0;

        for (key, value) in settings {
            let mut new_key = key.clone();

            if !self.config.key_prefix.is_empty() {
                new_key = format!("{}{}", self.config.key_prefix, new_key);
                keys_prefixed += 1;
            }

            if !self.config.key_suffix.is_empty() {
                new_key = format!("{}{}", new_key, self.config.key_suffix);
                keys_suffixed += 1;
            }

            result_settings.insert(new_key, value.clone());

            if self.config.preserve_original && (keys_prefixed > 0 || keys_suffixed > 0) {
                result_settings.insert(key.clone(), value.clone());
            }
        }

        self.stats.record_type(self.config.denorm_type);

        let result = DenormalizationResult::success(result_settings)
            .with_counts(0, keys_prefixed, keys_suffixed);

        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &DenormalizerStats {
        &self.stats
    }

    /// Get config
    pub fn config(&self) -> &DenormalizerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_denormalization::types::DenormalizationType;

    #[test]
    fn test_denormalizer_new() {
        let d = SettingsDenormalizer::new(DenormalizerConfig::default());
        assert_eq!(d.stats().total_denormalizations, 0);
    }

    #[test]
    fn test_denormalizer_prefix() {
        let mut d = SettingsDenormalizer::new(
            DenormalizerConfig::new(DenormalizationType::Prefix)
                .key_prefix("app.")
        );
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());

        let result = d.denormalize(&settings);
        assert!(result.settings.contains_key("app.key"));
    }

    #[test]
    fn test_denormalizer_suffix() {
        let mut d = SettingsDenormalizer::new(
            DenormalizerConfig::new(DenormalizationType::Suffix)
                .key_suffix("_setting")
        );
        let mut settings = HashMap::new();
        settings.insert("key".to_string(), "value".to_string());

        let result = d.denormalize(&settings);
        assert!(result.settings.contains_key("key_setting"));
    }
}
