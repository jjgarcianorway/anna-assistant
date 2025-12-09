//! CommsGenerator struct (v0.0.254).
//!
//! Internal communications helper for Service Desk Theatre.
//! Uses roster and dialogue systems to create authentic messages.
//!
//! v0.0.254: Added query context and model for LLM-generated dialogue.

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
    /// v0.0.254: User query for context
    pub(crate) query: String,
    /// v0.0.254: Model to use for dialogue generation (None = static only)
    pub(crate) dialogue_model: Option<String>,
}

impl CommsGenerator {
    /// Create a new comms generator for a request
    pub fn new(team: Team, case_id: &str) -> Self {
        Self {
            team,
            case_id: case_id.to_string(),
            seed: seed_from_str(case_id),
            probes_planned: 0,
            query: String::new(),
            dialogue_model: None,
        }
    }

    /// v0.0.254: Set the query for context-aware dialogue
    pub fn with_query(mut self, query: &str) -> Self {
        self.query = query.to_string();
        self
    }

    /// v0.0.254: Set the model for LLM-generated dialogue
    pub fn with_model(mut self, model: &str) -> Self {
        self.dialogue_model = Some(model.to_string());
        self
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
