// v0.0.783: Settings Refuge - Config (Phase 359)
// Refuge configuration

use serde::{Deserialize, Serialize};
use super::types::{RefugeType, RefugeStatus};

/// Refuge config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefugeConfig {
    /// Name
    pub name: String,
    /// Refuge type
    pub refuge_type: RefugeType,
    /// Status
    pub status: RefugeStatus,
    /// Max inhabitants
    pub max_inhabitants: usize,
}

impl RefugeConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            refuge_type: RefugeType::Wildlife,
            status: RefugeStatus::Active,
            max_inhabitants: 100,
        }
    }

    /// Set type
    pub fn refuge_type(mut self, rt: RefugeType) -> Self {
        self.refuge_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RefugeStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max inhabitants
    pub fn max_inhabitants(mut self, max: usize) -> Self {
        self.max_inhabitants = max;
        self
    }
}

impl Default for RefugeConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RefugeConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RefugeConfig::new("test")
            .refuge_type(RefugeType::Bird)
            .status(RefugeStatus::Sheltering);
        assert_eq!(c.refuge_type, RefugeType::Bird);
        assert_eq!(c.status, RefugeStatus::Sheltering);
    }
}
