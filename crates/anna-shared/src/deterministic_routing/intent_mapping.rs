//! Intent Mapping - Individual mapping entry for an intent.
//!
//! Part of the Deterministic Intent Map (v0.0.439).

use super::intent_schema::{CanonicalIntent, Department};

/// Mapping entry for an intent.
#[derive(Debug, Clone)]
pub struct IntentMapping {
    /// The canonical intent.
    pub intent: CanonicalIntent,
    /// Department that owns this intent.
    pub department: Department,
    /// Required probes (must succeed for direct answer).
    pub required_probes: Vec<&'static str>,
    /// Optional probes (nice to have).
    pub optional_probes: Vec<&'static str>,
    /// Whether this intent can be answered directly from probes.
    pub can_answer_from_probes: bool,
    /// Description of what this intent covers.
    pub description: &'static str,
}

impl IntentMapping {
    /// Create a new mapping.
    pub const fn new(
        intent: CanonicalIntent,
        department: Department,
        required: &'static [&'static str],
        optional: &'static [&'static str],
        direct_answer: bool,
        desc: &'static str,
    ) -> Self {
        Self {
            intent,
            department,
            required_probes: Vec::new(), // Will be populated in build
            optional_probes: Vec::new(), // Will be populated in build
            can_answer_from_probes: direct_answer,
            description: desc,
        }
    }
}
