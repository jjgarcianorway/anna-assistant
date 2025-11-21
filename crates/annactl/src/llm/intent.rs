//! Intent Detection - Classify user query intent
//!
//! Beta.200: Focused module for intent detection and classification
//!
//! Responsibilities:
//! - Detect whether query is informational or actionable
//! - Classify query type (system info, package management, configuration, etc.)
//! - Determine if query matches a known recipe pattern

use anyhow::Result;

/// Query intent classification
#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    /// User wants information about the system
    Informational,

    /// User wants to perform an action (install, configure, fix, etc.)
    Actionable,

    /// User wants a system report
    Report,

    /// Unclear or ambiguous intent
    Unknown,
}

/// Detect the intent of a user query
///
/// Beta.200: Simplified intent detection
/// - Informational: "what", "show", "tell me", "how much"
/// - Actionable: "install", "fix", "configure", "setup"
/// - Report: "report", "status", "health"
pub fn detect_intent(query: &str) -> Intent {
    let query_lower = query.to_lowercase();

    // Report intent
    if query_lower.contains("report")
        || query_lower.contains("full system")
        || query_lower.contains("system report")
    {
        return Intent::Report;
    }

    // Actionable intent keywords
    let actionable_keywords = [
        "install",
        "setup",
        "configure",
        "fix",
        "enable",
        "disable",
        "start",
        "stop",
        "restart",
        "update",
        "upgrade",
        "remove",
        "uninstall",
    ];

    for keyword in &actionable_keywords {
        if query_lower.contains(keyword) {
            return Intent::Actionable;
        }
    }

    // Informational intent keywords
    let informational_keywords = [
        "what", "show", "tell me", "how much", "how many", "which", "list", "check",
    ];

    for keyword in &informational_keywords {
        if query_lower.contains(keyword) {
            return Intent::Informational;
        }
    }

    // Default to informational for safety
    Intent::Informational
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_informational_intent() {
        assert_eq!(
            detect_intent("what is my CPU model?"),
            Intent::Informational
        );
        assert_eq!(detect_intent("show me disk space"), Intent::Informational);
        assert_eq!(
            detect_intent("how much RAM do I have?"),
            Intent::Informational
        );
    }

    #[test]
    fn test_actionable_intent() {
        assert_eq!(detect_intent("install docker"), Intent::Actionable);
        assert_eq!(detect_intent("fix my network"), Intent::Actionable);
        assert_eq!(detect_intent("setup nginx"), Intent::Actionable);
    }

    #[test]
    fn test_report_intent() {
        assert_eq!(
            detect_intent("give me a full system report"),
            Intent::Report
        );
        assert_eq!(detect_intent("system report"), Intent::Report);
    }
}
