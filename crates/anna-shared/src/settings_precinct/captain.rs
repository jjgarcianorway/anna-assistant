// v0.0.753: Settings Precinct Captain (Phase 329)
// Captain management for precincts

use serde::{Deserialize, Serialize};

/// Precinct captain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecinctCaptain {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Ballot ID
    pub ballot_id: String,
}

impl PrecinctCaptain {
    /// Create new captain
    pub fn new(key: impl Into<String>, name: impl Into<String>, ballot_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ballot_id: ballot_id.into(),
        }
    }
}
