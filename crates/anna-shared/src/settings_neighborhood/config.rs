// v0.0.754: Settings Neighborhood (Phase 330)
// Neighborhood configuration

use serde::{Deserialize, Serialize};
use super::types::{NeighborhoodType, NeighborhoodStatus};

/// Neighborhood config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighborhoodConfig {
    /// Name
    pub name: String,
    /// Neighborhood type
    pub neighborhood_type: NeighborhoodType,
    /// Status
    pub status: NeighborhoodStatus,
    /// Max initiatives
    pub max_initiatives: usize,
}

impl NeighborhoodConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            neighborhood_type: NeighborhoodType::Residential,
            status: NeighborhoodStatus::Planned,
            max_initiatives: 100,
        }
    }

    /// Set type
    pub fn neighborhood_type(mut self, nt: NeighborhoodType) -> Self {
        self.neighborhood_type = nt;
        self
    }

    /// Set status
    pub fn status(mut self, s: NeighborhoodStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max initiatives
    pub fn max_initiatives(mut self, max: usize) -> Self {
        self.max_initiatives = max;
        self
    }
}

impl Default for NeighborhoodConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
