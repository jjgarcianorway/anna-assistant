// v0.0.668: Denormalizer Configuration
// Configuration for the denormalizer

use serde::{Deserialize, Serialize};
use super::types::{DenormalizationType, TargetFormat};

/// Denormalizer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenormalizerConfig {
    /// Denormalization type
    pub denorm_type: DenormalizationType,
    /// Target format
    pub target_format: TargetFormat,
    /// Key prefix
    pub key_prefix: String,
    /// Key suffix
    pub key_suffix: String,
    /// Preserve original keys
    pub preserve_original: bool,
}

impl DenormalizerConfig {
    /// Create new config
    pub fn new(denorm_type: DenormalizationType) -> Self {
        Self {
            denorm_type,
            target_format: TargetFormat::Json,
            key_prefix: String::new(),
            key_suffix: String::new(),
            preserve_original: false,
        }
    }

    /// Set target format
    pub fn target_format(mut self, format: TargetFormat) -> Self {
        self.target_format = format;
        self
    }

    /// Set key prefix
    pub fn key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }

    /// Set key suffix
    pub fn key_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.key_suffix = suffix.into();
        self
    }

    /// Set preserve original
    pub fn preserve_original(mut self, preserve: bool) -> Self {
        self.preserve_original = preserve;
        self
    }
}

impl Default for DenormalizerConfig {
    fn default() -> Self {
        Self::new(DenormalizationType::Expand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = DenormalizerConfig::new(DenormalizationType::Expand);
        assert!(!c.preserve_original);
    }

    #[test]
    fn test_config_builder() {
        let c = DenormalizerConfig::new(DenormalizationType::Prefix)
            .key_prefix("app.")
            .preserve_original(true);
        assert_eq!(c.key_prefix, "app.");
        assert!(c.preserve_original);
    }
}
