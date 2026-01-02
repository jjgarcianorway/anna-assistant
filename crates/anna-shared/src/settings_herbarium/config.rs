// v0.0.774: Settings Herbarium - Config
// Herbarium configuration

use serde::{Deserialize, Serialize};
use super::types::{HerbariumType, HerbariumStatus};

/// Herbarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HerbariumConfig {
    /// Name
    pub name: String,
    /// Herbarium type
    pub herbarium_type: HerbariumType,
    /// Status
    pub status: HerbariumStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl HerbariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            herbarium_type: HerbariumType::University,
            status: HerbariumStatus::Active,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn herbarium_type(mut self, ht: HerbariumType) -> Self {
        self.herbarium_type = ht;
        self
    }

    /// Set status
    pub fn status(mut self, s: HerbariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for HerbariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
