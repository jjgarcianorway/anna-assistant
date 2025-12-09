//! Review type enums (v0.0.222).

use serde::{Deserialize, Serialize};

/// Decision from review gate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ReviewDecision {
    /// Answer passes review, can publish
    Accept,
    /// Answer needs revision, generate correction
    #[default]
    Revise,
    /// Escalate to senior reviewer
    EscalateToSenior,
    /// Need user clarification (only when evidence truly missing)
    ClarifyUser,
}

impl std::fmt::Display for ReviewDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "accept"),
            Self::Revise => write!(f, "revise"),
            Self::EscalateToSenior => write!(f, "escalate"),
            Self::ClarifyUser => write!(f, "clarify"),
        }
    }
}

/// Who performed the review
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ReviewerType {
    /// Pure deterministic logic (no LLM)
    #[default]
    Deterministic,
    /// Team junior reviewer (LLM)
    Junior,
    /// Team senior reviewer (LLM)
    Senior,
}

impl std::fmt::Display for ReviewerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deterministic => write!(f, "deterministic"),
            Self::Junior => write!(f, "junior"),
            Self::Senior => write!(f, "senior"),
        }
    }
}

/// Severity level of a review issue.
/// Pinned ordering: Info < Warning < Blocker
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ReviewSeverity {
    /// Informational note (does not block)
    #[default]
    Info,
    /// Warning (should be addressed but doesn't block)
    Warning,
    /// Blocker (must be fixed before publish)
    Blocker,
}

impl std::fmt::Display for ReviewSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Blocker => write!(f, "blocker"),
        }
    }
}

/// Categories of review issues.
/// Each kind maps to specific remediation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewIssueKind {
    /// Answer lacks required evidence for claims
    MissingEvidence,
    /// Answer contains specifics that cannot be verified
    UnverifiableSpecifics,
    /// Answer contradicts collected evidence
    Contradiction,
    /// Answer suggests risky action without safeguards
    RiskyAction,
    /// Answer contains non-deterministic claims
    NonDeterministicClaim,
    /// Answer requires user clarification
    NeedsClarification,
    /// Answer is too vague for the domain
    TooVague,
    /// Answer format doesn't match expected output
    FormatIssue,
    /// Other issue not covered by specific kinds
    Other,
}

impl std::fmt::Display for ReviewIssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidence => write!(f, "missing_evidence"),
            Self::UnverifiableSpecifics => write!(f, "unverifiable_specifics"),
            Self::Contradiction => write!(f, "contradiction"),
            Self::RiskyAction => write!(f, "risky_action"),
            Self::NonDeterministicClaim => write!(f, "non_deterministic_claim"),
            Self::NeedsClarification => write!(f, "needs_clarification"),
            Self::TooVague => write!(f, "too_vague"),
            Self::FormatIssue => write!(f, "format_issue"),
            Self::Other => write!(f, "other"),
        }
    }
}
