// v0.0.759: Settings Tract Config (Phase 335)
// Tract configuration

use serde::{Deserialize, Serialize};
use super::types::{TractType, TractStatus};

/// Tract config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TractConfig {
    /// Name
    pub name: String,
    /// Tract type
    pub tract_type: TractType,
    /// Status
    pub status: TractStatus,
    /// Max grants
    pub max_grants: usize,
}

impl TractConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tract_type: TractType::Residential,
            status: TractStatus::Surveyed,
            max_grants: 100,
        }
    }

    /// Set type
    pub fn tract_type(mut self, tt: TractType) -> Self {
        self.tract_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TractStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max grants
    pub fn max_grants(mut self, max: usize) -> Self {
        self.max_grants = max;
        self
    }
}

impl Default for TractConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = TractConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TractConfig::new("test")
            .tract_type(TractType::Wilderness)
            .status(TractStatus::Disputed);
        assert_eq!(c.tract_type, TractType::Wilderness);
        assert_eq!(c.status, TractStatus::Disputed);
    }
}
