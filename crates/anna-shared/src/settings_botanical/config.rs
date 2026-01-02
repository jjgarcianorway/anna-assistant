// v0.0.773: Settings Botanical Config (Phase 349)
// Configuration for botanical gardens

use serde::{Deserialize, Serialize};
use super::types::{BotanicalType, BotanicalStatus};

/// Botanical config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotanicalConfig {
    /// Name
    pub name: String,
    /// Botanical type
    pub botanical_type: BotanicalType,
    /// Status
    pub status: BotanicalStatus,
    /// Max collections
    pub max_collections: usize,
}

impl BotanicalConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            botanical_type: BotanicalType::Display,
            status: BotanicalStatus::Active,
            max_collections: 100,
        }
    }

    /// Set type
    pub fn botanical_type(mut self, bt: BotanicalType) -> Self {
        self.botanical_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BotanicalStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max collections
    pub fn max_collections(mut self, max: usize) -> Self {
        self.max_collections = max;
        self
    }
}

impl Default for BotanicalConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
