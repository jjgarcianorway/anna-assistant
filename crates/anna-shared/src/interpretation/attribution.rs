//! Attribution - Determine who resolved an issue.
//!
//! Attribution logic:
//! - Was it Anna? Check outcome ledger for Anna actions
//! - Was it User? Check session history for user questions then resolution
//! - Unknown? Insufficient evidence to attribute
//!
//! NEVER GUESS. If attribution is unclear, mark as Unknown.

use super::recognition::ResolutionEvent;
use crate::outcome_ledger::{read_all_outcomes, Outcome};
use serde::{Deserialize, Serialize};

/// The actor who resolved an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    /// Anna performed an action that resolved the issue
    Anna,
    /// User performed an external action (not through Anna)
    User,
    /// Cannot determine who resolved it
    Unknown,
}

/// Confidence level in the attribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// Strong evidence supports this attribution
    High,
    /// Some evidence, but not conclusive
    Medium,
    /// Attribution is uncertain
    Low,
    /// No evidence either way
    None,
}

/// Attribution result for a resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    /// Who resolved the issue
    pub actor: Actor,
    /// Confidence in this attribution
    pub confidence: Confidence,
    /// Evidence supporting this attribution (if any)
    pub evidence: Option<String>,
    /// Reason for this attribution
    pub reason: String,
}

impl Attribution {
    /// Create an Anna attribution.
    fn anna(confidence: Confidence, evidence: &str) -> Self {
        Self {
            actor: Actor::Anna,
            confidence,
            evidence: Some(evidence.to_string()),
            reason: "Anna action recorded in outcome ledger".to_string(),
        }
    }

    /// Create a User attribution.
    fn user(confidence: Confidence, evidence: &str) -> Self {
        Self {
            actor: Actor::User,
            confidence,
            evidence: Some(evidence.to_string()),
            reason: "User asked about issue, then issue resolved externally".to_string(),
        }
    }

    /// Create an Unknown attribution.
    fn unknown() -> Self {
        Self {
            actor: Actor::Unknown,
            confidence: Confidence::None,
            evidence: None,
            reason: "Insufficient evidence to determine actor".to_string(),
        }
    }
}

/// Attribute a resolution to an actor.
///
/// Attribution logic (pseudocode):
/// ```text
/// 1. Check outcome ledger for Anna actions on this issue type
///    - If Anna action with Resolved outcome exists before resolution: Anna (High)
///
/// 2. Check session history for user questions about this issue
///    - If user asked about issue, then issue resolved: User (Medium)
///
/// 3. Otherwise: Unknown
/// ```
pub fn attribute_resolution(resolution: &ResolutionEvent) -> Attribution {
    // Step 1: Check if Anna performed any action on this issue type
    if let Some(anna_evidence) = check_anna_actions(resolution) {
        return Attribution::anna(Confidence::High, &anna_evidence);
    }

    // Step 2: Check if user asked about this issue (then resolved externally)
    if let Some(user_evidence) = check_user_involvement(resolution) {
        return Attribution::user(Confidence::Medium, &user_evidence);
    }

    // Step 3: No evidence - Unknown
    Attribution::unknown()
}

/// Check outcome ledger for Anna actions related to this issue.
fn check_anna_actions(resolution: &ResolutionEvent) -> Option<String> {
    use crate::outcome_ledger::IntentClassRecord;

    let entries = read_all_outcomes().ok()?;

    // Look for entries where Anna resolved something related to this issue type
    for entry in entries.iter().rev() {
        // Skip entries after the resolution was detected
        if let Ok(entry_time) = chrono::DateTime::parse_from_rfc3339(&entry.ts_utc) {
            let entry_utc = entry_time.with_timezone(&chrono::Utc);
            if entry_utc > resolution.detected_at {
                continue;
            }

            // Check if this entry is related to the issue type
            // and has a Resolved outcome
            if entry.outcome == Outcome::Resolved {
                // Loose matching: mutating actions might resolve config/service issues
                if relates_to_issue(&entry.intent, &resolution.issue_type) {
                    return Some(format!(
                        "Anna action at {} with outcome Resolved",
                        entry.ts_utc
                    ));
                }
            }
        }
    }

    None
}

/// Check if an intent relates to an issue type.
fn relates_to_issue(
    intent: &crate::outcome_ledger::IntentClassRecord,
    issue_type: &crate::monitor::IssueType,
) -> bool {
    use crate::outcome_ledger::IntentClassRecord;
    use crate::monitor::IssueType;

    // This is a loose heuristic - we're conservative about claiming Anna did it
    match (intent, issue_type) {
        (IntentClassRecord::Mutating, IssueType::ConfigChanged) => true,
        (IntentClassRecord::Mutating, IssueType::ServiceFailed) => true,
        _ => false,
    }
}

/// Check session history for user involvement with this issue.
fn check_user_involvement(resolution: &ResolutionEvent) -> Option<String> {
    // Check if user asked about this issue type recently
    // This is a heuristic: if user asked, then resolved, likely user did it

    // For now, we check if the issue summary was mentioned in any recent session
    // This would require access to session store, which we don't have directly here
    // So we return None - conservative approach

    // In a full implementation, this would:
    // 1. Load session history
    // 2. Search for questions mentioning the issue type or summary
    // 3. If found and before resolution, attribute to User

    // For now: we cannot determine user involvement without session access
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::IssueType;
    use chrono::Utc;

    #[test]
    fn test_actor_equality() {
        assert_eq!(Actor::Anna, Actor::Anna);
        assert_ne!(Actor::Anna, Actor::User);
        assert_ne!(Actor::User, Actor::Unknown);
    }

    #[test]
    fn test_attribution_unknown() {
        let attr = Attribution::unknown();
        assert_eq!(attr.actor, Actor::Unknown);
        assert_eq!(attr.confidence, Confidence::None);
        assert!(attr.evidence.is_none());
    }

    #[test]
    fn test_attribution_anna() {
        let attr = Attribution::anna(Confidence::High, "test evidence");
        assert_eq!(attr.actor, Actor::Anna);
        assert_eq!(attr.confidence, Confidence::High);
        assert_eq!(attr.evidence, Some("test evidence".to_string()));
    }

    #[test]
    fn test_attribution_user() {
        let attr = Attribution::user(Confidence::Medium, "user asked then resolved");
        assert_eq!(attr.actor, Actor::User);
        assert_eq!(attr.confidence, Confidence::Medium);
    }
}
