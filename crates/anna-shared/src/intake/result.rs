//! Intake result types (v0.0.180).

use serde::{Deserialize, Serialize};

use crate::facts::FactKey;
use crate::rpc::{QueryIntent, SpecialistDomain};

use super::question::ClarificationQuestion;

/// Result of intake analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntakeResult {
    /// The classified intent
    pub intent: QueryIntent,
    /// The target domain
    pub domain: SpecialistDomain,
    /// Clarifications needed before proceeding
    pub clarifications_needed: Vec<ClarificationQuestion>,
    /// Facts that were already known (from FactsStore)
    pub facts_used: Vec<FactKey>,
    /// Whether intake can proceed without clarification
    pub can_proceed: bool,
    /// Confidence in the classification (0.0-1.0)
    pub confidence: f32,
}

impl IntakeResult {
    /// Create a result that can proceed without clarification
    pub fn proceed(intent: QueryIntent, domain: SpecialistDomain) -> Self {
        Self {
            intent,
            domain,
            clarifications_needed: vec![],
            facts_used: vec![],
            can_proceed: true,
            confidence: 1.0,
        }
    }

    /// Create a result that needs clarification
    pub fn needs_clarification(
        intent: QueryIntent,
        domain: SpecialistDomain,
        clarifications: Vec<ClarificationQuestion>,
    ) -> Self {
        Self {
            intent,
            domain,
            clarifications_needed: clarifications,
            facts_used: vec![],
            can_proceed: false,
            confidence: 0.5,
        }
    }
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed
    pub verified: bool,
    /// The verified value (if successful)
    pub value: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Alternative options found (if verification failed)
    pub alternatives: Vec<String>,
    /// Source of verification
    pub source: String,
}

impl VerificationResult {
    /// Create a successful verification
    pub fn success(value: String, source: &str) -> Self {
        Self {
            verified: true,
            value: Some(value),
            error: None,
            alternatives: vec![],
            source: source.to_string(),
        }
    }

    /// Create a failed verification with alternatives
    pub fn failed_with_alternatives(error: &str, alternatives: Vec<String>, source: &str) -> Self {
        Self {
            verified: false,
            value: None,
            error: Some(error.to_string()),
            alternatives,
            source: source.to_string(),
        }
    }

    /// Create a simple failure
    pub fn failed(error: &str, source: &str) -> Self {
        Self {
            verified: false,
            value: None,
            error: Some(error.to_string()),
            alternatives: vec![],
            source: source.to_string(),
        }
    }
}
