//! Evidence Binding Types - v0.0.437.
//!
//! Core type definitions for evidence binding.

use super::intent::Subject;
use serde::{Deserialize, Serialize};

/// A claim that is bound to evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundClaim {
    /// The claim text.
    pub text: String,
    /// Evidence IDs that support this claim.
    pub evidence_ids: Vec<String>,
    /// Field this claim corresponds to.
    pub field: String,
    /// Whether the binding is valid.
    pub binding_valid: bool,
}

impl BoundClaim {
    /// Create a new bound claim.
    pub fn new(text: &str, field: &str, evidence_ids: Vec<String>) -> Self {
        Self {
            text: text.to_string(),
            evidence_ids,
            field: field.to_string(),
            binding_valid: true,
        }
    }

    /// Check if claim has evidence.
    pub fn has_evidence(&self) -> bool {
        !self.evidence_ids.is_empty()
    }

    /// Mark as invalid binding.
    pub fn invalidate(&mut self, reason: &str) {
        self.binding_valid = false;
        self.text = format!("[Invalid: {}] {}", reason, self.text);
    }
}

/// Result of evidence binding.
#[derive(Debug, Clone)]
pub enum BindingResult {
    /// All claims are properly bound to evidence.
    Valid { claims: Vec<BoundClaim> },
    /// Some claims could not be bound.
    PartialBind {
        valid_claims: Vec<BoundClaim>,
        unbound_claims: Vec<String>,
    },
    /// Evidence exists but doesn't match the question.
    MismatchedEvidence { message: String, suggestion: String },
    /// No evidence at all.
    NoEvidence,
}

impl BindingResult {
    /// Check if binding is fully valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid { .. })
    }

    /// Get valid claims if any.
    pub fn valid_claims(&self) -> Vec<&BoundClaim> {
        match self {
            Self::Valid { claims } => claims.iter().collect(),
            Self::PartialBind { valid_claims, .. } => valid_claims.iter().collect(),
            _ => Vec::new(),
        }
    }

    /// Get the fallback message for mismatched evidence.
    pub fn fallback_message(&self) -> Option<String> {
        match self {
            Self::MismatchedEvidence {
                message,
                suggestion,
            } => Some(format!("{}\n\nSuggestion: {}", message, suggestion)),
            Self::NoEvidence => {
                Some("I could not find evidence to answer this question.".to_string())
            }
            Self::PartialBind { unbound_claims, .. } if !unbound_claims.is_empty() => {
                Some(format!("I could not verify: {}", unbound_claims.join(", ")))
            }
            _ => None,
        }
    }
}

/// An unbound claim from specialist output.
#[derive(Debug, Clone)]
pub struct UnboundClaim {
    /// The claim text.
    pub text: String,
    /// Field this claim corresponds to.
    pub field: String,
}

impl UnboundClaim {
    /// Create a new unbound claim.
    pub fn new(text: &str, field: &str) -> Self {
        Self {
            text: text.to_string(),
            field: field.to_string(),
        }
    }
}

/// An evidence item available for binding.
#[derive(Debug, Clone)]
pub struct EvidenceItem {
    /// Evidence ID.
    pub id: String,
    /// Subject this evidence relates to.
    pub subject: Subject,
    /// Fields this evidence provides.
    pub fields: Vec<String>,
    /// Brief summary.
    pub summary: String,
}

impl EvidenceItem {
    /// Create a new evidence item.
    pub fn new(id: &str, subject: Subject, fields: Vec<&str>, summary: &str) -> Self {
        Self {
            id: id.to_string(),
            subject,
            fields: fields.into_iter().map(String::from).collect(),
            summary: summary.to_string(),
        }
    }
}

/// Binding violation types.
#[derive(Debug, Clone)]
pub enum BindingViolation {
    /// Claim has no evidence support.
    NoEvidence { claim: String },
    /// Binding is marked invalid.
    InvalidBinding { claim: String },
    /// Evidence doesn't match subject.
    SubjectMismatch { expected: Subject, found: Subject },
}
