//! Team-aware review protocol for Junior/Senior specialists.
//!
//! ReviewArtifact is the unified schema for all team reviews.
//! Supports both deterministic gate reviews and LLM escalation reviews.
//!
//! Pinned ordering for deterministic serialization.

use crate::teams::Team;
use crate::trace::SpecialistOutcome;
use serde::{Deserialize, Serialize};

/// Decision from review gate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    /// Answer passes review, can publish
    Accept,
    /// Answer needs revision, generate correction
    Revise,
    /// Escalate to senior reviewer
    EscalateToSenior,
    /// Need user clarification (only when evidence truly missing)
    ClarifyUser,
}

impl Default for ReviewDecision {
    fn default() -> Self {
        Self::Revise
    }
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
pub enum ReviewerType {
    /// Pure deterministic logic (no LLM)
    Deterministic,
    /// Team junior reviewer (LLM)
    Junior,
    /// Team senior reviewer (LLM)
    Senior,
}

impl Default for ReviewerType {
    fn default() -> Self {
        Self::Deterministic
    }
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

/// Severity level of a review issue.
/// Pinned ordering: Info < Warning < Blocker
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    /// Informational note (does not block)
    Info,
    /// Warning (should be addressed but doesn't block)
    Warning,
    /// Blocker (must be fixed before publish)
    Blocker,
}

impl Default for ReviewSeverity {
    fn default() -> Self {
        Self::Info
    }
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
    pub fn new(severity: ReviewSeverity, kind: ReviewIssueKind, message: impl Into<String>) -> Self {
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

/// Unified review artifact from team specialists.
/// Used by both Junior (gate) and Senior (escalation) reviewers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewArtifact {
    /// Team that performed the review
    pub team: Team,
    /// Reviewer level ("junior" or "senior")
    pub reviewer: String,
    /// Confidence in the review (0.0-1.0)
    pub confidence: f32,
    /// Issues identified
    pub issues: Vec<ReviewIssue>,
    /// Revision instructions
    pub revisions: Vec<ReviewRevision>,
    /// Whether answer can be published (no Blockers)
    pub allow_publish: bool,
    /// Reliability score from deterministic gate (0-100)
    pub score: u8,
    /// Decision from review gate (v0.0.26)
    #[serde(default)]
    pub decision: ReviewDecision,
    /// Who performed the review (v0.0.26)
    #[serde(default)]
    pub reviewer_type: ReviewerType,
    /// Summary of inputs used for this review (v0.0.26)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_summary: Option<ReviewInputsSummary>,
}

impl ReviewArtifact {
    /// Create a new review artifact
    pub fn new(team: Team, reviewer: impl Into<String>) -> Self {
        Self {
            team,
            reviewer: reviewer.into(),
            confidence: 0.0,
            issues: Vec::new(),
            revisions: Vec::new(),
            allow_publish: false,
            score: 0,
            decision: ReviewDecision::Revise,
            reviewer_type: ReviewerType::Deterministic,
            inputs_summary: None,
        }
    }

    /// Create a passing review (no issues, allow publish)
    pub fn pass(team: Team, reviewer: impl Into<String>, score: u8) -> Self {
        Self {
            team,
            reviewer: reviewer.into(),
            confidence: 1.0,
            issues: Vec::new(),
            revisions: Vec::new(),
            allow_publish: true,
            score,
            decision: ReviewDecision::Accept,
            reviewer_type: ReviewerType::Deterministic,
            inputs_summary: None,
        }
    }

    /// Set decision
    pub fn with_decision(mut self, decision: ReviewDecision) -> Self {
        self.decision = decision;
        self
    }

    /// Set reviewer type
    pub fn with_reviewer_type(mut self, reviewer_type: ReviewerType) -> Self {
        self.reviewer_type = reviewer_type;
        self
    }

    /// Set inputs summary
    pub fn with_inputs_summary(mut self, summary: ReviewInputsSummary) -> Self {
        self.inputs_summary = Some(summary);
        self
    }

    /// Set confidence level
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set reliability score
    pub fn with_score(mut self, score: u8) -> Self {
        self.score = score;
        self
    }

    /// Add an issue
    pub fn with_issue(mut self, issue: ReviewIssue) -> Self {
        self.issues.push(issue);
        // Recalculate allow_publish
        self.allow_publish = !self.has_blockers();
        self
    }

    /// Add a revision instruction
    pub fn with_revision(mut self, revision: ReviewRevision) -> Self {
        self.revisions.push(revision);
        self
    }

    /// Check if any issues are blockers
    pub fn has_blockers(&self) -> bool {
        self.issues.iter().any(|i| i.severity == ReviewSeverity::Blocker)
    }

    /// Get count of issues by severity
    pub fn issue_count(&self, severity: ReviewSeverity) -> usize {
        self.issues.iter().filter(|i| i.severity == severity).count()
    }

    /// Get a summary of issues for transcript
    pub fn issues_summary(&self) -> Vec<String> {
        self.issues.iter().map(|i| i.message.clone()).collect()
    }

    /// Check if revisions are needed
    pub fn needs_revision(&self) -> bool {
        !self.revisions.is_empty()
    }
}

impl Default for ReviewArtifact {
    fn default() -> Self {
        Self::new(Team::General, "junior")
    }
}

impl ReviewInputsSummary {
    /// Create summary from values
    pub fn new(score: u8, grounding_ratio: f32, total_claims: u32) -> Self {
        Self { score, grounding_ratio, total_claims, ..Default::default() }
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
// Tests: tests/review_tests.rs
