//! Privilege escalation with friendly user interaction

use anyhow::{Context, Result, bail};
use std::io::{self, Write};
use std::process::Command;

/// Represents a request for elevated privileges
#[derive(Debug, Clone)]
pub struct PrivilegeRequest {
    /// Human-readable explanation of why privileges are needed
    pub reason: String,
    /// The command that needs to run with privileges
    pub command: String,
    /// Arguments for the command
    pub args: Vec<String>,
}

impl PrivilegeRequest {
    pub fn new(reason: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            command: command.into(),
            args: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
}

/// Check if we're running as root
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Check if a command needs privilege escalation
pub fn needs_privilege(path: &str) -> bool {
    // Paths that typically require root access
    let privileged_paths = [
        "/etc/",
        "/usr/local/",
        "/usr/bin/",
        "/usr/sbin/",
        "/var/lib/",
        "/var/log/",
        "/run/",
        "/lib/systemd/",
    ];

    privileged_paths.iter().any(|prefix| path.starts_with(prefix))
}

/// Request privilege escalation with friendly explanation
///
/// This function:
/// 1. Checks if we're already root (if so, returns Ok immediately)
/// 2. Explains to the user why privileges are needed
/// 3. Asks for confirmation (if config.confirm_privilege is true)
/// 4. Executes the command with sudo
/// 5. Returns a friendly error message if something goes wrong
pub fn request_privilege(request: PrivilegeRequest) -> Result<()> {
    // If we're already root, no need to escalate
    if is_root() {
        return Ok(());
    }

    let config = crate::config::load_config();

    // Explain why we need privileges
    crate::messaging::anna_narrative(format!(
        "I need administrator rights to {}",
        request.reason
    ));

    // Ask for confirmation if configured
    if config.confirm_privilege {
        crate::messaging::anna_info("May I proceed with sudo?");
        print!("  [Y/n] ");
        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        let response = response.trim().to_lowercase();

        if !response.is_empty() && response != "y" && response != "yes" {
            bail!("User declined privilege escalation");
        }
    }

    // Execute with sudo
    crate::messaging::anna_info("Requesting administrator privileges...");

    let status = Command::new("sudo")
        .arg(&request.command)
        .args(&request.args)
        .status()
        .context("Failed to execute sudo command")?;

    if !status.success() {
        crate::messaging::anna_error(format!(
            "The command failed with exit code: {}",
            status.code().unwrap_or(-1)
        ));
        bail!("Command execution failed");
    }

    crate::messaging::anna_ok("Done!");

    Ok(())
}

/// Execute a command, automatically escalating privileges if needed
///
/// This is a convenience wrapper that:
/// - Detects if a path needs privileges
/// - Runs directly if we're root or path doesn't need privileges
/// - Otherwise, requests privilege escalation with a friendly explanation
pub fn execute_with_privilege_if_needed(
    command: &str,
    args: &[String],
    reason: &str,
) -> Result<()> {
    // Check if any argument contains a privileged path
    let needs_priv = args.iter().any(|arg| needs_privilege(arg))
        || needs_privilege(command);

    if needs_priv && !is_root() {
        // Need to escalate
        let request = PrivilegeRequest::new(reason, command)
            .with_args(args.to_vec());
        request_privilege(request)
    } else {
        // Run directly
        let status = Command::new(command)
            .args(args)
            .status()
            .context("Failed to execute command")?;

        if !status.success() {
            bail!("Command failed with exit code: {}", status.code().unwrap_or(-1));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_privilege() {
        assert!(needs_privilege("/etc/anna/config"));
        assert!(needs_privilege("/usr/local/bin/annad"));
        assert!(needs_privilege("/var/log/anna/install.log"));
        assert!(!needs_privilege("/home/user/file.txt"));
        assert!(!needs_privilege("/tmp/test"));
    }

    #[test]
    fn test_is_root() {
        // This will be false in normal test runs
        // Just verify it doesn't panic
        let _ = is_root();
    }

    #[test]
    fn test_privilege_request_creation() {
        let req = PrivilegeRequest::new(
            "install system files",
            "cp"
        ).with_args(vec!["source".to_string(), "/etc/dest".to_string()]);

        assert_eq!(req.reason, "install system files");
        assert_eq!(req.command, "cp");
        assert_eq!(req.args.len(), 2);
    }
}
