//! Claim types and definitions.

use serde::{Deserialize, Serialize};

/// A claim that needs to be validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim text.
    pub text: String,
    /// Type of claim.
    pub claim_type: ClaimType,
    /// Required evidence type.
    pub required_evidence: EvidenceType,
}

impl Claim {
    /// Create a new claim.
    pub fn new(text: &str, claim_type: ClaimType) -> Self {
        Self {
            text: text.to_string(),
            claim_type,
            required_evidence: claim_type.required_evidence(),
        }
    }

    /// Create a factual claim (needs probe evidence).
    pub fn factual(text: &str) -> Self {
        Self::new(text, ClaimType::Factual)
    }

    /// Create a documentation claim (needs doc evidence).
    pub fn documentation(text: &str) -> Self {
        Self::new(text, ClaimType::Documentation)
    }

    /// Create a recommendation claim (needs any evidence).
    pub fn recommendation(text: &str) -> Self {
        Self::new(text, ClaimType::Recommendation)
    }
}

/// Type of claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    /// Statement about current system state (needs probe evidence).
    Factual,
    /// Statement about how something works (needs doc evidence).
    Documentation,
    /// Suggestion or recommendation (needs some evidence).
    Recommendation,
    /// Acknowledgment of uncertainty (no evidence needed).
    Uncertainty,
}

impl ClaimType {
    /// Get required evidence type.
    pub fn required_evidence(&self) -> EvidenceType {
        match self {
            Self::Factual => EvidenceType::Probe,
            Self::Documentation => EvidenceType::Documentation,
            Self::Recommendation => EvidenceType::Any,
            Self::Uncertainty => EvidenceType::None,
        }
    }
}

/// Type of evidence required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Must have probe output.
    Probe,
    /// Must have documentation (man, help, wiki).
    Documentation,
    /// Any evidence type.
    Any,
    /// No evidence required.
    None,
}
