// v0.0.766: Settings Orchard Config
// Orchard configuration

use serde::{Deserialize, Serialize};
use super::types::{OrchardType, OrchardStatus};

/// Orchard config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardConfig {
    /// Name
    pub name: String,
    /// Orchard type
    pub orchard_type: OrchardType,
    /// Status
    pub status: OrchardStatus,
    /// Max fruits
    pub max_fruits: usize,
}

impl OrchardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            orchard_type: OrchardType::Apple,
            status: OrchardStatus::Dormant,
            max_fruits: 100,
        }
    }

    /// Set type
    pub fn orchard_type(mut self, ot: OrchardType) -> Self {
        self.orchard_type = ot;
        self
    }

    /// Set status
    pub fn status(mut self, s: OrchardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max fruits
    pub fn max_fruits(mut self, max: usize) -> Self {
        self.max_fruits = max;
        self
    }
}

impl Default for OrchardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
