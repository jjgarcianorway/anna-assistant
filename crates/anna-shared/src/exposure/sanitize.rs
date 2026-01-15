//! Dialogue Sanitization - Forbidden pattern detection and enforcement.
//!
//! Ensures all user-visible dialogue:
//! - Avoids urgency language
//! - Avoids authority escalation tone
//! - Avoids consciousness/intent attribution
//! - Maintains calm, professional tone

use regex::Regex;
use std::sync::LazyLock;

/// Categories of forbidden patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForbiddenPattern {
    /// Urgency language (critical, urgent, immediately, etc.)
    Urgency,
    /// Authority escalation (must, required, mandatory, etc.)
    Authority,
    /// Consciousness attribution (thinks, decides, wants, feels, etc.)
    Consciousness,
    /// Alarm language (error, failure, danger, etc. in dialogue)
    Alarm,
    /// Manual commands (sudo, shell commands, edit instructions)
    /// Added in Phase 15: Anna executes actions, not the user.
    ManualCommands,
}

impl ForbiddenPattern {
    /// Human-readable description of the pattern category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Urgency => "Urgency language creates unnecessary stress",
            Self::Authority => "Authority language implies Anna has power over user",
            Self::Consciousness => "Consciousness language implies Anna is sentient",
            Self::Alarm => "Alarm language creates anxiety",
            Self::ManualCommands => "Manual commands violate contract: Anna executes, not user",
        }
    }
}

/// Result of sanitization check.
#[derive(Debug, Clone)]
pub struct SanitizationResult {
    /// Whether the text is clean (no violations).
    pub is_clean: bool,
    /// List of violations found.
    pub violations: Vec<Violation>,
    /// Suggested replacement text (if violations found).
    pub suggested: Option<String>,
}

/// A single violation found in text.
#[derive(Debug, Clone)]
pub struct Violation {
    /// The pattern category violated.
    pub pattern: ForbiddenPattern,
    /// The matched text.
    pub matched: String,
    /// Position in the original string.
    pub position: usize,
    /// Suggested replacement.
    pub replacement: &'static str,
}

// Urgency patterns
static URGENCY_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?i)\b(critical|critically)\b").unwrap(), "important"),
        (Regex::new(r"(?i)\b(urgent|urgently)\b").unwrap(), "timely"),
        (Regex::new(r"(?i)\bimmediately\b").unwrap(), "now"),
        (Regex::new(r"(?i)\b(asap|a\.s\.a\.p\.)\b").unwrap(), "soon"),
        (Regex::new(r"(?i)\bemergency\b").unwrap(), "issue"),
        (Regex::new(r"(?i)\bright away\b").unwrap(), "now"),
        (Regex::new(r"(?i)\btime.?sensitive\b").unwrap(), "timely"),
    ]
});

// Authority patterns
static AUTHORITY_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?i)\byou must\b").unwrap(), "you can"),
        (Regex::new(r"(?i)\brequired to\b").unwrap(), "able to"),
        (Regex::new(r"(?i)\bmandatory\b").unwrap(), "recommended"),
        (Regex::new(r"(?i)\byou have to\b").unwrap(), "you can"),
        (Regex::new(r"(?i)\byou need to\b").unwrap(), "you can"),
        (Regex::new(r"(?i)\byou should\b").unwrap(), "you may"),
        (Regex::new(r"(?i)\bdo this now\b").unwrap(), "consider this"),
    ]
});

// Consciousness patterns (Anna as subject)
static CONSCIOUSNESS_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?i)\b(I|Anna) (think|thinks)\b").unwrap(), "analysis suggests"),
        (Regex::new(r"(?i)\b(I|Anna) (decide|decides)\b").unwrap(), "the system routes"),
        (Regex::new(r"(?i)\b(I|Anna) (want|wants)\b").unwrap(), "the request is"),
        (Regex::new(r"(?i)\b(I|Anna) (feel|feels)\b").unwrap(), "the status is"),
        (Regex::new(r"(?i)\b(I|Anna) (believe|believes)\b").unwrap(), "evidence suggests"),
        (Regex::new(r"(?i)\b(I|Anna) (know|knows)\b").unwrap(), "data shows"),
        (Regex::new(r"(?i)\bmy opinion\b").unwrap(), "the analysis"),
        (Regex::new(r"(?i)\bI'm (concerned|worried)\b").unwrap(), "there may be"),
        (Regex::new(r"(?i)\bI'm (happy|glad|pleased)\b").unwrap(), "completed"),
    ]
});

// Alarm patterns (in dialogue context, not error messages)
static ALARM_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r"(?i)\bdanger(ous)?\b").unwrap(), "notable"),
        (Regex::new(r"(?i)\bwarning!\b").unwrap(), "note:"),
        (Regex::new(r"(?i)\balert!\b").unwrap(), "update:"),
        (Regex::new(r"(?i)\bpanic\b").unwrap(), "issue"),
        (Regex::new(r"(?i)\bsevere\b").unwrap(), "significant"),
    ]
});

// Manual command patterns - Phase 15
// Anna executes actions, not the user. These patterns block manual instructions.
static MANUAL_COMMAND_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // sudo instructions
        (Regex::new(r"(?i)\bsudo\s").unwrap(), "[action]"),
        (Regex::new(r"(?i)\brun:\s*sudo\b").unwrap(), "[action]"),
        (Regex::new(r"(?i)\btry:\s*sudo\b").unwrap(), "[action]"),
        // Shell command patterns in prose
        (Regex::new(r"(?i)\brun\s+(this|the|these)\s+command").unwrap(), "[action]"),
        (Regex::new(r"(?i)\bexecute\s+(this|the|these)\s+command").unwrap(), "[action]"),
        (Regex::new(r"(?i)\btype\s+(this|the|these)\s+command").unwrap(), "[action]"),
        // File editing instructions
        (Regex::new(r"(?i)\bedit\s+(this|the|your)\s+file").unwrap(), "[action]"),
        (Regex::new(r"(?i)\bopen\s+.+\s+in\s+(nano|vim|vi|emacs|gedit|kate)").unwrap(), "[action]"),
        (Regex::new(r"(?i)\b(nano|vim|vi)\s+/").unwrap(), "[action]"),
        // Manual config instructions
        (Regex::new(r"(?i)\badd\s+(this|the|these)\s+(line|entry|section)").unwrap(), "[action]"),
        (Regex::new(r"(?i)\bmodify\s+(this|the|your)\s+(file|config)").unwrap(), "[action]"),
        (Regex::new(r"(?i)\bchange\s+(this|the)\s+(line|value)\s+to\b").unwrap(), "[action]"),
        // Direct command suggestions in code blocks are caught by shell patterns
        (Regex::new(r"```\s*(sh|bash|shell)?\s*\n[^`]*\b(sudo|systemctl|pacman|nano|vim)\b").unwrap(), "[action]"),
    ]
});

/// Validate text against forbidden patterns.
pub fn sanitize_dialogue(text: &str) -> SanitizationResult {
    let mut violations = Vec::new();

    // Check urgency patterns
    for (re, replacement) in URGENCY_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::Urgency,
                matched: m.as_str().to_string(),
                position: m.start(),
                replacement,
            });
        }
    }

    // Check authority patterns
    for (re, replacement) in AUTHORITY_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::Authority,
                matched: m.as_str().to_string(),
                position: m.start(),
                replacement,
            });
        }
    }

    // Check consciousness patterns
    for (re, replacement) in CONSCIOUSNESS_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::Consciousness,
                matched: m.as_str().to_string(),
                position: m.start(),
                replacement,
            });
        }
    }

    // Check alarm patterns
    for (re, replacement) in ALARM_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::Alarm,
                matched: m.as_str().to_string(),
                position: m.start(),
                replacement,
            });
        }
    }

    // Check manual command patterns (Phase 15)
    for (re, replacement) in MANUAL_COMMAND_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::ManualCommands,
                matched: m.as_str().to_string(),
                position: m.start(),
                replacement,
            });
        }
    }

    let is_clean = violations.is_empty();
    let suggested = if is_clean {
        None
    } else {
        Some(apply_replacements(text, &violations))
    };

    SanitizationResult {
        is_clean,
        violations,
        suggested,
    }
}

/// Apply all replacements to produce sanitized text.
fn apply_replacements(text: &str, violations: &[Violation]) -> String {
    let mut result = text.to_string();

    // Apply replacements in reverse order to preserve positions
    let mut sorted_violations: Vec<_> = violations.iter().collect();
    sorted_violations.sort_by(|a, b| b.position.cmp(&a.position));

    for v in sorted_violations {
        // Use regex replacement for case preservation
        result = result.replacen(&v.matched, v.replacement, 1);
    }

    result
}

/// Validate that text follows wording guidelines.
/// Returns Ok(()) if valid, Err with description if not.
pub fn validate_wording(text: &str) -> Result<(), String> {
    let result = sanitize_dialogue(text);
    if result.is_clean {
        Ok(())
    } else {
        let issues: Vec<String> = result
            .violations
            .iter()
            .map(|v| format!("'{}' ({:?})", v.matched, v.pattern))
            .collect();
        Err(format!("Forbidden patterns found: {}", issues.join(", ")))
    }
}

/// Check if text is safe for user display.
pub fn is_safe_for_display(text: &str) -> bool {
    sanitize_dialogue(text).is_clean
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text_passes() {
        let clean = "Processing request. Analysis complete.";
        let result = sanitize_dialogue(clean);
        assert!(result.is_clean);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_urgency_detected() {
        let text = "This is critical and needs immediate attention.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::Urgency));
    }

    #[test]
    fn test_authority_detected() {
        let text = "You must restart the service.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::Authority));
    }

    #[test]
    fn test_consciousness_detected() {
        let text = "I think the problem is in the config.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::Consciousness));
    }

    #[test]
    fn test_alarm_detected() {
        let text = "Warning! Dangerous operation detected.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::Alarm));
    }

    #[test]
    fn test_suggested_replacement() {
        let text = "You must do this immediately.";
        let result = sanitize_dialogue(text);
        assert!(result.suggested.is_some());
        let suggested = result.suggested.unwrap();
        assert!(!suggested.contains("must"));
        assert!(!suggested.contains("immediately"));
    }

    #[test]
    fn test_validate_wording_ok() {
        assert!(validate_wording("Request processed.").is_ok());
    }

    #[test]
    fn test_validate_wording_err() {
        assert!(validate_wording("I think this is urgent.").is_err());
    }

    #[test]
    fn test_case_insensitive() {
        let text = "CRITICAL EMERGENCY";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_professional_alternatives() {
        // These should all pass as they use approved language
        let approved = [
            "Request processed.",
            "Analysis complete.",
            "The system routed the request.",
            "Data shows the disk is full.",
            "You can restart the service.",
            "Consider reviewing the logs.",
        ];
        for text in approved {
            assert!(sanitize_dialogue(text).is_clean, "Failed: {}", text);
        }
    }

    #[test]
    fn test_no_false_positives_in_commands() {
        // Commands in probes should not trigger consciousness detection
        let text = "[probe] systemctl status";
        let result = sanitize_dialogue(text);
        assert!(result.is_clean);
    }

    // Phase 15: Manual command detection tests
    #[test]
    fn test_manual_command_sudo_detected() {
        let text = "Run: sudo pacman -Syu to update.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::ManualCommands));
    }

    #[test]
    fn test_manual_command_inline_sudo() {
        let text = "You can fix this with sudo systemctl restart nginx";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::ManualCommands));
    }

    #[test]
    fn test_manual_command_run_this_command() {
        let text = "Run this command to fix it: df -h";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::ManualCommands));
    }

    #[test]
    fn test_manual_command_edit_file() {
        let text = "Edit the file /etc/fstab with your changes.";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::ManualCommands));
    }

    #[test]
    fn test_manual_command_nano_vim() {
        let text = "Open it with nano /etc/hosts";
        let result = sanitize_dialogue(text);
        assert!(!result.is_clean);
        assert!(result.violations.iter().any(|v| v.pattern == ForbiddenPattern::ManualCommands));
    }

    #[test]
    fn test_clean_abstract_actions_pass() {
        // These describe WHAT Anna will do, not instructions for user
        let clean_texts = [
            "I'll update the system configuration.",
            "The service will be restarted.",
            "Configuration changes have been applied.",
            "The file has been modified with the new settings.",
            "Power management policy updated.",
        ];
        for text in clean_texts {
            assert!(sanitize_dialogue(text).is_clean, "Should pass: {}", text);
        }
    }
}
