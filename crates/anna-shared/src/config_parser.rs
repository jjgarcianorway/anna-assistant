//! Natural language config command parser (v0.0.239).
//!
//! Parses user requests to change Anna's settings, like:
//! - "disable learning mode"
//! - "enable auto-confirm for low risk"
//! - "make Anna more casual"
//! - "hide internal communications"
//! - "set verbosity to detailed"
//! - "my email is user@example.com"
//! - "notify me at user@example.com"
//!
//! v0.0.239: Added email setup via natural language.

use crate::user_profile::UserPreferences;
use regex::Regex;

/// A parsed config change request
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigChange {
    /// Toggle learning mode (show explanations)
    LearningMode(bool),
    /// Change verbosity (0=minimal, 1=normal, 2=detailed)
    Verbosity(u8),
    /// Toggle auto-confirm for low-risk changes
    AutoConfirmLowRisk(bool),
    /// Toggle internal comms display (fly on wall)
    ShowInternalComms(bool),
    /// Change formality (0=casual, 1=balanced, 2=formal)
    Formality(u8),
    /// Change humor level (0=none, 1=subtle, 2=playful)
    Humor(u8),
    /// Change technical depth (0=simple, 1=balanced, 2=expert)
    TechnicalDepth(u8),
    /// v0.0.239: Set email address for notifications
    Email(String),
    /// v0.0.239: Clear email (disable notifications)
    ClearEmail,
}

impl ConfigChange {
    /// Apply this change to preferences (for preference-based settings only)
    /// Email changes need to be handled separately via apply_email()
    pub fn apply(&self, prefs: &mut UserPreferences) {
        match self {
            ConfigChange::LearningMode(v) => prefs.learning_mode = *v,
            ConfigChange::Verbosity(v) => prefs.verbosity = *v,
            ConfigChange::AutoConfirmLowRisk(v) => prefs.auto_confirm_low_risk = *v,
            ConfigChange::ShowInternalComms(v) => prefs.show_internal_comms = *v,
            ConfigChange::Formality(v) => prefs.personality.formality = *v,
            ConfigChange::Humor(v) => prefs.personality.humor = *v,
            ConfigChange::TechnicalDepth(v) => prefs.personality.technical_depth = *v,
            // Email changes are handled separately
            ConfigChange::Email(_) | ConfigChange::ClearEmail => {}
        }
    }

    /// Check if this is an email-related change
    pub fn is_email_change(&self) -> bool {
        matches!(self, ConfigChange::Email(_) | ConfigChange::ClearEmail)
    }

    /// Get email address if this is an email change
    pub fn get_email(&self) -> Option<&str> {
        match self {
            ConfigChange::Email(e) => Some(e),
            _ => None,
        }
    }

    /// Describe the change in human-readable form
    pub fn description(&self) -> String {
        match self {
            ConfigChange::LearningMode(true) => {
                "Enabled learning mode - I'll explain why commands work.".to_string()
            }
            ConfigChange::LearningMode(false) => {
                "Disabled learning mode - answers will be more concise.".to_string()
            }
            ConfigChange::Verbosity(0) => {
                "Set verbosity to minimal - short answers only.".to_string()
            }
            ConfigChange::Verbosity(1) => "Set verbosity to normal.".to_string(),
            ConfigChange::Verbosity(2) => {
                "Set verbosity to detailed - full explanations.".to_string()
            }
            ConfigChange::Verbosity(_) => "Verbosity updated.".to_string(),
            ConfigChange::AutoConfirmLowRisk(true) => {
                "Enabled auto-confirm for low-risk changes.".to_string()
            }
            ConfigChange::AutoConfirmLowRisk(false) => {
                "Disabled auto-confirm - I'll ask before changes.".to_string()
            }
            ConfigChange::ShowInternalComms(true) => {
                "Showing internal IT communications.".to_string()
            }
            ConfigChange::ShowInternalComms(false) => {
                "Hidden internal communications - cleaner output.".to_string()
            }
            ConfigChange::Formality(0) => "Made my style more casual.".to_string(),
            ConfigChange::Formality(1) => "Set to balanced formality.".to_string(),
            ConfigChange::Formality(2) => "Made my style more formal.".to_string(),
            ConfigChange::Formality(_) => "Formality updated.".to_string(),
            ConfigChange::Humor(0) => "Disabled humor - professional mode.".to_string(),
            ConfigChange::Humor(1) => "Set to subtle humor.".to_string(),
            ConfigChange::Humor(2) => "Enabled playful humor mode.".to_string(),
            ConfigChange::Humor(_) => "Humor level updated.".to_string(),
            ConfigChange::TechnicalDepth(0) => "Simplified technical language.".to_string(),
            ConfigChange::TechnicalDepth(1) => "Balanced technical depth.".to_string(),
            ConfigChange::TechnicalDepth(2) => "Expert mode - full technical details.".to_string(),
            ConfigChange::TechnicalDepth(_) => "Technical depth updated.".to_string(),
            ConfigChange::Email(addr) => format!(
                "Got it! I'll notify you at {} for long-running tickets.",
                addr
            ),
            ConfigChange::ClearEmail => "Email notifications disabled.".to_string(),
        }
    }
}

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

/// Check if query matches any of the patterns
fn matches_any(query: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| query.contains(p))
}

/// Extract email address from text using regex
fn extract_email(text: &str) -> Option<String> {
    let re = Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").ok()?;
    re.find(text).map(|m| m.as_str().to_string())
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
        "learning mode",
        "formal",
        "casual",
        "humor",
        "jokes",
        "technical",
        "confirm",
        "internal comms",
        "fly on wall",
    ];

    config_indicators.iter().any(|&ind| q.contains(ind))
        && (q.contains("anna")
            || q.contains("setting")
            || q.contains("config")
            || q.contains("mode")
            || q.contains("prefer")
            || q.contains("style"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_learning_mode() {
        assert_eq!(
            parse_config_request("enable learning mode"),
            Some(ConfigChange::LearningMode(true))
        );
        assert_eq!(
            parse_config_request("disable learning mode"),
            Some(ConfigChange::LearningMode(false))
        );
    }

    #[test]
    fn test_parse_formality() {
        assert_eq!(
            parse_config_request("make anna more casual"),
            Some(ConfigChange::Formality(0))
        );
        assert_eq!(
            parse_config_request("be more formal"),
            Some(ConfigChange::Formality(2))
        );
    }

    #[test]
    fn test_parse_verbosity() {
        assert_eq!(
            parse_config_request("be more verbose"),
            Some(ConfigChange::Verbosity(2))
        );
        assert_eq!(
            parse_config_request("give me brief answers"),
            Some(ConfigChange::Verbosity(0))
        );
    }

    #[test]
    fn test_parse_humor() {
        assert_eq!(
            parse_config_request("no jokes please"),
            Some(ConfigChange::Humor(0))
        );
        assert_eq!(
            parse_config_request("be more playful"),
            Some(ConfigChange::Humor(2))
        );
    }

    #[test]
    fn test_parse_internal_comms() {
        assert_eq!(
            parse_config_request("show internal comms"),
            Some(ConfigChange::ShowInternalComms(true))
        );
        assert_eq!(
            parse_config_request("hide internal communications"),
            Some(ConfigChange::ShowInternalComms(false))
        );
    }

    #[test]
    fn test_is_config_request() {
        assert!(is_config_request("Anna, enable learning mode"));
        assert!(is_config_request("make Anna more casual"));
        assert!(is_config_request("Anna disable auto confirm"));
        assert!(is_config_request("change my setting to verbose"));
        assert!(!is_config_request("how much disk space do I have"));
    }

    #[test]
    fn test_not_config_request() {
        assert_eq!(parse_config_request("how much memory"), None);
        assert_eq!(parse_config_request("disk usage"), None);
    }

    #[test]
    fn test_extract_email() {
        assert_eq!(
            extract_email("my email is user@example.com"),
            Some("user@example.com".to_string())
        );
        assert_eq!(
            extract_email("notify me at test@domain.org please"),
            Some("test@domain.org".to_string())
        );
        assert_eq!(extract_email("no email here"), None);
    }

    #[test]
    fn test_parse_email() {
        assert_eq!(
            parse_config_request("my email is user@example.com"),
            Some(ConfigChange::Email("user@example.com".to_string()))
        );
        assert_eq!(
            parse_config_request("notify me at test@domain.org"),
            Some(ConfigChange::Email("test@domain.org".to_string()))
        );
        assert_eq!(
            parse_config_request("disable email notifications"),
            Some(ConfigChange::ClearEmail)
        );
    }

    #[test]
    fn test_is_email_config_request() {
        assert!(is_config_request("my email is user@example.com"));
        assert!(is_config_request("notify me at test@domain.org"));
        assert!(is_config_request("disable email notifications"));
    }
}
