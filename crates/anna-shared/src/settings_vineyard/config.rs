// v0.0.767: Settings Vineyard Config
// Configuration for vineyard

use serde::{Deserialize, Serialize};
use super::types::{VineyardType, VineyardStatus};

/// Vineyard config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VineyardConfig {
    /// Name
    pub name: String,
    /// Vineyard type
    pub vineyard_type: VineyardType,
    /// Status
    pub status: VineyardStatus,
    /// Max vines
    pub max_vines: usize,
}

impl VineyardConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            vineyard_type: VineyardType::RedWine,
            status: VineyardStatus::Pruned,
            max_vines: 100,
        }
    }

    /// Set type
    pub fn vineyard_type(mut self, vt: VineyardType) -> Self {
        self.vineyard_type = vt;
        self
    }

    /// Set status
    pub fn status(mut self, s: VineyardStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max vines
    pub fn max_vines(mut self, max: usize) -> Self {
        self.max_vines = max;
        self
    }
}

impl Default for VineyardConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
