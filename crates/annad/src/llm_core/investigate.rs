//! Investigation utilities for the LLM core loop.
//!
//! This module provides helpers for the investigation phase:
//! - Command safety checks
//! - Output parsing helpers
//! - Investigation strategies

use super::Finding;

/// Check if a command is safe to execute
pub fn is_safe_command(cmd: &str) -> bool {
    let dangerous_patterns = [
        "rm -rf /",
        "rm -rf /*",
        "dd if=",
        "mkfs",
        ":(){:|:&};:",
        "> /dev/sd",
        "chmod -R 777 /",
        "chown -R",
    ];

    let cmd_lower = cmd.to_lowercase();

    // Block obviously dangerous commands
    for pattern in dangerous_patterns {
        if cmd_lower.contains(pattern) {
            return false;
        }
    }

    // Block interactive commands that won't work
    let interactive = ["vim", "nano", "vi", "emacs", "less", "more", "top", "htop", "man"];
    let first_word = cmd.split_whitespace().next().unwrap_or("");

    if interactive.contains(&first_word) {
        return false;
    }

    true
}

/// Check if findings contain evidence of a specific issue
pub fn findings_contain(findings: &[Finding], pattern: &str) -> bool {
    findings.iter().any(|f| f.output.to_lowercase().contains(&pattern.to_lowercase()))
}

/// Extract numeric value from findings (e.g., disk percentage)
pub fn extract_percentage(findings: &[Finding], context: &str) -> Option<u8> {
    for finding in findings {
        if finding.output.contains(context) {
            // Look for patterns like "85%" or "85 %"
            for word in finding.output.split_whitespace() {
                if let Some(num_str) = word.strip_suffix('%') {
                    if let Ok(num) = num_str.parse::<u8>() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

/// Summarize findings for context
pub fn summarize_findings(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "No investigation performed yet.".to_string();
    }

    let successful: Vec<_> = findings.iter().filter(|f| f.success).collect();
    let failed: Vec<_> = findings.iter().filter(|f| !f.success).collect();

    format!(
        "{} commands executed ({} successful, {} failed)",
        findings.len(),
        successful.len(),
        failed.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        assert!(is_safe_command("df -h"));
        assert!(is_safe_command("ps aux"));
        assert!(is_safe_command("systemctl status"));
        assert!(!is_safe_command("rm -rf /"));
        assert!(!is_safe_command("vim file.txt"));
        assert!(!is_safe_command("top"));
    }

    #[test]
    fn test_findings_contain() {
        let findings = vec![
            Finding {
                command: "df -h".to_string(),
                output: "Filesystem Size Used Avail Use% Mounted\n/dev/sda1 100G 85G 15G 85% /".to_string(),
                success: true,
            }
        ];

        assert!(findings_contain(&findings, "85%"));
        assert!(findings_contain(&findings, "/dev/sda1"));
        assert!(!findings_contain(&findings, "error"));
    }
}
