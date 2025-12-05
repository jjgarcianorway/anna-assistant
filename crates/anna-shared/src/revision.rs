//! Revision instructions for junior/senior feedback.
//!
//! Junior and Senior reviewers return revision instructions (not direct answers)
//! to keep Anna as the visible owner of the response.
//!
//! Re-exports team-aware ReviewArtifact from the review module for gradual migration.

use serde::{Deserialize, Serialize};

// Re-export new team-aware review types
pub use crate::review::{
    ReviewArtifact, ReviewIssue, ReviewIssueKind, ReviewRevision, ReviewSeverity,
};
use crate::teams::Team;

/// Issue categories for revision instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionIssue {
    /// Answer lacks required evidence claims
    MissingEvidence,
    /// Answer contradicts collected evidence
    Contradiction,
    /// Answer is too vague or unspecific
    TooVague,
    /// Answer includes unverifiable claims
    UnverifiableClaims,
    /// Answer missing required probe data
    MissingProbes,
    /// Answer exceeds confidence bounds
    OverConfident,
    /// Answer format needs improvement
    FormatIssue,
}

impl std::fmt::Display for RevisionIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidence => write!(f, "missing evidence"),
            Self::Contradiction => write!(f, "contradiction"),
            Self::TooVague => write!(f, "too vague"),
            Self::UnverifiableClaims => write!(f, "unverifiable claims"),
            Self::MissingProbes => write!(f, "missing probes"),
            Self::OverConfident => write!(f, "over-confident"),
            Self::FormatIssue => write!(f, "format issue"),
        }
    }
}

/// Revision instruction from junior or senior reviewer
///
/// Contains structured feedback for Anna to apply deterministically:
/// - Issues identified in the current answer
/// - Claims that must be included (with evidence refs)
/// - Claims that must be removed (unverifiable)
/// - Additional probes to run if evidence is missing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RevisionInstruction {
    /// Issues identified (templated categories)
    pub issues: Vec<RevisionIssue>,
    /// Claims that must be included (e.g., "include / percent_used")
    pub required_claims: Vec<String>,
    /// Claims that must be removed (unverifiable)
    pub forbidden_claims: Vec<String>,
    /// Additional probes to run if evidence_required and probes missing
    pub recommended_probes: Vec<String>,
    /// Free-form explanation for transcript (not used for logic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl RevisionInstruction {
    /// Create an empty instruction (no changes needed)
    pub fn none() -> Self {
        Self::default()
    }

    /// Check if this instruction requires any changes
    pub fn has_changes(&self) -> bool {
        !self.issues.is_empty()
            || !self.required_claims.is_empty()
            || !self.forbidden_claims.is_empty()
            || !self.recommended_probes.is_empty()
    }

    /// Add an issue
    pub fn with_issue(mut self, issue: RevisionIssue) -> Self {
        if !self.issues.contains(&issue) {
            self.issues.push(issue);
        }
        self
    }

    /// Add a required claim
    pub fn with_required_claim(mut self, claim: impl Into<String>) -> Self {
        self.required_claims.push(claim.into());
        self
    }

    /// Add a forbidden claim
    pub fn with_forbidden_claim(mut self, claim: impl Into<String>) -> Self {
        self.forbidden_claims.push(claim.into());
        self
    }

    /// Add a recommended probe
    pub fn with_recommended_probe(mut self, probe: impl Into<String>) -> Self {
        self.recommended_probes.push(probe.into());
        self
    }

    /// Add explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }

    /// Format as a concise summary for transcript
    pub fn summary(&self) -> String {
        if !self.has_changes() {
            return "no changes needed".to_string();
        }

        let mut parts = Vec::new();

        if !self.issues.is_empty() {
            let issues: Vec<_> = self.issues.iter().map(|i| i.to_string()).collect();
            parts.push(format!("issues=[{}]", issues.join(", ")));
        }

        if !self.required_claims.is_empty() {
            parts.push(format!("required_claims=[{}]", self.required_claims.join(", ")));
        }

        if !self.forbidden_claims.is_empty() {
            parts.push(format!("forbidden_claims=[{}]", self.forbidden_claims.join(", ")));
        }

        if !self.recommended_probes.is_empty() {
            parts.push(format!("recommended_probes=[{}]", self.recommended_probes.join(", ")));
        }

        parts.join(" ")
    }
}

/// Junior verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuniorVerification {
    /// Reliability score (0-100)
    pub score: u8,
    /// Whether the answer meets the threshold
    pub verified: bool,
    /// Revision instructions if not verified
    pub instruction: RevisionInstruction,
}

impl JuniorVerification {
    /// Create a verified result (score meets threshold)
    pub fn verified(score: u8) -> Self {
        Self {
            score,
            verified: true,
            instruction: RevisionInstruction::none(),
        }
    }

    /// Create a result requiring revision
    pub fn needs_revision(score: u8, instruction: RevisionInstruction) -> Self {
        Self {
            score,
            verified: false,
            instruction,
        }
    }
}

/// Senior escalation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeniorEscalation {
    /// Whether senior was able to provide useful guidance
    pub successful: bool,
    /// Revision instructions from senior
    pub instruction: RevisionInstruction,
    /// Optional explanation for why escalation was needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SeniorEscalation {
    /// Create a successful escalation with instructions
    pub fn success(instruction: RevisionInstruction) -> Self {
        Self {
            successful: true,
            instruction,
            reason: None,
        }
    }

    /// Create a failed escalation (senior couldn't help)
    pub fn failed(reason: impl Into<String>) -> Self {
        Self {
            successful: false,
            instruction: RevisionInstruction::none(),
            reason: Some(reason.into()),
        }
    }
}

// =============================================================================
// Conversion helpers: Legacy types <-> ReviewArtifact
// =============================================================================

/// Convert legacy JuniorVerification to ReviewArtifact
pub fn junior_to_review_artifact(verification: &JuniorVerification, team: Team) -> ReviewArtifact {
    let mut artifact = ReviewArtifact::new(team, "junior")
        .with_score(verification.score)
        .with_confidence(if verification.verified { 1.0 } else { 0.5 });

    // Map legacy RevisionIssue to ReviewIssue
    for issue in &verification.instruction.issues {
        let (kind, severity) = match issue {
            RevisionIssue::MissingEvidence => {
                (ReviewIssueKind::MissingEvidence, ReviewSeverity::Blocker)
            }
            RevisionIssue::Contradiction => {
                (ReviewIssueKind::Contradiction, ReviewSeverity::Blocker)
            }
            RevisionIssue::TooVague => (ReviewIssueKind::TooVague, ReviewSeverity::Warning),
            RevisionIssue::UnverifiableClaims => {
                (ReviewIssueKind::UnverifiableSpecifics, ReviewSeverity::Blocker)
            }
            RevisionIssue::MissingProbes => {
                (ReviewIssueKind::MissingEvidence, ReviewSeverity::Warning)
            }
            RevisionIssue::OverConfident => {
                (ReviewIssueKind::NonDeterministicClaim, ReviewSeverity::Warning)
            }
            RevisionIssue::FormatIssue => (ReviewIssueKind::FormatIssue, ReviewSeverity::Info),
        };
        artifact = artifact.with_issue(ReviewIssue::new(severity, kind, issue.to_string()));
    }

    // Add revisions from required/forbidden claims
    if !verification.instruction.required_claims.is_empty()
        || !verification.instruction.forbidden_claims.is_empty()
    {
        let mut revision = ReviewRevision::new("legacy_revision", "Apply revision changes");
        for claim in &verification.instruction.required_claims {
            revision = revision.with_add_claim(claim);
        }
        for claim in &verification.instruction.forbidden_claims {
            revision = revision.with_remove_claim(claim);
        }
        artifact = artifact.with_revision(revision);
    }

    // Set allow_publish based on verification result and blockers
    artifact.allow_publish = verification.verified && !artifact.has_blockers();

    artifact
}

/// Convert legacy SeniorEscalation to ReviewArtifact
pub fn senior_to_review_artifact(escalation: &SeniorEscalation, team: Team) -> ReviewArtifact {
    let mut artifact = ReviewArtifact::new(team, "senior")
        .with_confidence(if escalation.successful { 0.8 } else { 0.3 });

    // Map issues from escalation instruction
    for issue in &escalation.instruction.issues {
        let (kind, severity) = match issue {
            RevisionIssue::MissingEvidence => {
                (ReviewIssueKind::MissingEvidence, ReviewSeverity::Warning)
            }
            RevisionIssue::Contradiction => {
                (ReviewIssueKind::Contradiction, ReviewSeverity::Warning)
            }
            RevisionIssue::TooVague => (ReviewIssueKind::TooVague, ReviewSeverity::Info),
            RevisionIssue::UnverifiableClaims => {
                (ReviewIssueKind::UnverifiableSpecifics, ReviewSeverity::Warning)
            }
            RevisionIssue::MissingProbes => {
                (ReviewIssueKind::MissingEvidence, ReviewSeverity::Info)
            }
            RevisionIssue::OverConfident => {
                (ReviewIssueKind::NonDeterministicClaim, ReviewSeverity::Info)
            }
            RevisionIssue::FormatIssue => (ReviewIssueKind::FormatIssue, ReviewSeverity::Info),
        };
        artifact = artifact.with_issue(ReviewIssue::new(severity, kind, issue.to_string()));
    }

    // Add revisions
    if !escalation.instruction.required_claims.is_empty()
        || !escalation.instruction.forbidden_claims.is_empty()
    {
        let mut revision = ReviewRevision::new("senior_guidance", "Senior guidance applied");
        for claim in &escalation.instruction.required_claims {
            revision = revision.with_add_claim(claim);
        }
        for claim in &escalation.instruction.forbidden_claims {
            revision = revision.with_remove_claim(claim);
        }
        artifact = artifact.with_revision(revision);
    }

    artifact.allow_publish = escalation.successful;

    artifact
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revision_issue_display() {
        assert_eq!(RevisionIssue::MissingEvidence.to_string(), "missing evidence");
        assert_eq!(RevisionIssue::Contradiction.to_string(), "contradiction");
        assert_eq!(RevisionIssue::TooVague.to_string(), "too vague");
    }

    #[test]
    fn test_empty_instruction_has_no_changes() {
        let inst = RevisionInstruction::none();
        assert!(!inst.has_changes());
        assert_eq!(inst.summary(), "no changes needed");
    }

    #[test]
    fn test_instruction_with_issues() {
        let inst = RevisionInstruction::none()
            .with_issue(RevisionIssue::MissingEvidence)
            .with_required_claim("/ is 95% full");

        assert!(inst.has_changes());
        assert!(inst.summary().contains("missing evidence"));
        assert!(inst.summary().contains("required_claims"));
    }

    #[test]
    fn test_junior_verification_verified() {
        let v = JuniorVerification::verified(85);
        assert!(v.verified);
        assert_eq!(v.score, 85);
        assert!(!v.instruction.has_changes());
    }

    #[test]
    fn test_junior_verification_needs_revision() {
        let inst = RevisionInstruction::none()
            .with_issue(RevisionIssue::MissingEvidence);
        let v = JuniorVerification::needs_revision(65, inst);
        assert!(!v.verified);
        assert_eq!(v.score, 65);
        assert!(v.instruction.has_changes());
    }

    #[test]
    fn test_senior_escalation_success() {
        let inst = RevisionInstruction::none()
            .with_required_claim("add specific memory usage");
        let e = SeniorEscalation::success(inst);
        assert!(e.successful);
        assert!(e.instruction.has_changes());
    }

    #[test]
    fn test_senior_escalation_failed() {
        let e = SeniorEscalation::failed("insufficient evidence to improve");
        assert!(!e.successful);
        assert!(!e.instruction.has_changes());
        assert!(e.reason.is_some());
    }

    #[test]
    fn test_instruction_builder_deduplicates_issues() {
        let inst = RevisionInstruction::none()
            .with_issue(RevisionIssue::MissingEvidence)
            .with_issue(RevisionIssue::MissingEvidence);

        assert_eq!(inst.issues.len(), 1);
    }

    #[test]
    fn test_junior_to_review_artifact_verified() {
        let junior = JuniorVerification::verified(85);
        let artifact = junior_to_review_artifact(&junior, Team::Storage);

        assert!(artifact.allow_publish);
        assert_eq!(artifact.score, 85);
        assert_eq!(artifact.team, Team::Storage);
        assert!(artifact.issues.is_empty());
    }

    #[test]
    fn test_junior_to_review_artifact_needs_revision() {
        let inst = RevisionInstruction::none()
            .with_issue(RevisionIssue::MissingEvidence)
            .with_required_claim("disk is 95% full");
        let junior = JuniorVerification::needs_revision(65, inst);
        let artifact = junior_to_review_artifact(&junior, Team::Storage);

        assert!(!artifact.allow_publish);
        assert_eq!(artifact.score, 65);
        assert!(artifact.has_blockers()); // MissingEvidence maps to blocker
    }

    #[test]
    fn test_senior_to_review_artifact_successful() {
        let inst = RevisionInstruction::none()
            .with_required_claim("include memory usage");
        let senior = SeniorEscalation::success(inst);
        let artifact = senior_to_review_artifact(&senior, Team::Performance);

        assert!(artifact.allow_publish);
        assert_eq!(artifact.team, Team::Performance);
        assert_eq!(artifact.reviewer, "senior");
        assert!(artifact.needs_revision());
    }

    #[test]
    fn test_senior_to_review_artifact_failed() {
        let senior = SeniorEscalation::failed("insufficient evidence");
        let artifact = senior_to_review_artifact(&senior, Team::Network);

        assert!(!artifact.allow_publish);
        assert_eq!(artifact.confidence, 0.3);
    }
}
