//! Golden tests for review.rs types.

use anna_shared::review::{
    ReviewArtifact, ReviewDecision, ReviewInputsSummary, ReviewIssue, ReviewIssueKind,
    ReviewRevision, ReviewSeverity, ReviewerType,
};
use anna_shared::teams::Team;

// === Severity tests ===

#[test]
fn test_review_severity_ordering() {
    assert!(ReviewSeverity::Info < ReviewSeverity::Warning);
    assert!(ReviewSeverity::Warning < ReviewSeverity::Blocker);
}

#[test]
fn test_review_severity_display() {
    assert_eq!(ReviewSeverity::Info.to_string(), "info");
    assert_eq!(ReviewSeverity::Warning.to_string(), "warning");
    assert_eq!(ReviewSeverity::Blocker.to_string(), "blocker");
}

// === Issue kind tests ===

#[test]
fn test_review_issue_kind_display() {
    assert_eq!(
        ReviewIssueKind::MissingEvidence.to_string(),
        "missing_evidence"
    );
    assert_eq!(ReviewIssueKind::RiskyAction.to_string(), "risky_action");
}

// === ReviewIssue tests ===

#[test]
fn test_review_issue_builders() {
    let issue = ReviewIssue::blocker(ReviewIssueKind::MissingEvidence, "Need disk usage data")
        .with_evidence("Disk");

    assert_eq!(issue.severity, ReviewSeverity::Blocker);
    assert_eq!(issue.kind, ReviewIssueKind::MissingEvidence);
    assert_eq!(issue.evidence_needed.len(), 1);
}

// === ReviewRevision tests ===

#[test]
fn test_review_revision_builder() {
    let rev = ReviewRevision::new("add_disk_usage", "Include disk usage percentage")
        .with_add_claim("/ is 95% full")
        .with_remove_claim("disk might be full");

    assert_eq!(rev.template_id, "add_disk_usage");
    assert_eq!(rev.add_claims.len(), 1);
    assert_eq!(rev.remove_claims.len(), 1);
}

// === ReviewArtifact tests ===

#[test]
fn test_review_artifact_pass() {
    let artifact = ReviewArtifact::pass(Team::Storage, "junior", 85);

    assert!(artifact.allow_publish);
    assert!(!artifact.has_blockers());
    assert_eq!(artifact.score, 85);
    assert_eq!(artifact.team, Team::Storage);
}

#[test]
fn test_review_artifact_with_blocker() {
    let artifact = ReviewArtifact::new(Team::Storage, "junior").with_score(65).with_issue(
        ReviewIssue::blocker(ReviewIssueKind::MissingEvidence, "Disk usage data required"),
    );

    assert!(!artifact.allow_publish);
    assert!(artifact.has_blockers());
    assert_eq!(artifact.issue_count(ReviewSeverity::Blocker), 1);
}

#[test]
fn test_review_artifact_warnings_dont_block() {
    // Warning doesn't block - pass() starts with allow_publish=true
    let artifact = ReviewArtifact::pass(Team::Network, "senior", 75).with_issue(
        ReviewIssue::warning(ReviewIssueKind::TooVague, "Could include more detail"),
    );

    assert!(artifact.allow_publish);
    assert!(!artifact.has_blockers());
}

#[test]
fn test_review_artifact_issues_summary() {
    let artifact = ReviewArtifact::new(Team::Performance, "junior")
        .with_issue(ReviewIssue::warning(ReviewIssueKind::TooVague, "Issue 1"))
        .with_issue(ReviewIssue::info(ReviewIssueKind::Other, "Issue 2"));

    let summary = artifact.issues_summary();
    assert_eq!(summary.len(), 2);
    assert!(summary.contains(&"Issue 1".to_string()));
}

#[test]
fn test_review_artifact_serialization() {
    let artifact = ReviewArtifact::pass(Team::Storage, "junior", 90);
    let json = serde_json::to_string(&artifact).unwrap();
    let parsed: ReviewArtifact = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.team, Team::Storage);
    assert_eq!(parsed.reviewer, "junior");
    assert_eq!(parsed.score, 90);
    assert!(parsed.allow_publish);
}

#[test]
fn test_confidence_clamped() {
    let artifact = ReviewArtifact::new(Team::General, "junior").with_confidence(1.5); // Over max

    assert_eq!(artifact.confidence, 1.0);

    let artifact2 = ReviewArtifact::new(Team::General, "junior").with_confidence(-0.5); // Under min

    assert_eq!(artifact2.confidence, 0.0);
}

// === v0.0.26 tests: ReviewDecision, ReviewerType, ReviewInputsSummary ===

#[test]
fn test_review_decision_display() {
    assert_eq!(ReviewDecision::Accept.to_string(), "accept");
    assert_eq!(ReviewDecision::Revise.to_string(), "revise");
    assert_eq!(ReviewDecision::EscalateToSenior.to_string(), "escalate");
    assert_eq!(ReviewDecision::ClarifyUser.to_string(), "clarify");
}

#[test]
fn test_reviewer_type_display() {
    assert_eq!(ReviewerType::Deterministic.to_string(), "deterministic");
    assert_eq!(ReviewerType::Junior.to_string(), "junior");
    assert_eq!(ReviewerType::Senior.to_string(), "senior");
}

#[test]
fn test_review_artifact_with_decision() {
    let artifact = ReviewArtifact::new(Team::Storage, "junior")
        .with_decision(ReviewDecision::EscalateToSenior)
        .with_reviewer_type(ReviewerType::Deterministic);

    assert_eq!(artifact.decision, ReviewDecision::EscalateToSenior);
    assert_eq!(artifact.reviewer_type, ReviewerType::Deterministic);
}

#[test]
fn test_review_inputs_summary() {
    let summary = ReviewInputsSummary::new(85, 0.9, 3)
        .with_invention(false)
        .with_evidence_required(true);

    assert_eq!(summary.score, 85);
    assert_eq!(summary.grounding_ratio, 0.9);
    assert_eq!(summary.total_claims, 3);
    assert!(!summary.invention_detected);
    assert!(summary.evidence_required);
}

#[test]
fn test_review_artifact_with_inputs_summary() {
    let summary = ReviewInputsSummary::new(80, 0.8, 2);
    let artifact = ReviewArtifact::pass(Team::Network, "junior", 80).with_inputs_summary(summary);

    assert!(artifact.inputs_summary.is_some());
    assert_eq!(artifact.inputs_summary.unwrap().score, 80);
}

#[test]
fn test_review_decision_serialization() {
    let artifact = ReviewArtifact::new(Team::Security, "senior")
        .with_decision(ReviewDecision::Accept)
        .with_reviewer_type(ReviewerType::Senior)
        .with_score(90);

    let json = serde_json::to_string(&artifact).unwrap();
    let parsed: ReviewArtifact = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.decision, ReviewDecision::Accept);
    assert_eq!(parsed.reviewer_type, ReviewerType::Senior);
}

// === Determinism tests ===

#[test]
fn test_review_artifact_deterministic_construction() {
    let a1 = ReviewArtifact::pass(Team::Storage, "junior", 85);
    let a2 = ReviewArtifact::pass(Team::Storage, "junior", 85);

    assert_eq!(a1.team, a2.team);
    assert_eq!(a1.reviewer, a2.reviewer);
    assert_eq!(a1.score, a2.score);
    assert_eq!(a1.decision, a2.decision);
}

#[test]
fn test_review_inputs_summary_default() {
    let summary = ReviewInputsSummary::default();

    assert_eq!(summary.score, 0);
    assert_eq!(summary.grounding_ratio, 0.0);
    assert_eq!(summary.total_claims, 0);
    assert!(!summary.invention_detected);
}
