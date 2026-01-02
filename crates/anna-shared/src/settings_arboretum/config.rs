// v0.0.772: Settings Arboretum Config (Phase 348)
// Configuration for arboretum instances

use serde::{Deserialize, Serialize};
use super::types::{ArboretumType, ArboretumStatus};

/// Arboretum config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArboretumConfig {
    /// Name
    pub name: String,
    /// Arboretum type
    pub arboretum_type: ArboretumType,
    /// Status
    pub status: ArboretumStatus,
    /// Max specimens
    pub max_specimens: usize,
}

impl ArboretumConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arboretum_type: ArboretumType::Public,
            status: ArboretumStatus::Open,
            max_specimens: 100,
        }
    }

    /// Set type
    pub fn arboretum_type(mut self, at: ArboretumType) -> Self {
        self.arboretum_type = at;
        self
    }

    /// Set status
    pub fn status(mut self, s: ArboretumStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max specimens
    pub fn max_specimens(mut self, max: usize) -> Self {
        self.max_specimens = max;
        self
    }
}

impl Default for ArboretumConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
