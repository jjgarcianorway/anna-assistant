//! Team action narration functions (v0.0.218).

use crate::review::{ReviewArtifact, ReviewSeverity};
use crate::teams::Team;

use super::roles::{team_role_name, team_tag};

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
    format!("Ticket {} assigned to {} team", short_id, team_tag(team))
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
