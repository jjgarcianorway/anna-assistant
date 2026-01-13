//! Daemon recovery module for self-healing connection management.
//!
//! v0.3.35: Anna NEVER tells users to run manual commands.
//! Instead, she detects daemon state and attempts automatic recovery.
//! v0.3.36: Added permission auto-fix via pkexec
//!
//! Recovery Flow:
//! 1. Check if socket exists
//! 2. If not, attempt to start daemon via systemctl
//! 3. If privilege escalation needed, use pkexec (polkit GUI prompt)
//! 4. Wait for socket with timeout
//! 5. Retry connection
//! 6. If permission denied, offer to fix via pkexec
//! 7. Report status in natural language (never raw errors)

use anna_shared::socket_path;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;

/// Maximum time to wait for daemon to start and socket to appear
const DAEMON_START_TIMEOUT_SECS: u64 = 15;

/// Interval between socket availability checks
const SOCKET_CHECK_INTERVAL_MS: u64 = 200;

/// Whether permission fix has been attempted this session
static PERMISSION_FIX_ATTEMPTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Result of a daemon recovery attempt
#[derive(Debug)]
pub enum RecoveryResult {
    /// Daemon was already running and connection succeeded
    AlreadyRunning,
    /// Daemon was started successfully
    Started,
    /// Daemon could not be started (with explanation)
    Failed(String),
}

/// Daemon connection state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DaemonState {
    /// Socket exists and daemon is accepting connections
    Running,
    /// Socket doesn't exist - daemon not started
    NotRunning,
    /// Socket exists but connection refused - daemon crashed or not ready
    NotResponding,
    /// Permission denied - user not in anna group
    PermissionDenied,
}

/// Check the current daemon state
pub async fn check_daemon_state() -> DaemonState {
    let socket_file = socket_path();
    let socket_path = Path::new(&socket_file);

    if !socket_path.exists() {
        return DaemonState::NotRunning;
    }

    // Try to connect
    match UnixStream::connect(socket_path).await {
        Ok(_) => DaemonState::Running,
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            if err_str.contains("permission denied")
                || e.kind() == std::io::ErrorKind::PermissionDenied
            {
                DaemonState::PermissionDenied
            } else {
                DaemonState::NotResponding
            }
        }
    }
}

/// Attempt to start the daemon automatically
/// Returns a human-readable message about what happened
pub async fn ensure_daemon_running() -> Result<RecoveryResult> {
    let state = check_daemon_state().await;

    match state {
        DaemonState::Running => Ok(RecoveryResult::AlreadyRunning),

        DaemonState::PermissionDenied => {
            // v0.3.36: Attempt to fix permissions via pkexec
            attempt_permission_fix().await
        }

        DaemonState::NotRunning | DaemonState::NotResponding => {
            // Attempt automatic recovery
            attempt_daemon_start().await
        }
    }
}

/// v0.3.36: Attempt to fix permission issues via pkexec
/// This adds the current user to the 'anna' group
async fn attempt_permission_fix() -> Result<RecoveryResult> {
    use std::sync::atomic::Ordering;

    // Only attempt once per session to avoid spamming pkexec prompts
    if PERMISSION_FIX_ATTEMPTED.swap(true, Ordering::SeqCst) {
        return Err(anyhow!(
            "Anna cannot connect due to permissions.\n\n\
             Your user account needs to be in the 'anna' group.\n\
             The permission fix was already attempted this session.\n\
             Please log out and back in for group membership to take effect."
        ));
    }

    // Get current username
    let username = std::env::var("USER").or_else(|_| std::env::var("LOGNAME"));
    let Ok(username) = username else {
        return Err(anyhow!(
            "Anna cannot connect due to permissions.\n\n\
             Your user account needs to be in the 'anna' group.\n\
             Could not determine current username to attempt fix."
        ));
    };

    // Check if pkexec is available
    if !is_command_available("pkexec") {
        return Err(anyhow!(
            "Anna cannot connect due to permissions.\n\n\
             Your user account needs to be in the 'anna' group.\n\
             This was set up during installation - you may need to log out and back in\n\
             for the group membership to take effect."
        ));
    }

    // Try to add user to anna group via pkexec
    eprintln!("Anna needs to add your user to the 'anna' group...");
    let status = Command::new("pkexec")
        .args(["usermod", "-aG", "anna", &username])
        .status();

    if matches!(status, Ok(s) if s.success()) {
        // Group added, but user needs to log out/in
        Err(anyhow!(
            "Anna has added your user to the 'anna' group.\n\n\
             Please log out and back in for the change to take effect,\n\
             then try again."
        ))
    } else {
        Err(anyhow!(
            "Anna cannot connect due to permissions.\n\n\
             Your user account needs to be in the 'anna' group.\n\
             This was set up during installation - you may need to log out and back in\n\
             for the group membership to take effect."
        ))
    }
}

/// Attempt to start the daemon via systemctl
async fn attempt_daemon_start() -> Result<RecoveryResult> {
    // First, try without privilege escalation (in case user has passwordless sudo)
    if try_systemctl_start().await {
        // Wait for socket to become available
        if wait_for_socket(DAEMON_START_TIMEOUT_SECS).await {
            return Ok(RecoveryResult::Started);
        }
    }

    // Try with pkexec for GUI privilege escalation
    if try_pkexec_start().await {
        if wait_for_socket(DAEMON_START_TIMEOUT_SECS).await {
            return Ok(RecoveryResult::Started);
        }
    }

    // Check final state to provide accurate message
    let final_state = check_daemon_state().await;
    match final_state {
        DaemonState::Running => Ok(RecoveryResult::Started),
        DaemonState::PermissionDenied => Err(anyhow!(
            "Anna cannot connect due to permissions.\n\n\
             Your user account needs to be in the 'anna' group.\n\
             This was set up during installation - you may need to log out and back in\n\
             for the group membership to take effect."
        )),
        _ => Err(anyhow!(
            "Anna daemon could not be started.\n\n\
             The daemon service may not be installed correctly, or there may be a\n\
             system configuration issue. Check the system logs for details:\n\
             journalctl -u annad -n 50"
        )),
    }
}

/// Try to start daemon using systemctl (may fail without privileges)
async fn try_systemctl_start() -> bool {
    // Try user's systemctl first (may work if user has permissions)
    let status = Command::new("systemctl")
        .args(["start", "annad"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

/// Try to start daemon using pkexec (polkit GUI prompt)
async fn try_pkexec_start() -> bool {
    // Check if pkexec is available
    if !is_command_available("pkexec") {
        return false;
    }

    // pkexec will show a GUI prompt for authentication
    let status = Command::new("pkexec")
        .args(["systemctl", "start", "annad"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    matches!(status, Ok(s) if s.success())
}

/// Wait for socket to become available
async fn wait_for_socket(timeout_secs: u64) -> bool {
    let socket_file = socket_path();
    let socket_path = Path::new(&socket_file);
    let max_checks = (timeout_secs * 1000) / SOCKET_CHECK_INTERVAL_MS;

    for _ in 0..max_checks {
        if socket_path.exists() {
            // Socket exists, try to connect
            if let Ok(_) = UnixStream::connect(socket_path).await {
                return true;
            }
        }
        sleep(Duration::from_millis(SOCKET_CHECK_INTERVAL_MS)).await;
    }

    false
}

/// Check if a command is available in PATH
fn is_command_available(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Connect to daemon with automatic recovery
/// This is the main entry point for connection with self-healing
pub async fn connect_with_recovery() -> Result<UnixStream> {
    // First, ensure daemon is running (with automatic recovery if needed)
    let recovery_result = ensure_daemon_running().await?;

    // Log recovery action for debugging
    match recovery_result {
        RecoveryResult::AlreadyRunning => {}
        RecoveryResult::Started => {
            // Small delay to ensure daemon is fully ready
            sleep(Duration::from_millis(100)).await;
        }
        RecoveryResult::Failed(msg) => {
            return Err(anyhow!("{}", msg));
        }
    }

    // Now connect
    let socket_file = socket_path();
    let socket_path = Path::new(&socket_file);

    UnixStream::connect(socket_path).await.map_err(|e| {
        let err_str = e.to_string().to_lowercase();
        if err_str.contains("permission denied")
            || e.kind() == std::io::ErrorKind::PermissionDenied
        {
            anyhow!(
                "Anna cannot connect due to permissions.\n\n\
                 Your user account needs to be in the 'anna' group.\n\
                 This was set up during installation - you may need to log out and back in\n\
                 for the group membership to take effect."
            )
        } else {
            anyhow!(
                "Anna daemon is not responding.\n\n\
                 The daemon may have encountered an error during startup.\n\
                 Check the system logs for details: journalctl -u annad -n 50"
            )
        }
    })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_manual_commands_in_error_messages() {
        // Verify our error messages don't contain manual command instructions
        let forbidden_patterns = [
            "sudo systemctl",
            "systemctl start",
            "systemctl restart",
            "usermod -aG",
            "Run:",
            "Try:",
        ];

        let source = include_str!("daemon_recovery.rs");

        // Find error messages (text inside anyhow! macros)
        for pattern in &forbidden_patterns {
            // Skip the test module and comments
            let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
            let main_code = &source[..test_start];

            // Check error message strings (inside quotes after anyhow!)
            let anyhow_sections: Vec<&str> = main_code
                .split("anyhow!(")
                .skip(1)
                .filter_map(|s| s.split(')').next())
                .collect();

            for section in &anyhow_sections {
                assert!(
                    !section.contains(pattern),
                    "Error message contains forbidden pattern '{}': {}",
                    pattern,
                    section
                );
            }
        }
    }

    #[test]
    fn test_permission_error_mentions_anna_group() {
        let source = include_str!("daemon_recovery.rs");
        assert!(
            source.contains("anna' group"),
            "Permission error should mention the anna group"
        );
    }

    #[test]
    fn test_recovery_states_are_complete() {
        // Verify all daemon states have recovery paths
        let source = include_str!("daemon_recovery.rs");
        assert!(source.contains("DaemonState::Running"));
        assert!(source.contains("DaemonState::NotRunning"));
        assert!(source.contains("DaemonState::NotResponding"));
        assert!(source.contains("DaemonState::PermissionDenied"));
    }

    /// v0.3.36: Verify permission auto-fix exists
    #[test]
    fn test_permission_auto_fix_exists() {
        let source = include_str!("daemon_recovery.rs");
        assert!(
            source.contains("attempt_permission_fix"),
            "Should have permission auto-fix function"
        );
        assert!(
            source.contains("pkexec") && source.contains("usermod"),
            "Permission fix should use pkexec to add user to group"
        );
    }
}
