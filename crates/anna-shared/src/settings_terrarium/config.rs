// v0.0.777: Settings Terrarium (Phase 353)
// Terrarium configuration

use serde::{Deserialize, Serialize};
use crate::settings_terrarium::types::{TerrariumType, TerrariumStatus};

/// Terrarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrariumConfig {
    /// Name
    pub name: String,
    /// Terrarium type
    pub terrarium_type: TerrariumType,
    /// Status
    pub status: TerrariumStatus,
    /// Max plants
    pub max_plants: usize,
}

impl TerrariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            terrarium_type: TerrariumType::Desert,
            status: TerrariumStatus::Building,
            max_plants: 100,
        }
    }

    /// Set type
    pub fn terrarium_type(mut self, tt: TerrariumType) -> Self {
        self.terrarium_type = tt;
        self
    }

    /// Set status
    pub fn status(mut self, s: TerrariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max plants
    pub fn max_plants(mut self, max: usize) -> Self {
        self.max_plants = max;
        self
    }
}

impl Default for TerrariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let c = TerrariumConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = TerrariumConfig::new("test")
            .terrarium_type(TerrariumType::Woodland)
            .status(TerrariumStatus::Sealed);
        assert_eq!(c.terrarium_type, TerrariumType::Woodland);
        assert_eq!(c.status, TerrariumStatus::Sealed);
    }
}
