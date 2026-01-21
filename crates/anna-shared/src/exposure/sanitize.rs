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
    /// Phase 34A: LLM evaluator output contamination (ALL CAPS evaluator patterns)
    /// This prevents LLM internal evaluation responses from leaking into user output.
    EvaluatorContamination,
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
            Self::EvaluatorContamination => "LLM evaluator output leaked into user response",
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

// Manual command patterns - Phase 15 + Phase 22 enhancements
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
        // Direct command suggestions in code blocks
        (Regex::new(r"```\s*(sh|bash|shell)?\s*\n[^`]*\b(sudo|systemctl|pacman|nano|vim)\b").unwrap(), "[action]"),
        // Phase 22: Block command instructions (not mentions in Evidence summaries)
        // These patterns match when commands appear as instructions at line start or after prompts
        (Regex::new(r"(?i)^cat\s+/proc/").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^free\s+-[hmg]").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^journalctl\s").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^systemctl\s").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^pacman\s+-").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^apt\s+(list|show|search|install|remove)").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^df\s+-").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^du\s+-").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^lsblk\b").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^swapon\s+-").unwrap(), "[probe]"),
        (Regex::new(r"(?i)^pactl\s").unwrap(), "[probe]"),
        // Shell prompt patterns ($ or > followed by command)
        (Regex::new(r"(?i)\$\s*(cat|free|df|du|systemctl|journalctl|pacman|apt)\s").unwrap(), "[probe]"),
        (Regex::new(r"(?i)>\s*(cat|free|df|du|systemctl|journalctl|pacman|apt)\s").unwrap(), "[probe]"),
        // Forbidden phrases
        (Regex::new(r"(?i)\bwould you like me to\b").unwrap(), "[offer]"),
        (Regex::new(r"(?i)\bI need to handle it myself\b").unwrap(), "[offer]"),
        (Regex::new(r"(?i)\bI can help.+but.+handle it\b").unwrap(), "[offer]"),
    ]
});

// Phase 34A: LLM evaluator contamination patterns
// These patterns detect when LLM internal evaluation/rating output leaks into responses
static EVALUATOR_CONTAMINATION_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // ALL CAPS evaluator keywords that indicate leaked evaluation output
        (Regex::new(r"\bCOMPLETE\b").unwrap(), ""),
        (Regex::new(r"\bINCOMPLETE\b").unwrap(), ""),
        (Regex::new(r"\bCONFIDENCE[:\s]+\d+").unwrap(), ""),
        (Regex::new(r"\bMISSING:\s").unwrap(), ""),
        (Regex::new(r"\bEXPLANATION:\s").unwrap(), ""),
        (Regex::new(r"\bANNA:\s+NONE\b").unwrap(), ""),
        // Evaluation formatting patterns
        (Regex::new(r"\b(YES|NO)/?(YES|NO)?/?(NA)?\b").unwrap(), ""),
        // Rating scales that indicate evaluation output
        (Regex::new(r"\bRATING:\s*\d+/\d+").unwrap(), ""),
        (Regex::new(r"\bSCORE:\s*\d+").unwrap(), ""),
        // "THE RESPONSE IS" evaluation phrase
        (Regex::new(r"(?i)\bTHE RESPONSE IS\s+(COMPLETE|INCOMPLETE|SUFFICIENT|INSUFFICIENT)\b").unwrap(), ""),
        // Structured evaluation output markers
        (Regex::new(r"^\d+\.\s+(Does|Is|Are|Can)\s+.+\?\s+(YES|NO)").unwrap(), ""),
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

    // Check evaluator contamination patterns (Phase 34A)
    for (re, replacement) in EVALUATOR_CONTAMINATION_PATTERNS.iter() {
        for m in re.find_iter(text) {
            violations.push(Violation {
                pattern: ForbiddenPattern::EvaluatorContamination,
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
    fn test_pattern_detection() {
        // Clean text passes
        assert!(sanitize_dialogue("Processing request. Analysis complete.").is_clean);
        // Urgency detected
        assert!(!sanitize_dialogue("This is critical and urgent.").is_clean);
        // Authority detected
        assert!(!sanitize_dialogue("You must restart the service.").is_clean);
        // Consciousness detected
        assert!(!sanitize_dialogue("I think the problem is here.").is_clean);
        // Alarm detected
        assert!(!sanitize_dialogue("Warning! Dangerous operation.").is_clean);
    }

    #[test]
    fn test_manual_commands_detected() {
        // Phase 15: Manual command patterns
        assert!(!sanitize_dialogue("Run: sudo pacman -Syu").is_clean);
        assert!(!sanitize_dialogue("sudo systemctl restart nginx").is_clean);
        assert!(!sanitize_dialogue("Run this command: df -h").is_clean);
        assert!(!sanitize_dialogue("Edit the file /etc/fstab").is_clean);
        assert!(!sanitize_dialogue("nano /etc/hosts").is_clean);
    }

    #[test]
    fn test_clean_patterns_pass() {
        // Phase 22: Updated - evidence should use summaries, not raw commands
        let approved = [
            "Request processed.", "Analysis complete.", "[probe]",
            "The service will be restarted.", "Configuration changes have been applied.",
            "Evidence: disk usage, memory info", "Analysis shows 45% disk usage.",
            "Your swap usage is 1.2GB.", "PipeWire is running.",
        ];
        for text in approved {
            assert!(sanitize_dialogue(text).is_clean, "Should pass: {}", text);
        }
    }

    // ==========================================================================
    // Phase 34A: Evaluator Contamination Detection
    // ==========================================================================

    #[test]
    fn test_evaluator_contamination_detected() {
        // ALL CAPS evaluator output patterns should be blocked
        assert!(!sanitize_dialogue("COMPLETE, CONFIDENCE 80").is_clean);
        assert!(!sanitize_dialogue("INCOMPLETE, need more data").is_clean);
        assert!(!sanitize_dialogue("EXPLANATION: the response is valid").is_clean);
        assert!(!sanitize_dialogue("MISSING: specific values").is_clean);
        assert!(!sanitize_dialogue("ANNA: NONE").is_clean);
        assert!(!sanitize_dialogue("THE RESPONSE IS COMPLETE").is_clean);
    }

    #[test]
    fn test_evaluator_contamination_partial_matches() {
        // These should NOT be blocked - normal text containing evaluator-like words
        // (in lowercase or natural context)
        assert!(sanitize_dialogue("The installation is complete.").is_clean);
        assert!(sanitize_dialogue("Your configuration appears incomplete.").is_clean);
        assert!(sanitize_dialogue("Anna cannot process this request.").is_clean);
    }

    // ==========================================================================
    // Phase 34A: Capability Abstain Messages Must Pass Filter
    // ==========================================================================

    #[test]
    fn test_capability_abstain_messages_pass() {
        // These are real abstain messages from display_scale.rs
        // They MUST pass sanitization (not be blocked)
        let abstain_messages = [
            "Your system uses SDDM. GDM scaling is not applicable.",
            "No monitors.xml found. Anna needs your display settings configured first.",
            "What to do: Open Settings > Displays, set your preferred scale (e.g., 200%), and click Apply.",
            "Your monitors.xml exists but has no scale setting.",
            "GDM is already configured with scale 2. Log out to verify.",
        ];
        for msg in abstain_messages {
            assert!(
                sanitize_dialogue(msg).is_clean,
                "Capability abstain message should pass: {}",
                msg
            );
        }
    }

    #[test]
    fn test_capability_confirmation_messages_pass() {
        // Confirmation messages from ActionPlan formatting should pass
        let confirmation_messages = [
            "Detected:\n  This will copy your monitors.xml to GDM.",
            "Plan:\n  Step 1: Create GDM configuration directory",
            "Step 2: Copy display configuration to GDM [requires approval]",
            "Step 3: Set file ownership for GDM [requires approval]",
            "Anna: Proceed? (yes/no)",
        ];
        for msg in confirmation_messages {
            assert!(
                sanitize_dialogue(msg).is_clean,
                "Capability confirmation message should pass: {}",
                msg
            );
        }
    }
}
