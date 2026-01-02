// v0.0.764: Settings Pasture - Config (Phase 340)

use serde::{Deserialize, Serialize};

use super::types::{PastureType, PastureStatus};

/// Pasture config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PastureConfig {
    /// Name
    pub name: String,
    /// Pasture type
    pub pasture_type: PastureType,
    /// Status
    pub status: PastureStatus,
    /// Max herds
    pub max_herds: usize,
}

impl PastureConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            pasture_type: PastureType::Permanent,
            status: PastureStatus::Open,
            max_herds: 100,
        }
    }

    /// Set type
    pub fn pasture_type(mut self, pt: PastureType) -> Self {
        self.pasture_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PastureStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max herds
    pub fn max_herds(mut self, max: usize) -> Self {
        self.max_herds = max;
        self
    }
}

impl Default for PastureConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = PastureConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PastureConfig::new("test")
            .pasture_type(PastureType::Intensive)
            .status(PastureStatus::Rested);
        assert_eq!(c.pasture_type, PastureType::Intensive);
        assert_eq!(c.status, PastureStatus::Rested);
    }
}
