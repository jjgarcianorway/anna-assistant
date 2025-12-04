//! Transcript Validation v0.0.72
//!
//! Ensures human mode output is clean and debug mode has expected content.

use regex::Regex;

/// Forbidden patterns in human mode output
pub const FORBIDDEN_HUMAN_PATTERNS: &[&str] = &[
    // Evidence IDs
    r"\[E\d+\]",
    // Tool name patterns
    r"hw_snapshot_",
    r"sw_snapshot_",
    r"status_snapshot",
    r"_snapshot_",
    r"_summary",
    r"_probe",
    // Raw commands
    r"journalctl",
    r"systemctl\s",
    r"nmcli\s",
    r"btrfs\s",
    r"smartctl",
    r"pacman\s-",
    r"resolvectl",
    r"hostnamectl",
    r"iw\s",
    r"ip\s+addr",
    r"ip\s+route",
    // Internal terms
    r"Parse\s+error",
    r"parse\s+error",
    r"ParseError",
    r"deterministic\s+fallback",
    r"deterministic_fallback",
    r"fallback_used",
    r"CANONICAL",
    r"tool=",
    r"evidence_id",
    r"parse_attempts",
];

/// Literal forbidden strings in human mode
pub const FORBIDDEN_HUMAN_LITERALS: &[&str] = &[
    "[E1]",
    "[E2]",
    "[E3]",
    "[E4]",
    "[E5]",
    "[E6]",
    "[E7]",
    "[E8]",
    "[E9]",
    "[E10]",
    "_snapshot",
    "_summary",
    "_probe",
    "deterministic fallback",
    "Parse error",
    "parse error",
];

/// Validate human mode output has no forbidden patterns
pub fn validate_human_output(lines: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    let content = lines.join("\n");

    // Check literal strings first (faster)
    for literal in FORBIDDEN_HUMAN_LITERALS {
        if content.contains(literal) {
            violations.push(format!(
                "Found forbidden literal '{}' in human output",
                literal
            ));
        }
    }

    // Check regex patterns
    for pattern in FORBIDDEN_HUMAN_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(m) = re.find(&content) {
                violations.push(format!(
                    "Found forbidden pattern '{}' matching '{}' in human output",
                    pattern,
                    m.as_str()
                ));
            }
        }
    }

    violations
}

/// Validate debug mode contains expected internal details
pub fn validate_debug_has_internals(lines: &[String]) -> DebugValidation {
    let content = lines.join("\n");

    DebugValidation {
        has_tool_names: content.contains("tool="),
        has_evidence_ids: Regex::new(r"\[E\d+\]")
            .map(|re| re.is_match(&content))
            .unwrap_or(false),
        has_timestamps: Regex::new(r"\d{2}:\d{2}:\d{2}")
            .map(|re| re.is_match(&content))
            .unwrap_or(false),
        has_duration: content.contains("ms)") || content.contains("ms "),
    }
}

/// Debug mode validation result
#[derive(Debug, Clone)]
pub struct DebugValidation {
    pub has_tool_names: bool,
    pub has_evidence_ids: bool,
    pub has_timestamps: bool,
    pub has_duration: bool,
}

impl DebugValidation {
    pub fn is_valid(&self) -> bool {
        // Debug mode should have at least tool names or evidence IDs
        self.has_tool_names || self.has_evidence_ids
    }
}

/// Quick check if a single line is clean for human mode
pub fn is_line_clean_for_human(line: &str) -> bool {
    // Fast literal check
    for literal in FORBIDDEN_HUMAN_LITERALS {
        if line.contains(literal) {
            return false;
        }
    }

    // Slower regex check for edge cases
    for pattern in FORBIDDEN_HUMAN_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(line) {
                return false;
            }
        }
    }

    true
}

/// Strip ANSI escape codes for validation
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;

    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if c == 'm' {
                in_escape = false;
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_clean_human_output() {
        let lines = vec![
            "[service desk] Opening case and reviewing request.".to_string(),
            "[network] Evidence from network status: WiFi connected".to_string(),
            "Reliability: 85% (High) - good evidence coverage".to_string(),
        ];
        let violations = validate_human_output(&lines);
        assert!(
            violations.is_empty(),
            "Expected no violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_validate_catches_evidence_id() {
        let lines = vec!["[network] [E1] network_status: carrier=true".to_string()];
        let violations = validate_human_output(&lines);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("E1")));
    }

    #[test]
    fn test_validate_catches_tool_name() {
        let lines = vec!["[anna] Checking hw_snapshot_summary...".to_string()];
        let violations = validate_human_output(&lines);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.contains("hw_snapshot_")));
    }

    #[test]
    fn test_validate_catches_parse_error() {
        let lines = vec!["[translator] Parse error: Invalid format".to_string()];
        let violations = validate_human_output(&lines);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_validate_catches_deterministic_fallback() {
        let lines = vec!["[anna] Using deterministic fallback for classification".to_string()];
        let violations = validate_human_output(&lines);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_debug_validation() {
        let lines = vec![
            "10:30:45.123 [tool_call] tool=hw_snapshot_summary".to_string(),
            "10:30:45.200 [evidence] [E1] hw_snapshot_summary (77ms)".to_string(),
        ];
        let validation = validate_debug_has_internals(&lines);
        assert!(validation.has_tool_names);
        assert!(validation.has_evidence_ids);
        assert!(validation.has_timestamps);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_is_line_clean() {
        assert!(is_line_clean_for_human(
            "[anna] Your CPU is Intel i9-14900HX"
        ));
        assert!(!is_line_clean_for_human("[anna] [E1] hw_snapshot_summary"));
        assert!(!is_line_clean_for_human("Parse error: invalid format"));
    }

    #[test]
    fn test_strip_ansi() {
        let colored = "\x1b[36m[anna]\x1b[0m test";
        assert_eq!(strip_ansi(colored), "[anna] test");
    }
}
