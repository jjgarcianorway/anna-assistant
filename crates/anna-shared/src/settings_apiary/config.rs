// v0.0.779: Settings Apiary - Config (Phase 355)
// Apiary configuration

use serde::{Deserialize, Serialize};
use super::types::{ApiaryType, ApiaryStatus};

/// Apiary config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiaryConfig {
    /// Name
    pub name: String,
    /// Apiary type
    pub apiary_type: ApiaryType,
    /// Status
    pub status: ApiaryStatus,
    /// Max hives
    pub max_hives: usize,
}

impl ApiaryConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            apiary_type: ApiaryType::Honey,
            status: ApiaryStatus::Active,
            max_hives: 100,
        }
    }

    /// Set type
    pub fn apiary_type(mut self, at: ApiaryType) -> Self {
        self.apiary_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: ApiaryStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max hives
    pub fn max_hives(mut self, max: usize) -> Self {
        self.max_hives = max;
        self
    }
}

impl Default for ApiaryConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = ApiaryConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = ApiaryConfig::new("test")
            .apiary_type(ApiaryType::Queen)
            .status(ApiaryStatus::Swarming);
        assert_eq!(c.apiary_type, ApiaryType::Queen);
        assert_eq!(c.status, ApiaryStatus::Swarming);
    }
}
