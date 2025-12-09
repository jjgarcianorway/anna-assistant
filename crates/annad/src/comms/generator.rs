//! CommsGenerator struct (v0.0.192).
//!
//! Internal communications helper for Service Desk Theatre.
//! Uses roster and dialogue systems to create authentic messages.

use anna_shared::dialogue::seed_from_str;
use anna_shared::roster::{person_for, PersonProfile, Tier};
use anna_shared::teams::Team;

/// Generate internal comms at key pipeline stages
pub struct CommsGenerator {
    pub(crate) team: Team,
    pub(crate) case_id: String,
    pub(crate) seed: u64,
    /// Track how many probes were planned (for commentary)
    pub(crate) probes_planned: usize,
}

impl CommsGenerator {
    /// Create a new comms generator for a request
    pub fn new(team: Team, case_id: &str) -> Self {
        Self {
            team,
            case_id: case_id.to_string(),
            seed: seed_from_str(case_id),
            probes_planned: 0,
        }
    }

    /// Get the junior staff member for this team
    pub(crate) fn junior(&self) -> PersonProfile {
        person_for(self.team, Tier::Junior)
    }

    /// Get the senior staff member for this team
    pub(crate) fn senior(&self) -> PersonProfile {
        person_for(self.team, Tier::Senior)
    }
}
