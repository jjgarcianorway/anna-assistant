// v0.0.769: Settings Nursery - Config (Phase 345)

use serde::{Deserialize, Serialize};
use super::types::{NurseryType, NurseryStatus};

/// Nursery config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NurseryConfig {
    /// Name
    pub name: String,
    /// Nursery type
    pub nursery_type: NurseryType,
    /// Status
    pub status: NurseryStatus,
    /// Max seedlings
    pub max_seedlings: usize,
}

impl NurseryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nursery_type: NurseryType::Retail,
            status: NurseryStatus::Seeding,
            max_seedlings: 100,
        }
    }

    /// Set type
    pub fn nursery_type(mut self, nt: NurseryType) -> Self {
        self.nursery_type = nt;
        self
    }

    /// Set status
    pub fn status(mut self, s: NurseryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max seedlings
    pub fn max_seedlings(mut self, max: usize) -> Self {
        self.max_seedlings = max;
        self
    }
}

impl Default for NurseryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = NurseryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = NurseryConfig::new("test")
            .nursery_type(NurseryType::Specialty)
            .status(NurseryStatus::Growing);
        assert_eq!(c.nursery_type, NurseryType::Specialty);
        assert_eq!(c.status, NurseryStatus::Growing);
    }
}
