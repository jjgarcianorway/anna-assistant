//! Privilege Detection for Anna v0.0.80
//!
//! Detects whether privileged execution is possible:
//! - Running as root
//! - sudo available non-interactively
//!
//! Never hangs waiting for password prompts.

use std::process::Command;

/// Result of privilege check
#[derive(Debug, Clone)]
pub struct PrivilegeStatus {
    /// Whether privileged execution is available
    pub available: bool,
    /// How privilege is available (root, sudo, none)
    pub method: PrivilegeMethod,
    /// Message explaining the status
    pub message: String,
}

/// How privilege is obtained
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeMethod {
    /// Running as root (UID 0)
    Root,
    /// sudo available without password
    SudoNoPassword,
    /// No privilege available
    None,
}

impl std::fmt::Display for PrivilegeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrivilegeMethod::Root => write!(f, "root"),
            PrivilegeMethod::SudoNoPassword => write!(f, "sudo"),
            PrivilegeMethod::None => write!(f, "none"),
        }
    }
}

/// Check if we're running as root
pub fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

/// Check if sudo is available without password prompt
/// Uses `sudo -n true` which fails immediately if password required
pub fn has_passwordless_sudo() -> bool {
    Command::new("sudo")
        .args(["-n", "true"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Check current privilege status
pub fn check_privilege() -> PrivilegeStatus {
    if is_root() {
        PrivilegeStatus {
            available: true,
            method: PrivilegeMethod::Root,
            message: "Running as root".to_string(),
        }
    } else if has_passwordless_sudo() {
        PrivilegeStatus {
            available: true,
            method: PrivilegeMethod::SudoNoPassword,
            message: "sudo available without password".to_string(),
        }
    } else {
        PrivilegeStatus {
            available: false,
            method: PrivilegeMethod::None,
            message: "Privilege required but not available. Run as root or configure passwordless sudo.".to_string(),
        }
    }
}

/// Run a command with appropriate privilege
/// Returns Ok(output) if successful, Err(message) if not
pub fn run_privileged(command: &str, args: &[&str]) -> Result<std::process::Output, String> {
    let status = check_privilege();

    if !status.available {
        return Err(status.message);
    }

    let output = match status.method {
        PrivilegeMethod::Root => Command::new(command).args(args).output(),
        PrivilegeMethod::SudoNoPassword => Command::new("sudo")
            .arg("-n")
            .arg(command)
            .args(args)
            .output(),
        PrivilegeMethod::None => {
            return Err("No privilege available".to_string());
        }
    };

    output.map_err(|e| format!("Failed to execute command: {}", e))
}

/// Generate manual commands for user to run themselves
pub fn generate_manual_commands(
    command: &str,
    args: &[&str],
) -> Vec<String> {
    let full_cmd = format!("sudo {} {}", command, args.join(" "));
    vec![full_cmd]
}

/// Format privilege blocked message with manual commands
pub fn format_privilege_blocked(commands: &[String]) -> String {
    let mut lines = vec![
        "╔════════════════════════════════════════════════════════════════╗".to_string(),
        "║  PRIVILEGE REQUIRED                                            ║".to_string(),
        "╠════════════════════════════════════════════════════════════════╣".to_string(),
        "║  Anna cannot execute this mutation without root privilege.     ║".to_string(),
        "║  You can run the following commands manually:                  ║".to_string(),
        "╚════════════════════════════════════════════════════════════════╝".to_string(),
        "".to_string(),
    ];

    for cmd in commands {
        lines.push(format!("  $ {}", cmd));
    }

    lines.push("".to_string());
    lines.push("To enable Anna to execute mutations, either:".to_string());
    lines.push("  1. Run annad as root (systemd service already does this)".to_string());
    lines.push("  2. Configure passwordless sudo for specific commands".to_string());

    lines.join("\n")
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_root() {
        // This test just verifies the function runs - result depends on who runs it
        let _ = is_root();
    }

    #[test]
    fn test_check_privilege() {
        // This test just verifies the function runs without hanging
        let status = check_privilege();
        // Status should be one of the valid methods
        assert!(matches!(
            status.method,
            PrivilegeMethod::Root | PrivilegeMethod::SudoNoPassword | PrivilegeMethod::None
        ));
    }

    #[test]
    fn test_generate_manual_commands() {
        let cmds = generate_manual_commands("systemctl", &["restart", "NetworkManager"]);
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("systemctl"));
        assert!(cmds[0].contains("restart"));
        assert!(cmds[0].contains("NetworkManager"));
    }

    #[test]
    fn test_format_privilege_blocked() {
        let cmds = vec!["sudo systemctl restart NetworkManager".to_string()];
        let msg = format_privilege_blocked(&cmds);
        assert!(msg.contains("PRIVILEGE REQUIRED"));
        assert!(msg.contains("systemctl restart NetworkManager"));
    }
}
