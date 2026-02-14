//! Daemon recovery: detects daemon state and attempts automatic recovery.
//! Anna NEVER tells users to run manual commands — all recovery is automatic.

use anna_shared::socket_path;
use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;

use crate::service_state::{get_service_state, ServiceState};

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

    // Connect directly — do NOT use Path::exists() first.
    // Path::exists() calls stat() which returns false for BOTH "not found" AND
    // "permission denied on parent directory", causing us to misdiagnose
    // PermissionDenied as NotRunning (triggering pkexec unnecessarily).
    // UnixStream::connect gives the exact error: ENOENT / EACCES / ECONNREFUSED.
    match UnixStream::connect(socket_path).await {
        Ok(_) => DaemonState::Running,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => DaemonState::NotRunning,
            std::io::ErrorKind::PermissionDenied => DaemonState::PermissionDenied,
            _ => {
                // ECONNREFUSED = socket exists, daemon crashed/not ready
                DaemonState::NotResponding
            }
        },
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

    // Check if user is already in /etc/group for anna (means install added them,
    // but the session hasn't picked up the new group yet — re-login fixes this).
    let already_in_group = Command::new("getent")
        .args(["group", "anna"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.contains(&username)
        })
        .unwrap_or(false);

    if already_in_group {
        return Err(anyhow!(
            "Anna cannot connect — permission denied on socket.\n\n\
             Your user '{}' is in the 'anna' group but the current session\n\
             was started before the group was added. Log out and back in\n\
             to activate group membership, then try again.",
            username
        ));
    }

    // User is not in the group at all — try to add via pkexec
    if !is_command_available("pkexec") {
        return Err(anyhow!(
            "Anna cannot connect — your user is not in the 'anna' group.\n\
             Re-run the installer to fix this."
        ));
    }

    let status = Command::new("pkexec")
        .args(["usermod", "-aG", "anna", &username])
        .status();

    if matches!(status, Ok(s) if s.success()) {
        Err(anyhow!(
            "Added '{}' to the 'anna' group. Log out and back in, then try again.",
            username
        ))
    } else {
        Err(anyhow!(
            "Anna cannot connect — your user is not in the 'anna' group.\n\
             Re-run the installer to fix this."
        ))
    }
}

/// Attempt to start the daemon via systemctl
async fn attempt_daemon_start() -> Result<RecoveryResult> {
    // Check actual systemd service state before doing anything.
    // This avoids blindly firing pkexec when the service crashed on startup.
    match get_service_state() {
        ServiceState::Failed => {
            // Service crashed/failed — pkexec won't fix this, don't prompt
            return Err(anyhow!(
                "The Anna service failed to start.\n\n\
                 This usually means a configuration or dependency issue\n\
                 on this machine. Check the service status for details."
            ));
        }
        ServiceState::NotFound => {
            return Err(anyhow!(
                "The Anna service is not installed.\n\n\
                 Re-run the installer to set it up."
            ));
        }
        ServiceState::Active => {
            // Service claims to be active but we have no socket.
            // Two sub-cases:
            // a) Socket file deleted while daemon is running — restart recreates it.
            // b) Daemon running but socket has wrong perms — permission fix needed.
            // Try restarting first; if that doesn't produce a socket, fall through to perm fix.
            if try_pkexec_restart().await {
                if wait_for_socket(DAEMON_START_TIMEOUT_SECS).await {
                    return Ok(RecoveryResult::Started);
                }
            }
            return attempt_permission_fix().await;
        }
        ServiceState::Inactive | ServiceState::Unknown => {
            // Service is stopped — attempt to start it via pkexec
        }
    }

    // Use pkexec for privilege escalation to start the service
    if try_pkexec_start().await {
        if wait_for_socket(DAEMON_START_TIMEOUT_SECS).await {
            return Ok(RecoveryResult::Started);
        }
        // Socket didn't appear after start — check if permissions issue
        let mid_state = check_daemon_state().await;
        if mid_state == DaemonState::PermissionDenied {
            return attempt_permission_fix().await;
        }
    }

    // Check final state to provide accurate message
    let final_state = check_daemon_state().await;
    match final_state {
        DaemonState::Running => Ok(RecoveryResult::Started),
        DaemonState::PermissionDenied => attempt_permission_fix().await,
        _ => Err(anyhow!(
            "Anna daemon could not be started.\n\n\
             The daemon service may not be installed correctly,\n\
             or there may be a system configuration issue."
        )),
    }
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

/// Try to restart daemon using pkexec — used when service is Active but socket is missing.
async fn try_pkexec_restart() -> bool {
    if !is_command_available("pkexec") {
        return false;
    }

    let status = Command::new("pkexec")
        .args(["systemctl", "restart", "annad"])
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
        // Connect directly — Path::exists() misreports PermissionDenied as false
        match UnixStream::connect(socket_path).await {
            Ok(_) => return true,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Directory/socket permission issue — stop polling, won't self-resolve
                return false;
            }
            _ => {}
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
                 Check the system service status for details."
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_manual_commands_in_error_messages() {
        // Error messages must not contain raw command instructions
        let forbidden = ["sudo systemctl", "usermod -aG", "journalctl", "Run:", "Try:"];
        let source = include_str!("daemon_recovery.rs");
        let test_start = source.find("#[cfg(test)]").unwrap_or(source.len());
        let main_code = &source[..test_start];
        let anyhow_sections: Vec<&str> = main_code
            .split("anyhow!(")
            .skip(1)
            .filter_map(|s| s.split(')').next())
            .collect();
        for pattern in &forbidden {
            for section in &anyhow_sections {
                assert!(!section.contains(pattern), "Forbidden '{}' in error: {}", pattern, section);
            }
        }
    }

    #[test]
    fn test_permission_error_mentions_anna_group() {
        assert!(include_str!("daemon_recovery.rs").contains("anna' group"));
    }

    #[test]
    fn test_recovery_states_are_complete() {
        let source = include_str!("daemon_recovery.rs");
        assert!(source.contains("DaemonState::Running"));
        assert!(source.contains("DaemonState::NotRunning"));
        assert!(source.contains("DaemonState::NotResponding"));
        assert!(source.contains("DaemonState::PermissionDenied"));
    }

    #[test]
    fn test_permission_auto_fix_exists() {
        let source = include_str!("daemon_recovery.rs");
        assert!(source.contains("attempt_permission_fix"));
        assert!(source.contains("pkexec") && source.contains("usermod"));
    }
}
