// v0.0.753: Settings Precinct Config (Phase 329)
// Configuration for precincts

use serde::{Deserialize, Serialize};
use super::types::{PrecinctType, PrecinctStatus};

/// Precinct config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctConfig {
    /// Name
    pub name: String,
    /// Precinct type
    pub precinct_type: PrecinctType,
    /// Status
    pub status: PrecinctStatus,
    /// Max ballots
    pub max_ballots: usize,
}

impl PrecinctConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            precinct_type: PrecinctType::Voting,
            status: PrecinctStatus::Designated,
            max_ballots: 100,
        }
    }

    /// Set type
    pub fn precinct_type(mut self, pt: PrecinctType) -> Self {
        self.precinct_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PrecinctStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max ballots
    pub fn max_ballots(mut self, max: usize) -> Self {
        self.max_ballots = max;
        self
    }
}

impl Default for PrecinctConfig {
    fn default() -> Self {
        Self::new("default")
    }
}
