//! Guard types (v0.0.194).

use crate::claims::Claim;
use serde::{Deserialize, Serialize};

/// Result of verifying a single claim against evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum VerifyResult {
    /// Claim matches evidence exactly
    Verified,
    /// Claim contradicts evidence
    Contradiction {
        /// What the claim stated (deterministic format)
        claimed: String,
        /// What the evidence shows (deterministic format)
        evidence: String,
    },
    /// No evidence available to verify this claim
    Unverifiable,
}

impl VerifyResult {
    /// Check if this is a contradiction
    pub fn is_contradiction(&self) -> bool {
        matches!(self, Self::Contradiction { .. })
    }

    /// Check if this is unverifiable
    pub fn is_unverifiable(&self) -> bool {
        matches!(self, Self::Unverifiable)
    }
}

/// GUARD verification report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardReport {
    /// Total specific claims checked
    pub total_specific_claims: u32,
    /// Number of claims that contradict evidence
    pub contradictions: u32,
    /// Number of specific claims that couldn't be verified
    pub unverifiable_specifics: u32,
    /// Whether invention was detected (triggers CHAOS ceiling)
    pub invention_detected: bool,
    /// Per-claim verification details (same order as input claims)
    pub details: Vec<GuardItem>,
}

/// Single claim verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardItem {
    /// The claim that was checked
    pub claim: Claim,
    /// Verification result
    pub result: VerifyResult,
}
