// v0.0.749: Settings County Config (Phase 325)
// County configuration

use serde::{Deserialize, Serialize};
use super::types::{CountyType, CountyStatus};

/// County config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountyConfig {
    /// Name
    pub name: String,
    /// County type
    pub county_type: CountyType,
    /// Status
    pub status: CountyStatus,
    /// Max ordinances
    pub max_ordinances: usize,
}

impl CountyConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            county_type: CountyType::Metropolitan,
            status: CountyStatus::Established,
            max_ordinances: 100,
        }
    }

    /// Set type
    pub fn county_type(mut self, ct: CountyType) -> Self {
        self.county_type = ct;
        self
    }

    /// Set status
    pub fn status(mut self, s: CountyStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ordinances
    pub fn max_ordinances(mut self, max: usize) -> Self {
        self.max_ordinances = max;
        self
    }
}

impl Default for CountyConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = CountyConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CountyConfig::new("test")
            .county_type(CountyType::Historic)
            .status(CountyStatus::Active);
        assert_eq!(c.county_type, CountyType::Historic);
        assert_eq!(c.status, CountyStatus::Active);
    }
}
