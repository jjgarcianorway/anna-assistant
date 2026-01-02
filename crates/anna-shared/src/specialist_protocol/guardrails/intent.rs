//! Intent type classification for user questions.

/// Intent type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentType {
    /// User is asking about current state ("do I have X?", "is X running?")
    CheckState,
    /// User wants to know how to do something ("how do I configure X?")
    HowTo,
    /// User wants explanation ("what does X mean?")
    Explain,
    /// User is reporting a problem ("X is not working")
    Diagnose,
    /// User wants to perform an action ("install X", "restart Y")
    Action,
    /// Unknown/ambiguous intent
    Unknown,
}

/// Classify intent from question text
pub fn classify_intent(question: &str) -> IntentType {
    let lower = question.to_lowercase();

    // State-checking patterns (highest priority)
    let state_patterns = [
        "do i have",
        "is there",
        "are there",
        "am i",
        "is my",
        "show me",
        "list my",
        "what is my",
        "how much",
        "how many",
        "is it running",
        "is it installed",
        "is it enabled",
        "is it active",
        "check if",
        "currently",
        "right now",
    ];

    for pattern in &state_patterns {
        if lower.contains(pattern) {
            return IntentType::CheckState;
        }
    }

    // How-to patterns
    let howto_patterns = [
        "how do i",
        "how can i",
        "how to",
        "how should i",
        "what's the best way to",
        "steps to",
        "guide to",
        "tutorial",
        "configure",
        "set up",
        "setup",
    ];

    for pattern in &howto_patterns {
        if lower.contains(pattern) {
            return IntentType::HowTo;
        }
    }

    // Explain patterns
    let explain_patterns = [
        "what does",
        "what is",
        "what are",
        "explain",
        "meaning of",
        "difference between",
        "why does",
    ];

    for pattern in &explain_patterns {
        if lower.contains(pattern) {
            return IntentType::Explain;
        }
    }

    // Diagnose patterns
    let diagnose_patterns = [
        "not working",
        "doesn't work",
        "won't start",
        "failing",
        "failed",
        "error",
        "problem with",
        "issue with",
        "trouble with",
        "broken",
        "crash",
    ];

    for pattern in &diagnose_patterns {
        if lower.contains(pattern) {
            return IntentType::Diagnose;
        }
    }

    // Action patterns
    let action_patterns = [
        "install", "remove", "delete", "restart", "stop", "start", "enable", "disable", "update",
        "upgrade",
    ];

    for pattern in &action_patterns {
        if lower.contains(pattern) {
            return IntentType::Action;
        }
    }

    IntentType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_intent_state() {
        // "do i have" triggers CheckState
        assert_eq!(
            classify_intent("do I have any failed services?"),
            IntentType::CheckState
        );
        // "how much" triggers CheckState
        assert_eq!(
            classify_intent("how much RAM do I have?"),
            IntentType::CheckState
        );
        // "is it running" needs the full phrase
        assert_eq!(
            classify_intent("Is nginx currently running?"),
            IntentType::CheckState
        );
    }

    #[test]
    fn test_classify_intent_howto() {
        assert_eq!(
            classify_intent("How do I configure nginx?"),
            IntentType::HowTo
        );
        assert_eq!(classify_intent("How to install vim?"), IntentType::HowTo);
    }

    #[test]
    fn test_classify_intent_diagnose() {
        assert_eq!(
            classify_intent("My wifi is not working"),
            IntentType::Diagnose
        );
        assert_eq!(
            classify_intent("nginx service failed"),
            IntentType::Diagnose
        );
    }
}
