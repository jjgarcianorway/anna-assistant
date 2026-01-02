// v0.0.763: Settings Meadow Config
// Configuration for meadows

use serde::{Deserialize, Serialize};
use super::types::{MeadowType, MeadowStatus};

/// Meadow config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowConfig {
    /// Name
    pub name: String,
    /// Meadow type
    pub meadow_type: MeadowType,
    /// Status
    pub status: MeadowStatus,
    /// Max grasses
    pub max_grasses: usize,
}

impl MeadowConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            meadow_type: MeadowType::Hay,
            status: MeadowStatus::Resting,
            max_grasses: 100,
        }
    }

    /// Set type
    pub fn meadow_type(mut self, mt: MeadowType) -> Self {
        self.meadow_type = mt;
        self
    }

    /// Set status
    pub fn status(mut self, s: MeadowStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max grasses
    pub fn max_grasses(mut self, max: usize) -> Self {
        self.max_grasses = max;
        self
    }
}

impl Default for MeadowConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
