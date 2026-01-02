//! Clarification Decision Logic - v0.0.442.
//!
//! Enforce Fact-First Clarification Loop (FCL):
//! - For config-like intents, check required facts FIRST
//! - If facts missing → clarify BEFORE running probes
//! - No probes until facts are known
//!
//! WRONG ORDER: probes → clarification → answer
//! RIGHT ORDER: check facts → clarify if missing → probes → answer

use super::clarification_facts::KnownFacts;
use super::clarification_types::{ClarificationDecision, ClarificationRequiredIntent};

/// Check if clarification is needed BEFORE probes.
pub fn check_clarification_needed(intent: &str, known_facts: &KnownFacts) -> ClarificationDecision {
    // Check if this is a clarification-required intent
    let cri = match ClarificationRequiredIntent::from_intent(intent) {
        Some(i) => i,
        None => return ClarificationDecision::NotClarificationIntent,
    };

    // Get required facts for this intent
    let required = cri.required_facts();

    // Check which are missing
    let missing = known_facts.missing(&required);

    if missing.is_empty() {
        return ClarificationDecision::ProceedToProbes;
    }

    // Get clarification for first missing fact
    let first_missing = &missing[0];
    if let Some(question) = cri.clarification_for(first_missing) {
        return ClarificationDecision::NeedClarification {
            question,
            missing_facts: missing,
        };
    }

    // No clarification question defined, but facts are missing
    // This is a bug in the system - we should have questions for all required facts
    ClarificationDecision::ProceedToProbes
}

/// Intent patterns that should trigger clarification-first.
pub fn is_clarification_required_intent(intent: &str) -> bool {
    let lower = intent.to_lowercase();

    // Editor-related config questions
    if lower.contains("editor")
        && (lower.contains("syntax") || lower.contains("config") || lower.contains("setup"))
    {
        return true;
    }

    // Wallpaper questions
    if lower.contains("wallpaper") && (lower.contains("where") || lower.contains("location")) {
        return true;
    }

    // Terminal config questions
    if lower.contains("terminal") && (lower.contains("theme") || lower.contains("config")) {
        return true;
    }

    // Shell config questions
    if lower.contains("shell") && (lower.contains("prompt") || lower.contains("config")) {
        return true;
    }

    ClarificationRequiredIntent::from_intent(intent).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ticket_integrity::clarification::clarification_facts::FactSource;

    #[test]
    fn test_clarification_decision_need_clarification() {
        let facts = KnownFacts::new();
        let decision = check_clarification_needed("editor.syntax_status", &facts);

        match decision {
            ClarificationDecision::NeedClarification {
                question,
                missing_facts,
            } => {
                assert!(!missing_facts.is_empty());
                assert!(question.question.contains("editor"));
            }
            _ => panic!("Expected NeedClarification"),
        }
    }

    #[test]
    fn test_clarification_decision_proceed() {
        let mut facts = KnownFacts::new();
        facts.add("editor.name", "vim", FactSource::User);
        facts.add("editor.config_path", "~/.vimrc", FactSource::User);

        let decision = check_clarification_needed("editor.syntax_status", &facts);
        assert!(matches!(decision, ClarificationDecision::ProceedToProbes));
    }

    #[test]
    fn test_is_clarification_required_intent() {
        assert!(is_clarification_required_intent("editor.syntax_status"));
        assert!(is_clarification_required_intent("wallpapers_location"));
        assert!(!is_clarification_required_intent("memory.free"));
        assert!(!is_clarification_required_intent("system.swap_configured"));
    }
}
