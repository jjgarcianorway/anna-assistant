// v0.0.716: Settings Missive Config (Phase 292)
// Configuration for missive system

use serde::{Deserialize, Serialize};
use super::types::{MissiveType, MissiveDelivery};

/// Missive config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveConfig {
    /// Name
    pub name: String,
    /// Missive type
    pub missive_type: MissiveType,
    /// Delivery method
    pub delivery: MissiveDelivery,
    /// Max missives
    pub max_missives: usize,
}

impl MissiveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            missive_type: MissiveType::Formal,
            delivery: MissiveDelivery::Standard,
            max_missives: 250,
        }
    }

    /// Set type
    pub fn missive_type(mut self, mt: MissiveType) -> Self {
        self.missive_type = mt;
        self
    }

    /// Set delivery
    pub fn delivery(mut self, d: MissiveDelivery) -> Self {
        self.delivery = d;
        self
    }

    /// Set max missives
    pub fn max_missives(mut self, max: usize) -> Self {
        self.max_missives = max;
        self
    }
}

impl Default for MissiveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
