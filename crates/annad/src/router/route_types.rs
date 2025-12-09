//! Route types and structs (v0.0.172).

use anna_shared::probe_spine::RouteCapability;
use anna_shared::rpc::{QueryIntent, SpecialistDomain};

use super::QueryClass;

/// Route result from deterministic router
#[derive(Debug, Clone)]
pub struct DeterministicRoute {
    pub class: QueryClass,
    pub domain: SpecialistDomain,
    pub intent: QueryIntent,
    pub probes: Vec<String>,
    pub capability: RouteCapability,
}

impl DeterministicRoute {
    /// Legacy accessor for can_answer_deterministically
    pub fn can_answer_deterministically(&self) -> bool {
        self.capability.can_answer_deterministically
    }
}
