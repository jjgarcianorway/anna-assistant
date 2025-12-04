//! Validation for Transcript System v0.0.70
//!
//! Ensures human mode output doesn't leak internal details.

/// Forbidden terms in human mode output
pub const FORBIDDEN_HUMAN: &[&str] = &[
    // Evidence IDs
    "[E1]", "[E2]", "[E3]", "[E4]", "[E5]", "[E6]", "[E7]", "[E8]", "[E9]",
    // Tool patterns
    "_snapshot", "_summary", "_probe", "_check", "_status", "_info",
    // Raw commands
    "journalctl", "systemctl", "nmcli", "btrfs ", "smartctl", "pacman ",
    "resolvectl", "hostnamectl", "iw ", "ip addr", "ip route",
    // Parse/internal
    "Parse error", "parse error", "ParseError",
    "deterministic fallback", "fallback_used",
    "CANONICAL", "tool=", "evidence_id",
];

/// Validate human mode output
pub fn validate_human_output(lines: &[String]) -> Vec<String> {
    let mut violations = Vec::new();
    for line in lines {
        for term in FORBIDDEN_HUMAN {
            if line.contains(term) {
                violations.push(format!("Found '{}' in: {}", term, line));
            }
        }
    }
    violations
}

/// Validate debug mode has expected content
pub fn validate_debug_has_internals(lines: &[String]) -> bool {
    let content = lines.join("\n");
    content.contains("tool=")
        || content.contains("[tool]")
        || content.contains("CANONICAL")
        || content.contains("[translator]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_human_output() {
        let good_lines = vec![
            "[you] What is my WiFi status?".to_string(),
            "[networking] Evidence from network status snapshot: WiFi connected".to_string(),
        ];
        assert!(validate_human_output(&good_lines).is_empty());

        let bad_lines = vec![
            "[networking] [E1] network_status: carrier=true".to_string(),
        ];
        let violations = validate_human_output(&bad_lines);
        assert!(!violations.is_empty());
    }
}
