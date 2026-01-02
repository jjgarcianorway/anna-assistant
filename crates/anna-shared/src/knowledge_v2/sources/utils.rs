//! Utility functions for source fetching (v0.0.422).

/// Check if name is safe (no shell injection)
pub fn is_safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() < 100
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Check if command is dangerous
pub fn is_dangerous_command(cmd: &str) -> bool {
    const DANGEROUS: &[&str] = &[
        "rm", "dd", "mkfs", "fdisk", "parted", "sudo", "su", "chmod", "chown", "kill", "pkill",
        "killall", "reboot", "shutdown", "halt", "poweroff", "mv", "cp", "shred", "wipefs",
    ];
    DANGEROUS.contains(&cmd)
}

/// Check if output looks like help text
pub fn looks_like_help(content: &str) -> bool {
    let lower = content.to_lowercase();
    let has_help_indicators = lower.contains("usage")
        || lower.contains("options")
        || lower.contains("--help")
        || lower.contains("commands")
        || lower.contains("arguments");

    let not_error = !lower.contains("command not found")
        && !lower.contains("no such file")
        && !lower.contains("permission denied");

    has_help_indicators && not_error && content.len() >= 50
}

/// Truncate help output to max lines
pub fn truncate_help(content: &str, max_lines: usize) -> String {
    content
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

/// Truncate document to max lines
pub fn truncate_doc(content: &str, max_lines: usize) -> String {
    content
        .lines()
        .take(max_lines)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_name() {
        assert!(is_safe_name("systemctl"));
        assert!(is_safe_name("vim-enhanced"));
        assert!(is_safe_name("python3.11"));
        assert!(!is_safe_name("rm -rf /"));
        assert!(!is_safe_name("; rm -rf /"));
        assert!(!is_safe_name(""));
    }

    #[test]
    fn test_dangerous_commands() {
        assert!(is_dangerous_command("rm"));
        assert!(is_dangerous_command("dd"));
        assert!(is_dangerous_command("sudo"));
        assert!(!is_dangerous_command("systemctl"));
        assert!(!is_dangerous_command("pacman"));
    }

    #[test]
    fn test_looks_like_help() {
        assert!(looks_like_help(
            "Usage: command [options]\n\nOptions:\n  --help\n  -v, --version"
        ));
        assert!(!looks_like_help("command not found"));
        assert!(!looks_like_help("short"));
    }
}
