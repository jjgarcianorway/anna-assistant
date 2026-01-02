// v0.0.742: Zone Configuration (Phase 318)

use serde::{Deserialize, Serialize};
use super::types::{ZoneType, ZoneStatus};

/// Zone config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConfig {
    /// Name
    pub name: String,
    /// Zone type
    pub zone_type: ZoneType,
    /// Status
    pub status: ZoneStatus,
    /// Max regulations
    pub max_regulations: usize,
}

impl ZoneConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            zone_type: ZoneType::FreeTrade,
            status: ZoneStatus::Proposed,
            max_regulations: 100,
        }
    }

    /// Set type
    pub fn zone_type(mut self, zt: ZoneType) -> Self {
        self.zone_type = zt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ZoneStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max regulations
    pub fn max_regulations(mut self, max: usize) -> Self {
        self.max_regulations = max;
        self
    }
}

impl Default for ZoneConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ZoneConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ZoneConfig::new("test")
            .zone_type(ZoneType::Security)
            .status(ZoneStatus::Established);
        assert_eq!(c.zone_type, ZoneType::Security);
        assert_eq!(c.status, ZoneStatus::Established);
    }
}
