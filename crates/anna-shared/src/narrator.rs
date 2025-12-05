//! Team-aware dialog formatting for service desk narration.
//!
//! Provides consistent display names and narrative formatting for team actions.
//! Used by transcript rendering to present team-based review activity.

use crate::review::{ReviewArtifact, ReviewSeverity};
use crate::teams::Team;

/// Get the display name for a team + reviewer combination.
/// Returns human-readable role titles for non-debug display.
///
/// Pinned mapping table - same inputs always produce same output.
pub fn team_role_name(team: Team, reviewer: &str) -> &'static str {
    match (team, reviewer) {
        // Desktop team
        (Team::Desktop, "junior") => "Desktop Administrator",
        (Team::Desktop, "senior") => "Desktop Specialist",
        // Storage team
        (Team::Storage, "junior") => "Storage Engineer",
        (Team::Storage, "senior") => "Storage Architect",
        // Network team
        (Team::Network, "junior") => "Network Engineer",
        (Team::Network, "senior") => "Network Architect",
        // Performance team
        (Team::Performance, "junior") => "Performance Analyst",
        (Team::Performance, "senior") => "Performance Engineer",
        // Services team
        (Team::Services, "junior") => "Services Administrator",
        (Team::Services, "senior") => "Services Architect",
        // Security team
        (Team::Security, "junior") => "Security Analyst",
        (Team::Security, "senior") => "Security Engineer",
        // Hardware team
        (Team::Hardware, "junior") => "Hardware Technician",
        (Team::Hardware, "senior") => "Hardware Engineer",
        // General/fallback
        (Team::General, "junior") => "Support Analyst",
        (Team::General, "senior") => "Support Specialist",
        // Unknown reviewer level
        (_, _) => "Reviewer",
    }
}

/// Get short debug tag for team (used in debug mode).
pub fn team_tag(team: Team) -> &'static str {
    match team {
        Team::Desktop => "desktop",
        Team::Storage => "storage",
        Team::Network => "network",
        Team::Performance => "perf",
        Team::Services => "services",
        Team::Security => "security",
        Team::Hardware => "hardware",
        Team::General => "general",
    }
}

/// Narrate a team action for display.
/// Returns a formatted string describing what the team member is doing.
pub fn narrate_team_action(team: Team, reviewer: &str, action: &str) -> String {
    let role = team_role_name(team, reviewer);
    format!("{} {}", role, action)
}

/// Narrate a review result for display.
/// Returns a formatted summary of the review outcome.
pub fn narrate_review_result(artifact: &ReviewArtifact) -> String {
    let role = team_role_name(artifact.team, &artifact.reviewer);

    if artifact.allow_publish {
        if artifact.issues.is_empty() {
            format!("{}: approved (score {})", role, artifact.score)
        } else {
            let warning_count = artifact.issue_count(ReviewSeverity::Warning);
            let info_count = artifact.issue_count(ReviewSeverity::Info);
            format!(
                "{}: approved with {} warning{}, {} note{} (score {})",
                role,
                warning_count,
                if warning_count == 1 { "" } else { "s" },
                info_count,
                if info_count == 1 { "" } else { "s" },
                artifact.score
            )
        }
    } else {
        let blocker_count = artifact.issue_count(ReviewSeverity::Blocker);
        format!(
            "{}: needs revision - {} blocker{} (score {})",
            role,
            blocker_count,
            if blocker_count == 1 { "" } else { "s" },
            artifact.score
        )
    }
}

/// Narrate an escalation for display.
/// Returns a formatted string describing the escalation.
pub fn narrate_escalation(from_team: Team, reason: &str) -> String {
    let senior_role = team_role_name(from_team, "senior");
    format!("Escalating to {} - {}", senior_role, reason)
}

/// Narrate a ticket assignment for display.
pub fn narrate_ticket_assignment(team: Team, ticket_id: &str) -> String {
    let short_id = if ticket_id.len() > 8 {
        &ticket_id[..8]
    } else {
        ticket_id
    };
    format!(
        "Ticket {} assigned to {} team",
        short_id,
        team_tag(team)
    )
}

/// Format reviewer badge for debug display.
/// Returns formatted badge like "[storage:junior]" or "[network:senior]"
pub fn reviewer_badge(team: Team, reviewer: &str) -> String {
    format!("[{}:{}]", team_tag(team), reviewer)
}

/// Format issues list for display.
pub fn format_issues_list(artifact: &ReviewArtifact) -> Vec<String> {
    artifact
        .issues
        .iter()
        .map(|i| format!("[{}] {}: {}", i.severity, i.kind, i.message))
        .collect()
}

/// Get emoji indicator for review status (if emojis enabled).
pub fn status_indicator(allow_publish: bool) -> &'static str {
    if allow_publish {
        "✓"
    } else {
        "✗"
    }
}

// === IT Department Dialog Style (v0.0.28) ===

/// Phrases for IT department style greetings based on query type.
/// Returns a contextual greeting for the service desk response.
pub fn it_greeting(domain: &str) -> &'static str {
    match domain.to_lowercase().as_str() {
        "storage" | "disk" => "Let me check that storage information for you.",
        "memory" | "ram" => "I'll look into the memory usage right away.",
        "network" | "wifi" | "dns" => "Let me examine your network configuration.",
        "performance" | "cpu" | "slow" => "I'll analyze the system performance.",
        "service" | "systemd" => "Let me check those service statuses.",
        "security" | "permission" => "I'll review the security information carefully.",
        "hardware" | "gpu" => "Let me gather the hardware details.",
        _ => "Let me look into that for you.",
    }
}

/// Format reliability as IT confidence statement.
pub fn it_confidence(score: u8) -> &'static str {
    match score {
        90..=100 => "This information is verified from system data.",
        80..=89 => "This information is well-supported by system data.",
        70..=79 => "This information is based on available system data.",
        50..=69 => "This is based on partial data; some details may need verification.",
        _ => "This information could not be fully verified.",
    }
}

/// Format domain as IT department context.
pub fn it_domain_context(domain: &str) -> &'static str {
    match domain.to_lowercase().as_str() {
        "storage" => "Storage & Filesystems",
        "memory" => "Memory & RAM",
        "network" => "Network & Connectivity",
        "performance" => "System Performance",
        "service" | "services" => "System Services",
        "security" => "Security & Permissions",
        "hardware" => "Hardware & Devices",
        "system" => "System Status",
        _ => "General Support",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::{ReviewIssue, ReviewIssueKind};

    #[test]
    fn test_team_role_names() {
        assert_eq!(team_role_name(Team::Storage, "junior"), "Storage Engineer");
        assert_eq!(team_role_name(Team::Storage, "senior"), "Storage Architect");
        assert_eq!(team_role_name(Team::Desktop, "junior"), "Desktop Administrator");
        assert_eq!(team_role_name(Team::Security, "senior"), "Security Engineer");
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
            .with_issue(ReviewIssue::blocker(ReviewIssueKind::MissingEvidence, "need disk data"));
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
            .with_issue(ReviewIssue::warning(ReviewIssueKind::TooVague, "needs detail"))
            .with_issue(ReviewIssue::blocker(ReviewIssueKind::MissingEvidence, "no data"));

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
}
