//! Help output fetching (v0.0.422).

use std::process::Command;

use super::types::SourceFetchResult;
use super::utils::{is_dangerous_command, is_safe_name, looks_like_help, truncate_help};

/// Fetch help output for a command
pub fn fetch_help_output(command: &str) -> Option<SourceFetchResult> {
    // Validate command name
    if !is_safe_name(command) {
        return None;
    }

    // Check against dangerous commands
    if is_dangerous_command(command) {
        return None;
    }

    // Try --help first
    let output = Command::new(command).arg("--help").output();

    if let Ok(out) = output {
        let content = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            // Some commands output help to stderr
            String::from_utf8_lossy(&out.stderr).to_string()
        };

        if looks_like_help(&content) {
            let truncated = truncate_help(&content, 50);
            return Some(SourceFetchResult::new(
                truncated,
                &format!("{} --help", command),
            ));
        }
    }

    // Try -h as fallback
    let output = Command::new(command).arg("-h").output();

    if let Ok(out) = output {
        let content = if out.status.success() {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            String::from_utf8_lossy(&out.stderr).to_string()
        };

        if looks_like_help(&content) {
            let truncated = truncate_help(&content, 50);
            return Some(SourceFetchResult::new(
                truncated,
                &format!("{} -h", command),
            ));
        }
    }

    None
}
