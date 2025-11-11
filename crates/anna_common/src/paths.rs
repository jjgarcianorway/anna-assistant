//! Path helpers for Anna Assistant
//!
//! Phase rc.13.2: Dual-mode socket support (system and user)
//! Citation: [archwiki:XDG_Base_Directory]

use std::path::PathBuf;

/// Get the system socket path (requires systemd and anna group)
pub fn system_socket_path() -> PathBuf {
    PathBuf::from("/run/anna/anna.sock")
}

/// Get the user socket path (no root required)
///
/// Priority:
/// 1. $XDG_RUNTIME_DIR/anna/anna.sock (systemd user sessions)
/// 2. /tmp/anna-$UID/anna.sock (fallback)
pub fn user_socket_path() -> PathBuf {
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg_runtime).join("anna/anna.sock")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/anna-{}/anna.sock", uid))
    }
}

/// Get the user socket directory
pub fn user_socket_dir() -> PathBuf {
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg_runtime).join("anna")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/anna-{}", uid))
    }
}

/// Get the system socket directory
pub fn system_socket_dir() -> PathBuf {
    PathBuf::from("/run/anna")
}

/// Get the user PID file path
pub fn user_pid_file() -> PathBuf {
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(xdg_runtime).join("anna/annad.pid")
    } else {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/anna-{}/annad.pid", uid))
    }
}

/// Detect if we're running in user mode
///
/// Heuristics:
/// - Check if we have write access to /run/anna
/// - Check if $XDG_RUNTIME_DIR is set (user session indicator)
/// - Check effective UID (root = system mode likely)
pub fn should_use_user_mode() -> bool {
    // If we're root, prefer system mode
    if unsafe { libc::geteuid() } == 0 {
        return false;
    }

    // If /run/anna exists and is writable, use system mode
    if std::fs::metadata("/run/anna").is_ok()
        && std::fs::OpenOptions::new()
            .write(true)
            .open("/run/anna")
            .is_ok()
        {
            return false;
        }

    // Otherwise, use user mode
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_socket_path() {
        assert_eq!(system_socket_path(), PathBuf::from("/run/anna/anna.sock"));
    }

    #[test]
    fn test_user_socket_path_with_xdg() {
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        assert_eq!(
            user_socket_path(),
            PathBuf::from("/run/user/1000/anna/anna.sock")
        );
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_user_socket_path_fallback() {
        std::env::remove_var("XDG_RUNTIME_DIR");
        let path = user_socket_path();
        assert!(path.to_string_lossy().starts_with("/tmp/anna-"));
        assert!(path.to_string_lossy().ends_with("/anna.sock"));
    }
}
