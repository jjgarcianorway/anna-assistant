//! Review artifact (v0.0.222).

use serde::{Deserialize, Serialize};

use crate::teams::Team;

use super::inputs::ReviewInputsSummary;
use super::issue::{ReviewIssue, ReviewRevision};
use super::types::{ReviewDecision, ReviewSeverity, ReviewerType};

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
        self.issues
            .iter()
            .any(|i| i.severity == ReviewSeverity::Blocker)
    }

    /// Get count of issues by severity
    pub fn issue_count(&self, severity: ReviewSeverity) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .count()
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
