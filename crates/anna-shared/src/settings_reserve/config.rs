// v0.0.782: Settings Reserve - Config
// Reserve configuration

use serde::{Deserialize, Serialize};
use super::types::{ReserveType, ReserveStatus};

/// Reserve config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveConfig {
    /// Name
    pub name: String,
    /// Reserve type
    pub reserve_type: ReserveType,
    /// Status
    pub status: ReserveStatus,
    /// Max species
    pub max_species: usize,
}

impl ReserveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            reserve_type: ReserveType::Nature,
            status: ReserveStatus::Protected,
            max_species: 100,
        }
    }

    /// Set type
    pub fn reserve_type(mut self, rt: ReserveType) -> Self {
        self.reserve_type = rt;
        self
    }

    /// Set status
    pub fn status(mut self, s: ReserveStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max species
    pub fn max_species(mut self, max: usize) -> Self {
        self.max_species = max;
        self
    }
}

impl Default for ReserveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
