// v0.0.780: Settings Butterfly (Phase 356)
// Butterfly configuration

use serde::{Deserialize, Serialize};
use super::types::{ButterflyType, ButterflyStatus};

/// Butterfly config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButterflyConfig {
    /// Name
    pub name: String,
    /// Butterfly type
    pub butterfly_type: ButterflyType,
    /// Status
    pub status: ButterflyStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ButterflyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            butterfly_type: ButterflyType::Tropical,
            status: ButterflyStatus::Active,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn butterfly_type(mut self, bt: ButterflyType) -> Self {
        self.butterfly_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ButterflyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ButterflyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ButterflyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ButterflyConfig::new("test")
            .butterfly_type(ButterflyType::Monarch)
            .status(ButterflyStatus::Breeding);
        assert_eq!(c.butterfly_type, ButterflyType::Monarch);
        assert_eq!(c.status, ButterflyStatus::Breeding);
    }
}
