//! Humanized person-based narration (v0.0.218).

use crate::review::{ReviewArtifact, ReviewSeverity};
use crate::roster::{person_for, PersonProfile, Tier};
use crate::teams::Team;

/// Get person profile for a reviewer tier string
fn tier_from_str(reviewer: &str) -> Tier {
    if reviewer.to_lowercase().contains("senior") {
        Tier::Senior
    } else {
        Tier::Junior
    }
}

/// Get the person profile for a team + reviewer.
pub fn get_person(team: Team, reviewer: &str) -> PersonProfile {
    person_for(team, tier_from_str(reviewer))
}

/// Narrate a team action using the person's name (v0.0.32).
/// Returns: "Riley (Network Administrator) is reviewing..."
pub fn narrate_person_action(team: Team, reviewer: &str, action: &str) -> String {
    let person = get_person(team, reviewer);
    format!("{} {}", person.display(), action)
}

/// Narrate a review result using person's name (v0.0.32).
pub fn narrate_person_review(artifact: &ReviewArtifact) -> String {
    let person = get_person(artifact.team, &artifact.reviewer);
    if artifact.allow_publish {
        if artifact.issues.is_empty() {
            format!("{}: approved (score {})", person.display(), artifact.score)
        } else {
            let warning_count = artifact.issue_count(ReviewSeverity::Warning);
            format!(
                "{}: approved with {} note{} (score {})",
                person.display(),
                warning_count,
                if warning_count == 1 { "" } else { "s" },
                artifact.score
            )
        }
    } else {
        let blocker_count = artifact.issue_count(ReviewSeverity::Blocker);
        format!(
            "{}: needs revision - {} issue{} (score {})",
            person.display(),
            blocker_count,
            if blocker_count == 1 { "" } else { "s" },
            artifact.score
        )
    }
}

/// Narrate an escalation using person's name (v0.0.32).
pub fn narrate_person_escalation(from_team: Team, reason: &str) -> String {
    let senior = person_for(from_team, Tier::Senior);
    format!("Escalating to {} - {}", senior.display(), reason)
}
