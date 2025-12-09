//! Review inputs summary (v0.0.222).

use serde::{Deserialize, Serialize};

use crate::trace::SpecialistOutcome;

/// Stable summary of inputs used for review (for traceability)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewInputsSummary {
    /// Reliability score used
    pub score: u8,
    /// Grounding ratio from ANCHOR
    pub grounding_ratio: f32,
    /// Total claims extracted
    pub total_claims: u32,
    /// Whether invention was detected (GUARD)
    pub invention_detected: bool,
    /// Whether evidence was required
    pub evidence_required: bool,
    /// Specialist outcome from trace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specialist_outcome: Option<SpecialistOutcome>,
    /// Whether deterministic fallback was used
    pub fallback_used: bool,
}

impl ReviewInputsSummary {
    /// Create summary from values
    pub fn new(score: u8, grounding_ratio: f32, total_claims: u32) -> Self {
        Self {
            score,
            grounding_ratio,
            total_claims,
            ..Default::default()
        }
    }

    /// Set invention_detected
    pub fn with_invention(mut self, detected: bool) -> Self {
        self.invention_detected = detected;
        self
    }

    /// Set evidence_required
    pub fn with_evidence_required(mut self, required: bool) -> Self {
        self.evidence_required = required;
        self
    }
}
