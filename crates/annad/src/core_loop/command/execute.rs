//! Command execution with caching and timeout support.

use anyhow::Result;
use std::process::Command;
use tracing::debug;

use super::output::{get_alternative_command, strip_ansi_codes, truncate_output};
use crate::core_loop::cache::{
    cache_command, get_cached_command, get_perf_config, is_known_failed_command,
};

/// v0.0.938: Maximum output lines before truncation
pub const MAX_OUTPUT_LINES: usize = 100;
/// v0.0.938: Maximum output characters before truncation
pub const MAX_OUTPUT_CHARS: usize = 8000;

/// v0.0.925: Get command-specific timeout based on command type
pub fn get_command_timeout(cmd: &str) -> u64 {
    let base_timeout = get_perf_config().command_timeout_secs;
    let cmd_lower = cmd.to_lowercase();

    // Package managers need more time (downloads, installs)
    if cmd_lower.starts_with("pacman ")
        || cmd_lower.starts_with("yay ")
        || cmd_lower.starts_with("paru ")
        || cmd_lower.starts_with("apt ")
        || cmd_lower.starts_with("dnf ")
        || cmd_lower.starts_with("zypper ")
    {
        return 120.max(base_timeout);
    }

    // Recursive searches can take a while
    if cmd_lower.contains("find ") && (cmd_lower.contains(" /") || cmd_lower.contains(" ~"))
        || cmd_lower.contains("grep -r")
        || cmd_lower.contains("rg ")
    {
        return 60.max(base_timeout);
    }

    // System updates need even more time
    if cmd_lower.contains("-syu") || cmd_lower.contains("upgrade") || cmd_lower.contains("update")
    {
        return 180.max(base_timeout);
    }

    // Network commands with potential delays
    if cmd_lower.starts_with("ping ")
        || cmd_lower.starts_with("curl ")
        || cmd_lower.starts_with("wget ")
        || cmd_lower.starts_with("ssh ")
    {
        return 30.max(base_timeout);
    }

    // Quick read-only commands can use shorter timeout
    if cmd_lower.starts_with("cat ")
        || cmd_lower.starts_with("head ")
        || cmd_lower.starts_with("tail ")
        || cmd_lower.starts_with("echo ")
        || cmd_lower.starts_with("ls ")
        || cmd_lower.starts_with("stat ")
    {
        return 10.min(base_timeout);
    }

    base_timeout
}

/// Execute a shell command and return its output
/// v0.0.919: Added configurable timeout support
/// v0.0.921: Added negative learning (skip known-failed commands)
pub fn execute_command(cmd: &str) -> Result<String> {
    // Check cache first
    if let Some(cached) = get_cached_command(cmd) {
        return Ok(cached);
    }

    // v0.0.921: Check if this command is known to fail
    if let Some(error_type) = is_known_failed_command(cmd) {
        debug!("Skipping known-failed command: {} ({})", cmd, error_type);
        return Ok(format!("[SKIPPED] Known failed command: {}", error_type));
    }

    // v0.0.925: Get command-specific timeout
    let timeout_secs = get_command_timeout(cmd);

    // Use timeout wrapper to prevent hanging commands
    let output = Command::new("timeout")
        .arg(format!("{}s", timeout_secs))
        .arg("sh")
        .arg("-c")
        .arg(cmd)
        .output()?;

    // Check if command timed out (exit code 124)
    if output.status.code() == Some(124) {
        return Ok(format!(
            "[TIMEOUT] Command timed out after {}s: {}",
            timeout_secs, cmd
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if stdout.trim().is_empty() && !stderr.trim().is_empty() {
        stderr
    } else {
        stdout
    };

    // v0.0.925: If empty output and no error, try alternative command
    if result.trim().is_empty() && output.status.success() {
        if let Some(alt_cmd) = get_alternative_command(cmd) {
            debug!(
                "Empty output from '{}', trying alternative: {}",
                cmd, alt_cmd
            );
            let alt_output = Command::new("timeout")
                .arg(format!("{}s", timeout_secs))
                .arg("sh")
                .arg("-c")
                .arg(&alt_cmd)
                .output();

            if let Ok(alt_out) = alt_output {
                let alt_stdout = String::from_utf8_lossy(&alt_out.stdout).to_string();
                if !alt_stdout.trim().is_empty() {
                    let cleaned = strip_ansi_codes(&alt_stdout);
                    cache_command(cmd, &cleaned);
                    return Ok(cleaned);
                }
            }
        }
    }

    let cleaned = strip_ansi_codes(&result);
    // v0.0.938: Truncate very long outputs to save LLM context
    let truncated = truncate_output(&cleaned, MAX_OUTPUT_LINES, MAX_OUTPUT_CHARS);
    cache_command(cmd, &truncated);
    Ok(truncated)
}
