// v0.0.756: Settings Lot Config (Phase 332)
// Lot configuration

use serde::{Deserialize, Serialize};
use super::types::{LotType, LotStatus};

/// Lot config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotConfig {
    /// Name
    pub name: String,
    /// Lot type
    pub lot_type: LotType,
    /// Status
    pub status: LotStatus,
    /// Max deeds
    pub max_deeds: usize,
}

impl LotConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            lot_type: LotType::Residential,
            status: LotStatus::Vacant,
            max_deeds: 100,
        }
    }

    /// Set type
    pub fn lot_type(mut self, lt: LotType) -> Self {
        self.lot_type = lt;
        self
    }

    /// Set status
    pub fn status(mut self, s: LotStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max deeds
    pub fn max_deeds(mut self, max: usize) -> Self {
        self.max_deeds = max;
        self
    }
}

impl Default for LotConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
