// v0.0.775: Settings Aquarium - Config Module (Phase 351)
// Aquarium configuration

use serde::{Deserialize, Serialize};
use super::types::{AquariumType, AquariumStatus};

/// Aquarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AquariumConfig {
    /// Name
    pub name: String,
    /// Aquarium type
    pub aquarium_type: AquariumType,
    /// Status
    pub status: AquariumStatus,
    /// Max inhabitants
    pub max_inhabitants: usize,
}

impl AquariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            aquarium_type: AquariumType::Freshwater,
            status: AquariumStatus::Cycling,
            max_inhabitants: 100,
        }
    }

    /// Set type
    pub fn aquarium_type(mut self, at: AquariumType) -> Self {
        self.aquarium_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: AquariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max inhabitants
    pub fn max_inhabitants(mut self, max: usize) -> Self {
        self.max_inhabitants = max;
        self
    }
}

impl Default for AquariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = AquariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = AquariumConfig::new("test")
            .aquarium_type(AquariumType::Reef)
            .status(AquariumStatus::Stocking);
        assert_eq!(c.aquarium_type, AquariumType::Reef);
        assert_eq!(c.status, AquariumStatus::Stocking);
    }
}
