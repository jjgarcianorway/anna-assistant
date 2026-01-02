//! Learning eligibility types (v0.0.427).
//!
//! Data structures for representing eligibility check results and skip reasons.

use serde::{Deserialize, Serialize};

/// Result of eligibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EligibilityResult {
    /// Whether learning is eligible
    pub eligible: bool,
    /// Reason if not eligible
    pub reason: Option<SkipReason>,
    /// Confidence in the decision
    pub confidence: f32,
    /// Suggested recipe ID if eligible
    pub suggested_id: Option<String>,
}

/// Reason for skipping learning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// Specialist returned error
    ErrorStatus,
    /// Confidence too low
    LowConfidence,
    /// Answer was vague or speculative
    VagueAnswer,
    /// No probe evidence
    NoEvidence,
    /// Too user-specific (hardcoded paths, etc.)
    TooSpecific,
    /// Unstable probes (conflicting results)
    UnstableProbes,
    /// Intent not generalizable
    NotGeneralizable,
    /// Similar recipe already exists
    DuplicateRecipe,
    /// Insufficient data to learn from
    InsufficientData,
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ErrorStatus => write!(f, "error_status"),
            Self::LowConfidence => write!(f, "low_confidence"),
            Self::VagueAnswer => write!(f, "vague_answer"),
            Self::NoEvidence => write!(f, "no_evidence"),
            Self::TooSpecific => write!(f, "too_specific"),
            Self::UnstableProbes => write!(f, "unstable_probes"),
            Self::NotGeneralizable => write!(f, "not_generalizable"),
            Self::DuplicateRecipe => write!(f, "duplicate_recipe"),
            Self::InsufficientData => write!(f, "insufficient_data"),
        }
    }
}
