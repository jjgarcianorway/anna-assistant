//! Conversion helpers: Legacy types <-> ReviewArtifact (v0.0.208).

use crate::review::{ReviewArtifact, ReviewIssue, ReviewIssueKind, ReviewRevision, ReviewSeverity};
use crate::teams::Team;

use super::junior::JuniorVerification;
use super::senior::SeniorEscalation;
use super::types::RevisionIssue;

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
            RevisionIssue::UnverifiableClaims => (
                ReviewIssueKind::UnverifiableSpecifics,
                ReviewSeverity::Blocker,
            ),
            RevisionIssue::MissingProbes => {
                (ReviewIssueKind::MissingEvidence, ReviewSeverity::Warning)
            }
            RevisionIssue::OverConfident => (
                ReviewIssueKind::NonDeterministicClaim,
                ReviewSeverity::Warning,
            ),
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
            RevisionIssue::UnverifiableClaims => (
                ReviewIssueKind::UnverifiableSpecifics,
                ReviewSeverity::Warning,
            ),
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
