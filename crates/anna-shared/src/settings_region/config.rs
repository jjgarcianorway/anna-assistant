// v0.0.747: Settings Region Config (Phase 323)
// Region configuration

use serde::{Deserialize, Serialize};
use super::types::{RegionType, RegionStatus};

/// Region config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// Name
    pub name: String,
    /// Region type
    pub region_type: RegionType,
    /// Status
    pub status: RegionStatus,
    /// Max policies
    pub max_policies: usize,
}

impl RegionConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            region_type: RegionType::Administrative,
            status: RegionStatus::Defined,
            max_policies: 100,
        }
    }

    /// Set type
    pub fn region_type(mut self, rt: RegionType) -> Self {
        self.region_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: RegionStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max policies
    pub fn max_policies(mut self, max: usize) -> Self {
        self.max_policies = max;
        self
    }
}

impl Default for RegionConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = RegionConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = RegionConfig::new("test")
            .region_type(RegionType::Cultural)
            .status(RegionStatus::Expanding);
        assert_eq!(c.region_type, RegionType::Cultural);
        assert_eq!(c.status, RegionStatus::Expanding);
    }
}
