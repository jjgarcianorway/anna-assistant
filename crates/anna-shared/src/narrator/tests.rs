//! Tests for narrator module (v0.0.218).

#[cfg(test)]
mod tests {
    use crate::narrator::{
        format_issues_list, get_person, it_confidence, it_domain_context, it_greeting,
        narrate_escalation, narrate_person_action, narrate_person_escalation,
        narrate_person_review, narrate_review_result, narrate_team_action,
        narrate_ticket_assignment, reviewer_badge, status_indicator, team_role_name, team_tag,
    };
    use crate::review::{ReviewArtifact, ReviewIssue, ReviewIssueKind};
    use crate::teams::Team;

    #[test]
    fn test_team_role_names() {
        assert_eq!(team_role_name(Team::Storage, "junior"), "Storage Engineer");
        assert_eq!(team_role_name(Team::Storage, "senior"), "Storage Architect");
        assert_eq!(
            team_role_name(Team::Desktop, "junior"),
            "Desktop Administrator"
        );
        assert_eq!(
            team_role_name(Team::Security, "senior"),
            "Security Engineer"
        );
    }

    #[test]
    fn test_team_tags() {
        assert_eq!(team_tag(Team::Storage), "storage");
        assert_eq!(team_tag(Team::Performance), "perf");
        assert_eq!(team_tag(Team::General), "general");
    }

    #[test]
    fn test_narrate_team_action() {
        let result = narrate_team_action(Team::Storage, "junior", "reviewing disk usage claims");
        assert_eq!(result, "Storage Engineer reviewing disk usage claims");
    }

    #[test]
    fn test_narrate_review_result_approved() {
        let artifact = ReviewArtifact::pass(Team::Network, "junior", 85);
        let result = narrate_review_result(&artifact);
        assert!(result.contains("Network Engineer"));
        assert!(result.contains("approved"));
        assert!(result.contains("85"));
    }

    #[test]
    fn test_narrate_review_result_with_warnings() {
        let artifact = ReviewArtifact::pass(Team::Performance, "senior", 82)
            .with_issue(ReviewIssue::warning(ReviewIssueKind::TooVague, "test"));
        let result = narrate_review_result(&artifact);
        assert!(result.contains("Performance Engineer"));
        assert!(result.contains("approved"));
        assert!(result.contains("1 warning"));
    }

    #[test]
    fn test_narrate_review_result_needs_revision() {
        let artifact = ReviewArtifact::new(Team::Storage, "junior")
            .with_score(65)
            .with_issue(ReviewIssue::blocker(
                ReviewIssueKind::MissingEvidence,
                "need disk data",
            ));
        let result = narrate_review_result(&artifact);
        assert!(result.contains("Storage Engineer"));
        assert!(result.contains("needs revision"));
        assert!(result.contains("1 blocker"));
    }

    #[test]
    fn test_narrate_escalation() {
        let result = narrate_escalation(Team::Network, "cannot verify DNS configuration");
        assert!(result.contains("Network Architect"));
        assert!(result.contains("cannot verify DNS configuration"));
    }

    #[test]
    fn test_narrate_ticket_assignment() {
        let result = narrate_ticket_assignment(Team::Hardware, "abc123456789");
        assert!(result.contains("abc12345"));
        assert!(result.contains("hardware team"));
    }

    #[test]
    fn test_reviewer_badge() {
        assert_eq!(reviewer_badge(Team::Storage, "junior"), "[storage:junior]");
        assert_eq!(reviewer_badge(Team::Network, "senior"), "[network:senior]");
    }

    #[test]
    fn test_format_issues_list() {
        let artifact = ReviewArtifact::new(Team::General, "junior")
            .with_issue(ReviewIssue::warning(
                ReviewIssueKind::TooVague,
                "needs detail",
            ))
            .with_issue(ReviewIssue::blocker(
                ReviewIssueKind::MissingEvidence,
                "no data",
            ));

        let issues = format_issues_list(&artifact);
        assert_eq!(issues.len(), 2);
        assert!(issues[0].contains("warning"));
        assert!(issues[1].contains("blocker"));
    }

    #[test]
    fn test_status_indicator() {
        assert_eq!(status_indicator(true), "✓");
        assert_eq!(status_indicator(false), "✗");
    }

    // v0.0.28 tests

    #[test]
    fn test_it_greeting() {
        assert!(it_greeting("storage").contains("storage"));
        assert!(it_greeting("memory").contains("memory"));
        assert!(it_greeting("network").contains("network"));
        assert!(it_greeting("unknown").contains("look into"));
    }

    #[test]
    fn test_it_confidence() {
        assert!(it_confidence(95).contains("verified"));
        assert!(it_confidence(85).contains("well-supported"));
        assert!(it_confidence(75).contains("available"));
        assert!(it_confidence(55).contains("partial"));
        assert!(it_confidence(40).contains("not be fully"));
    }

    #[test]
    fn test_it_domain_context() {
        assert_eq!(it_domain_context("storage"), "Storage & Filesystems");
        assert_eq!(it_domain_context("MEMORY"), "Memory & RAM");
        assert_eq!(it_domain_context("unknown"), "General Support");
    }

    // v0.0.42: Updated person-based narration tests with new names

    #[test]
    fn test_get_person() {
        let p = get_person(Team::Network, "junior");
        assert_eq!(p.display_name, "Michael");
        assert_eq!(p.role_title, "Network Engineer");
    }

    #[test]
    fn golden_person_action_network_junior() {
        let result = narrate_person_action(Team::Network, "junior", "is reviewing connectivity");
        assert_eq!(
            result,
            "Michael (Network Engineer) is reviewing connectivity"
        );
    }

    #[test]
    fn golden_person_escalation_storage() {
        let result = narrate_person_escalation(Team::Storage, "disk verification failed");
        assert_eq!(
            result,
            "Escalating to Ines (Storage Architect) - disk verification failed"
        );
    }

    #[test]
    fn test_person_review_approved() {
        let artifact = ReviewArtifact::pass(Team::Performance, "junior", 90);
        let result = narrate_person_review(&artifact);
        assert!(result.contains("Kari (Performance Analyst)"));
        assert!(result.contains("approved"));
    }
}
