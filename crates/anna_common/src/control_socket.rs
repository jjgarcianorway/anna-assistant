//! Control Socket v7.42.0 - Authoritative Daemon Health Check
//!
//! Provides a Unix domain socket for fast, reliable daemon health checks.
//! This is the source of truth for "daemon running" - NOT snapshot presence.
//!
//! Socket: /run/anna/annad.sock
//! Protocol:
//!   - PING -> PONG (simple health check)
//!   - STATUS -> JSON { version, uptime_s, pid, snapshot_seq }
//!
//! annactl uses this socket first (150ms timeout), then falls back to systemd.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;

/// Socket directory (runtime directory managed by systemd)
pub const SOCKET_DIR: &str = "/run/anna";

/// Socket path
pub const SOCKET_PATH: &str = "/run/anna/annad.sock";

/// Timeout for client connections
pub const CLIENT_TIMEOUT_MS: u64 = 150;

/// Status response from daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub pid: u32,
    pub snapshot_seq: u64,
    pub boot_id: Option<String>,
}

/// Result of a daemon health check
#[derive(Debug, Clone)]
pub enum DaemonHealth {
    /// Daemon is running (confirmed via socket)
    Running(DaemonStatus),
    /// Daemon is running (confirmed via systemd, but socket unavailable)
    RunningSystemd,
    /// Daemon is not running
    Stopped,
    /// Cannot determine (neither socket nor systemd worked)
    Unknown(String),
}

impl DaemonHealth {
    pub fn is_running(&self) -> bool {
        matches!(
            self,
            DaemonHealth::Running(_) | DaemonHealth::RunningSystemd
        )
    }
}

/// Check if daemon is running (for annactl)
/// 1. Try control socket (fast, authoritative)
/// 2. Fall back to systemctl is-active (slower, but reliable)
pub fn check_daemon_health() -> DaemonHealth {
    // Try socket first
    if let Ok(status) = ping_socket() {
        return DaemonHealth::Running(status);
    }

    // Fall back to systemctl
    match check_systemd_active() {
        Ok(true) => DaemonHealth::RunningSystemd,
        Ok(false) => DaemonHealth::Stopped,
        Err(e) => DaemonHealth::Unknown(e),
    }
}

/// Ping the control socket and get status
pub fn ping_socket() -> Result<DaemonStatus, String> {
    let path = Path::new(SOCKET_PATH);
    if !path.exists() {
        return Err("socket does not exist".to_string());
    }

    let mut stream = UnixStream::connect(path).map_err(|e| format!("connect failed: {}", e))?;

    // Set timeout
    let timeout = Duration::from_millis(CLIENT_TIMEOUT_MS);
    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();

    // Send STATUS request
    stream
        .write_all(b"STATUS\n")
        .map_err(|e| format!("write failed: {}", e))?;

    // Read response
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("read failed: {}", e))?;

    // Parse JSON response
    serde_json::from_str(&response).map_err(|e| format!("parse failed: {}", e))
}

/// Check if socket exists and is connectable (simple PING)
pub fn socket_connectable() -> bool {
    let path = Path::new(SOCKET_PATH);
    if !path.exists() {
        return false;
    }

    match UnixStream::connect(path) {
        Ok(mut stream) => {
            let timeout = Duration::from_millis(CLIENT_TIMEOUT_MS);
            stream.set_read_timeout(Some(timeout)).ok();
            stream.set_write_timeout(Some(timeout)).ok();

            if stream.write_all(b"PING\n").is_err() {
                return false;
            }

            let mut response = [0u8; 16];
            if let Ok(n) = stream.read(&mut response) {
                let resp = String::from_utf8_lossy(&response[..n]);
                return resp.trim() == "PONG";
            }
            false
        }
        Err(_) => false,
    }
}

/// Check systemd active state
pub fn check_systemd_active() -> Result<bool, String> {
    let output = std::process::Command::new("systemctl")
        .args(["is-active", "annad"])
        .output()
        .map_err(|e| format!("systemctl failed: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "active")
}

/// Get systemd main PID if available
pub fn get_systemd_pid() -> Option<u32> {
    let output = std::process::Command::new("systemctl")
        .args(["show", "annad", "--property=MainPID", "--value"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse().ok().filter(|&pid| pid > 0)
}

// =============================================================================
// SERVER SIDE (for annad)
// =============================================================================

/// Control socket server (used by daemon)
pub struct ControlSocketServer {
    listener: UnixListener,
}

impl ControlSocketServer {
    /// Create and bind the control socket
    pub fn bind() -> std::io::Result<Self> {
        // Create socket directory
        std::fs::create_dir_all(SOCKET_DIR)?;

        // Remove stale socket if exists
        let path = Path::new(SOCKET_PATH);
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        // Bind socket
        let listener = UnixListener::bind(path)?;

        // Set non-blocking for accept
        listener.set_nonblocking(true)?;

        // Set permissions on socket
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o666);
            std::fs::set_permissions(path, perms)?;
        }

        Ok(ControlSocketServer { listener })
    }

    /// Accept and handle a single connection (non-blocking)
    /// Returns true if a connection was handled
    pub fn try_accept(&self, status: &DaemonStatus) -> bool {
        match self.listener.accept() {
            Ok((mut stream, _)) => {
                // Set timeout for this connection
                let timeout = Duration::from_millis(100);
                stream.set_read_timeout(Some(timeout)).ok();
                stream.set_write_timeout(Some(timeout)).ok();

                // Read request
                let mut buf = [0u8; 64];
                if let Ok(n) = stream.read(&mut buf) {
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let request = request.trim();

                    match request {
                        "PING" => {
                            let _ = stream.write_all(b"PONG\n");
                        }
                        "STATUS" => {
                            if let Ok(json) = serde_json::to_string(status) {
                                let _ = stream.write_all(json.as_bytes());
                            }
                        }
                        _ => {
                            let _ = stream.write_all(b"UNKNOWN\n");
                        }
                    }
                }
                true
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => false,
            Err(_) => false,
        }
    }

    /// Cleanup on drop
    pub fn cleanup(&self) {
        let _ = std::fs::remove_file(SOCKET_PATH);
    }
}

impl Drop for ControlSocketServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_status_serialize() {
        let status = DaemonStatus {
            version: "7.42.0".to_string(),
            uptime_secs: 3600,
            pid: 12345,
            snapshot_seq: 42,
            boot_id: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("7.42.0"));
        assert!(json.contains("12345"));
    }
}
