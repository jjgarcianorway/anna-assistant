// v0.0.666: Settings Transform Config (Phase 242)
// Configuration types for the transformer

use serde::{Deserialize, Serialize};
use super::types::{TransformType, TransformDirection};

/// Transformer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerConfig {
    /// Default transform type
    pub default_type: TransformType,
    /// Default direction
    pub default_direction: TransformDirection,
    /// Preserve originals
    pub preserve_originals: bool,
    /// Chain transforms
    pub chain_transforms: bool,
    /// Enable logging
    pub enable_logging: bool,
}

impl TransformerConfig {
    /// Create new config
    pub fn new(transform_type: TransformType) -> Self {
        Self {
            default_type: transform_type,
            default_direction: TransformDirection::Forward,
            preserve_originals: false,
            chain_transforms: true,
            enable_logging: false,
        }
    }

    /// Set direction
    pub fn direction(mut self, direction: TransformDirection) -> Self {
        self.default_direction = direction;
        self
    }

    /// Set preserve originals
    pub fn preserve_originals(mut self, preserve: bool) -> Self {
        self.preserve_originals = preserve;
        self
    }

    /// Set chain transforms
    pub fn chain_transforms(mut self, chain: bool) -> Self {
        self.chain_transforms = chain;
        self
    }
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self::new(TransformType::Map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = TransformerConfig::new(TransformType::Map);
        assert!(!c.preserve_originals);
    }

    #[test]
    fn test_config_builder() {
        let c = TransformerConfig::new(TransformType::Filter)
            .direction(TransformDirection::Reverse)
            .preserve_originals(true);
        assert_eq!(c.default_direction, TransformDirection::Reverse);
        assert!(c.preserve_originals);
    }
}
