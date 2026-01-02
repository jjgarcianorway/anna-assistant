//! Configuration request parsing logic.

use super::types::ConfigChange;
use super::utils::{extract_email, matches_any};

/// Try to parse a config change request from user query
pub fn parse_config_request(query: &str) -> Option<ConfigChange> {
    let q = query.to_lowercase();

    // v0.0.239: Email setup
    if let Some(email) = extract_email(query) {
        if matches_any(
            &q,
            &[
                "my email",
                "email is",
                "notify me",
                "reach me",
                "contact me",
                "email me",
            ],
        ) {
            return Some(ConfigChange::Email(email));
        }
    }
    if matches_any(
        &q,
        &[
            "disable email",
            "no email",
            "don't email",
            "stop email",
            "remove email",
            "clear email",
        ],
    ) {
        return Some(ConfigChange::ClearEmail);
    }

    // Learning mode
    if matches_any(
        &q,
        &[
            "enable learning",
            "turn on learning",
            "explain commands",
            "teach me",
        ],
    ) {
        return Some(ConfigChange::LearningMode(true));
    }
    if matches_any(
        &q,
        &[
            "disable learning",
            "turn off learning",
            "no explanations",
            "don't explain",
        ],
    ) {
        return Some(ConfigChange::LearningMode(false));
    }

    // Verbosity
    if matches_any(&q, &["minimal output", "brief", "less verbose", "concise"]) {
        return Some(ConfigChange::Verbosity(0));
    }
    if matches_any(
        &q,
        &["detailed", "verbose", "more detail", "full explanation"],
    ) {
        return Some(ConfigChange::Verbosity(2));
    }
    if matches_any(
        &q,
        &["normal verbosity", "regular output", "default verbosity"],
    ) {
        return Some(ConfigChange::Verbosity(1));
    }

    // Auto-confirm
    if matches_any(
        &q,
        &[
            "auto confirm",
            "don't ask",
            "auto-confirm low risk",
            "skip confirmation",
        ],
    ) {
        return Some(ConfigChange::AutoConfirmLowRisk(true));
    }
    if matches_any(
        &q,
        &["ask before", "confirm changes", "no auto", "always ask"],
    ) {
        return Some(ConfigChange::AutoConfirmLowRisk(false));
    }

    // Internal comms
    if matches_any(
        &q,
        &["show internal", "show comms", "fly on wall", "see the team"],
    ) {
        return Some(ConfigChange::ShowInternalComms(true));
    }
    if matches_any(
        &q,
        &["hide internal", "hide comms", "no internal", "clean output"],
    ) {
        return Some(ConfigChange::ShowInternalComms(false));
    }

    // Formality
    if matches_any(
        &q,
        &[
            "more casual",
            "be casual",
            "relaxed",
            "informal",
            "less formal",
        ],
    ) {
        return Some(ConfigChange::Formality(0));
    }
    if matches_any(
        &q,
        &["more formal", "professional", "be formal", "business"],
    ) {
        return Some(ConfigChange::Formality(2));
    }
    if matches_any(&q, &["balanced formality", "normal formality"]) {
        return Some(ConfigChange::Formality(1));
    }

    // Humor
    if matches_any(
        &q,
        &["no humor", "no jokes", "be serious", "professional mode"],
    ) {
        return Some(ConfigChange::Humor(0));
    }
    if matches_any(
        &q,
        &["be funny", "more humor", "playful", "be playful", "jokes"],
    ) {
        return Some(ConfigChange::Humor(2));
    }
    if matches_any(&q, &["subtle humor", "light humor"]) {
        return Some(ConfigChange::Humor(1));
    }

    // Technical depth
    if matches_any(
        &q,
        &[
            "simple terms",
            "simpler",
            "less technical",
            "explain simply",
        ],
    ) {
        return Some(ConfigChange::TechnicalDepth(0));
    }
    if matches_any(
        &q,
        &[
            "expert mode",
            "more technical",
            "full tech",
            "technical details",
        ],
    ) {
        return Some(ConfigChange::TechnicalDepth(2));
    }
    if matches_any(&q, &["balanced technical", "normal depth"]) {
        return Some(ConfigChange::TechnicalDepth(1));
    }

    None
}

/// Check if a query looks like a config request (for routing)
pub fn is_config_request(query: &str) -> bool {
    let q = query.to_lowercase();

    // v0.0.239: Check for email setup patterns (these are always config)
    if extract_email(query).is_some()
        && matches_any(
            &q,
            &[
                "my email",
                "email is",
                "notify me",
                "reach me",
                "contact me",
            ],
        )
    {
        return true;
    }
    if matches_any(
        &q,
        &[
            "disable email",
            "no email",
            "don't email",
            "stop email",
            "remove email",
        ],
    ) {
        return true;
    }

    // v0.0.790: Direct pattern matching for common settings commands
    // These patterns are unambiguous and don't need the secondary check
    let direct_patterns = [
        "show internal",
        "hide internal",
        "show comms",
        "hide comms",
        "fly on wall",
        "see the team",
        "internal comms",
        "learning mode",
        "auto confirm",
        "auto-confirm",
    ];
    if direct_patterns.iter().any(|&p| q.contains(p)) {
        return true;
    }

    // Check for common config-related keywords
    let config_indicators = [
        "enable",
        "disable",
        "turn on",
        "turn off",
        "set",
        "change",
        "make anna",
        "make you",
        "be more",
        "be less",
        "show me",
        "hide",
        "verbose",
        "formal",
        "casual",
        "humor",
        "jokes",
        "technical",
        "confirm",
    ];

    config_indicators.iter().any(|&ind| q.contains(ind))
        && (q.contains("anna")
            || q.contains("setting")
            || q.contains("config")
            || q.contains("mode")
            || q.contains("prefer")
            || q.contains("style"))
}
