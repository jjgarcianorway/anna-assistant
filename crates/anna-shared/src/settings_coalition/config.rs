// v0.0.736: Settings Coalition - Config (Phase 312)
// Coalition configuration

use serde::{Deserialize, Serialize};
use super::types::{CoalitionType, CoalitionStatus};

/// Coalition config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionConfig {
    /// Name
    pub name: String,
    /// Coalition type
    pub coalition_type: CoalitionType,
    /// Status
    pub status: CoalitionStatus,
    /// Max agreements
    pub max_agreements: usize,
}

impl CoalitionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            coalition_type: CoalitionType::Governing,
            status: CoalitionStatus::Forming,
            max_agreements: 100,
        }
    }

    /// Set type
    pub fn coalition_type(mut self, ct: CoalitionType) -> Self {
        self.coalition_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CoalitionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max agreements
    pub fn max_agreements(mut self, max: usize) -> Self {
        self.max_agreements = max;
        self
    }
}

impl Default for CoalitionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = CoalitionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CoalitionConfig::new("test")
            .coalition_type(CoalitionType::Opposition)
            .status(CoalitionStatus::Stable);
        assert_eq!(c.coalition_type, CoalitionType::Opposition);
        assert_eq!(c.status, CoalitionStatus::Stable);
    }
}
