// v0.0.746: Settings Province - Config (Phase 322)
// Province configuration

use serde::{Deserialize, Serialize};
use super::types::{ProvinceType, ProvinceStatus};

/// Province config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvinceConfig {
    /// Name
    pub name: String,
    /// Province type
    pub province_type: ProvinceType,
    /// Status
    pub status: ProvinceStatus,
    /// Max edicts
    pub max_edicts: usize,
}

impl ProvinceConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            province_type: ProvinceType::Autonomous,
            status: ProvinceStatus::Established,
            max_edicts: 100,
        }
    }

    /// Set type
    pub fn province_type(mut self, pt: ProvinceType) -> Self {
        self.province_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ProvinceStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max edicts
    pub fn max_edicts(mut self, max: usize) -> Self {
        self.max_edicts = max;
        self
    }
}

impl Default for ProvinceConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ProvinceConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ProvinceConfig::new("test")
            .province_type(ProvinceType::Federal)
            .status(ProvinceStatus::Developing);
        assert_eq!(c.province_type, ProvinceType::Federal);
        assert_eq!(c.status, ProvinceStatus::Developing);
    }
}
