//! Tests for revision module (v0.0.208).

#[cfg(test)]
mod tests {
    use crate::revision::{
        junior_to_review_artifact, senior_to_review_artifact, JuniorVerification,
        RevisionInstruction, RevisionIssue, SeniorEscalation,
    };
    use crate::teams::Team;

    #[test]
    fn test_revision_issue_display() {
        assert_eq!(
            RevisionIssue::MissingEvidence.to_string(),
            "missing evidence"
        );
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
        let inst = RevisionInstruction::none().with_issue(RevisionIssue::MissingEvidence);
        let v = JuniorVerification::needs_revision(65, inst);
        assert!(!v.verified);
        assert_eq!(v.score, 65);
        assert!(v.instruction.has_changes());
    }

    #[test]
    fn test_senior_escalation_success() {
        let inst = RevisionInstruction::none().with_required_claim("add specific memory usage");
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
        let inst = RevisionInstruction::none().with_required_claim("include memory usage");
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
