//! Review issue and revision types (v0.0.222).

use serde::{Deserialize, Serialize};

use super::types::{ReviewIssueKind, ReviewSeverity};

/// A specific issue identified during review.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewIssue {
    /// Severity of this issue
    pub severity: ReviewSeverity,
    /// Category of the issue
    pub kind: ReviewIssueKind,
    /// Human-readable description
    pub message: String,
    /// Evidence kinds needed to resolve (if applicable)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_needed: Vec<String>,
}

impl ReviewIssue {
    /// Create a new review issue
    pub fn new(
        severity: ReviewSeverity,
        kind: ReviewIssueKind,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            kind,
            message: message.into(),
            evidence_needed: Vec::new(),
        }
    }

    /// Create an info-level issue
    pub fn info(kind: ReviewIssueKind, message: impl Into<String>) -> Self {
        Self::new(ReviewSeverity::Info, kind, message)
    }

    /// Create a warning-level issue
    pub fn warning(kind: ReviewIssueKind, message: impl Into<String>) -> Self {
        Self::new(ReviewSeverity::Warning, kind, message)
    }

    /// Create a blocker issue
    pub fn blocker(kind: ReviewIssueKind, message: impl Into<String>) -> Self {
        Self::new(ReviewSeverity::Blocker, kind, message)
    }

    /// Add evidence needed to resolve this issue
    pub fn with_evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence_needed.push(evidence.into());
        self
    }
}

/// Revision instruction with template ID for determinism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewRevision {
    /// Template ID for deterministic application
    pub template_id: String,
    /// Human-readable instruction
    pub instruction: String,
    /// Claims to add (from evidence)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_claims: Vec<String>,
    /// Claims to remove (unverifiable)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove_claims: Vec<String>,
}

impl ReviewRevision {
    /// Create a new revision instruction
    pub fn new(template_id: impl Into<String>, instruction: impl Into<String>) -> Self {
        Self {
            template_id: template_id.into(),
            instruction: instruction.into(),
            add_claims: Vec::new(),
            remove_claims: Vec::new(),
        }
    }

    /// Add a claim to include
    pub fn with_add_claim(mut self, claim: impl Into<String>) -> Self {
        self.add_claims.push(claim.into());
        self
    }

    /// Add a claim to remove
    pub fn with_remove_claim(mut self, claim: impl Into<String>) -> Self {
        self.remove_claims.push(claim.into());
        self
    }
}
