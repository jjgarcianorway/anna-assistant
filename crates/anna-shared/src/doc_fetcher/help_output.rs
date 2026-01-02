//! Help output (--help/-h) fetching functionality.

use crate::evidence_engine::{DocSnippet, DocSource};
use std::process::Command;

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Help output line limit
const HELP_LINE_LIMIT: usize = 40;

/// Fetch --help or -h output for a command
pub fn fetch_help_output(command: &str) -> Option<DocSnippet> {
    // Only try for commands that look safe
    if !is_safe_help_command(command) {
        return None;
    }

    // Try --help first, then -h
    let output = Command::new("sh")
        .args([
            "-c",
            &format!(
                "{} --help 2>&1 | head -{} || {} -h 2>&1 | head -{}",
                super::utils::shell_escape(command),
                HELP_LINE_LIMIT,
                super::utils::shell_escape(command),
                HELP_LINE_LIMIT
            ),
        ])
        .output()
        .ok()?;

    let content = String::from_utf8_lossy(&output.stdout);
    if content.trim().is_empty() || content.contains("command not found") {
        return None;
    }

    // Check if it looks like help output
    let content_lower = content.to_lowercase();
    if !content_lower.contains("usage")
        && !content_lower.contains("options")
        && !content_lower.contains("--help")
    {
        return None;
    }

    Some(
        DocSnippet::new(
            DocSource::HelpOutput,
            &format!("{} --help", command),
            &super::utils::truncate(&content, MAX_SNIPPET),
            &format!("{} --help", command),
        )
        .with_relevance(70),
    )
}

/// Check if command is safe to run with --help
fn is_safe_help_command(cmd: &str) -> bool {
    let clean = cmd.split_whitespace().next().unwrap_or(cmd);

    // Blocklist dangerous commands
    let dangerous = [
        "rm", "dd", "mkfs", "fdisk", "parted", "sudo", "su", "chmod", "chown", "kill", "reboot",
        "shutdown", "halt",
    ];

    !dangerous.contains(&clean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_help_command() {
        assert!(is_safe_help_command("pacman"));
        assert!(is_safe_help_command("systemctl"));
        assert!(!is_safe_help_command("rm"));
        assert!(!is_safe_help_command("sudo"));
    }
}
