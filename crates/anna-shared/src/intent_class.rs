//! Intent Classification - READ_ONLY vs MUTATING
//!
//! Phase 22: Fixes the "Would you like me to handle it" problem for read-only questions.
//! READ_ONLY intents get direct answers without confirmation flows.

use regex::Regex;
use std::sync::LazyLock;

/// Intent classification for answer contract enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IntentClass {
    /// Read-only: inspection, diagnosis, reporting.
    /// - May run probes automatically
    /// - Must answer directly
    /// - Must NOT ask for confirmation
    /// - Must NOT output commands
    #[default]
    ReadOnly,
    /// Mutating: changes configuration, files, services.
    /// - Must generate ActionPlan
    /// - Must request confirmation
    /// - Must verify and rollback on failure
    Mutating,
}

impl IntentClass {
    /// Whether this intent allows automatic probe execution.
    pub fn allows_auto_probes(&self) -> bool {
        true // Both can run probes
    }

    /// Whether this intent requires user confirmation before action.
    pub fn requires_confirmation(&self) -> bool {
        *self == IntentClass::Mutating
    }

    /// Whether commands can appear in the answer.
    pub fn allows_commands_in_answer(&self) -> bool {
        false // Never show commands in answers (Phase 22 contract)
    }
}

// READ_ONLY patterns - questions that don't require changes
static READONLY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Inspection verbs
        Regex::new(r"(?i)^(show|display|list|check|detect|diagnose|report|describe)\b").unwrap(),
        // Question forms
        Regex::new(r"(?i)^(what('?s| is| are)|how (much|many)|is my|are my|why does|why is|why do)\b").unwrap(),
        // Status queries
        Regex::new(r"(?i)\b(status|usage|info|information|version|uptime)\b").unwrap(),
        // Diagnostic queries
        Regex::new(r"(?i)\b(using pipewire|using pulseaudio|thermal throttl|overheat|temperature)\b").unwrap(),
        Regex::new(r"(?i)\b(swap usage|swappiness|memory pressure|disk space|free space)\b").unwrap(),
        Regex::new(r"(?i)\b(bluetooth fail|wifi fail|network fail|audio fail)\b").unwrap(),
        // Recommend without explicit action request
        Regex::new(r"(?i)\brecommend(ed)?\s+(action|step|fix|solution)s?\b").unwrap(),
    ]
});

// MUTATING patterns - questions that request changes
static MUTATING_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Change verbs
        Regex::new(r"(?i)\b(change|set|configure|modify|edit|create|delete)\b").unwrap(),
        // Package operations
        Regex::new(r"(?i)\b(install|uninstall|remove|upgrade|update)\s").unwrap(),
        // Service control (verb + optional words + service/unit/daemon/service-name)
        Regex::new(r"(?i)\b(enable|disable|start|stop|restart|mask|unmask)\s+(the\s+)?(\w+\s+)?(service|unit|daemon)\b").unwrap(),
        // Service names directly (e.g., "disable bluetooth", "enable ssh")
        Regex::new(r"(?i)\b(enable|disable|start|stop|restart|mask|unmask)\s+(the\s+)?(bluetooth|ssh|sshd|nginx|httpd|docker|gdm|sddm|lightdm|cups|firewalld|networkmanager|pipewire|pulseaudio)\b").unwrap(),
        // Explicit fix requests
        Regex::new(r"(?i)\b(fix|repair|resolve)\s+(this|the|it|my)\b").unwrap(),
        // Explicit action requests
        Regex::new(r"(?i)\b(do it|apply|proceed|make the change|go ahead)\b").unwrap(),
        // Prevention/enforcement
        Regex::new(r"(?i)\b(prevent|ensure|guarantee|make sure)\b").unwrap(),
        // File/config targets
        Regex::new(r"(?i)\b(write to|append to|modify)\s+(/|~)").unwrap(),
    ]
});

/// Classify a user question as READ_ONLY or MUTATING.
pub fn classify_intent(question: &str) -> IntentClass {
    let q = question.trim();

    // Check for explicit MUTATING patterns first
    for pattern in MUTATING_PATTERNS.iter() {
        if pattern.is_match(q) {
            return IntentClass::Mutating;
        }
    }

    // Check for READ_ONLY patterns
    for pattern in READONLY_PATTERNS.iter() {
        if pattern.is_match(q) {
            return IntentClass::ReadOnly;
        }
    }

    // Default: READ_ONLY for questions, MUTATING for imperatives
    if q.ends_with('?') || q.to_lowercase().starts_with("what")
        || q.to_lowercase().starts_with("how")
        || q.to_lowercase().starts_with("why")
        || q.to_lowercase().starts_with("is ")
        || q.to_lowercase().starts_with("are ")
    {
        IntentClass::ReadOnly
    } else {
        // Conservative: treat unclear imperatives as read-only unless clearly mutating
        IntentClass::ReadOnly
    }
}

/// Check if a question is explicitly requesting changes.
pub fn is_mutating_request(question: &str) -> bool {
    classify_intent(question) == IntentClass::Mutating
}

/// Check if a question is read-only (diagnosis/inspection).
pub fn is_readonly_request(question: &str) -> bool {
    classify_intent(question) == IntentClass::ReadOnly
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_classification() {
        let readonly_questions = [
            "check swap usage and swappiness value",
            "is my audio stack using pipewire or pulseaudio",
            "detect thermal throttling and overheating",
            "why does bluetooth fail after suspend",
            "show disk usage",
            "what is my cpu usage",
            "how much ram do i have",
            "list running services",
            "diagnose network issues",
            "what's using my bandwidth",
            "recommend actions for slow boot",
        ];

        for q in readonly_questions {
            assert_eq!(classify_intent(q), IntentClass::ReadOnly,
                "Expected READ_ONLY for: {}", q);
        }
    }

    #[test]
    fn test_mutating_classification() {
        let mutating_questions = [
            "install neovim",
            "disable bluetooth service",
            "enable the ssh daemon",
            "fix this error",
            "change the swappiness value to 10",
            "set my hostname to archbox",
            "configure wifi",
            "remove the orphan packages",
            "do it",
            "apply the fix",
            "prevent thermal throttling",
        ];

        for q in mutating_questions {
            assert_eq!(classify_intent(q), IntentClass::Mutating,
                "Expected MUTATING for: {}", q);
        }
    }

    #[test]
    fn test_question_forms_are_readonly() {
        let questions = [
            "what is wrong with my system?",
            "how do I check disk usage?",
            "why is my fan spinning?",
            "is there a problem with my network?",
            "are there any errors in the logs?",
        ];

        for q in questions {
            assert_eq!(classify_intent(q), IntentClass::ReadOnly,
                "Question form should be READ_ONLY: {}", q);
        }
    }

    #[test]
    fn test_intent_class_properties() {
        assert!(!IntentClass::ReadOnly.requires_confirmation());
        assert!(IntentClass::Mutating.requires_confirmation());
        assert!(!IntentClass::ReadOnly.allows_commands_in_answer());
        assert!(!IntentClass::Mutating.allows_commands_in_answer());
    }
}
