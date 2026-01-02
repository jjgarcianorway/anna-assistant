use serde::{Deserialize, Serialize};
use super::types::{VivariumType, VivariumStatus};

/// Vivarium config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VivariumConfig {
    /// Name
    pub name: String,
    /// Vivarium type
    pub vivarium_type: VivariumType,
    /// Status
    pub status: VivariumStatus,
    /// Max creatures
    pub max_creatures: usize,
}

impl VivariumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vivarium_type: VivariumType::Reptile,
            status: VivariumStatus::Setup,
            max_creatures: 100,
        }
    }

    /// Set type
    pub fn vivarium_type(mut self, vt: VivariumType) -> Self {
        self.vivarium_type = vt;
        self
    }

    /// Set status
    pub fn status(mut self, s: VivariumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max creatures
    pub fn max_creatures(mut self, max: usize) -> Self {
        self.max_creatures = max;
        self
    }
}

impl Default for VivariumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
