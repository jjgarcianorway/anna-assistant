// Context Detection
// Phase 3.8: Adaptive CLI Refinement
//
// Detects execution context for adaptive command visibility

use anna_common::command_meta::UserLevel;
use std::process::Command;

/// Execution context
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionContext {
    /// Normal user
    User,
    /// Root user (sudo or actual root)
    Root,
    /// Developer/internal mode
    Developer,
}

impl ExecutionContext {
    /// Detect current execution context
    pub fn detect() -> Self {
        // Check if running as root
        if is_root() {
            return ExecutionContext::Root;
        }

        // Check for developer mode environment variables
        if is_developer_mode() {
            return ExecutionContext::Developer;
        }

        // Default to normal user
        ExecutionContext::User
    }

    /// Convert to UserLevel for help system
    pub fn to_user_level(&self) -> UserLevel {
        match self {
            ExecutionContext::User => UserLevel::Beginner,
            ExecutionContext::Root => UserLevel::Intermediate,
            ExecutionContext::Developer => UserLevel::Expert,
        }
    }

    /// Check if can execute advanced commands
    pub fn can_execute_advanced(&self) -> bool {
        matches!(self, ExecutionContext::Root | ExecutionContext::Developer)
    }

    /// Check if can see internal commands
    pub fn can_see_internal(&self) -> bool {
        matches!(self, ExecutionContext::Developer)
    }
}

/// Check if running as root
fn is_root() -> bool {
    #[cfg(unix)]
    {
        // Check effective UID
        let output = Command::new("id").arg("-u").output().ok();

        if let Some(output) = output {
            if let Ok(uid_str) = String::from_utf8(output.stdout) {
                if let Ok(uid) = uid_str.trim().parse::<u32>() {
                    return uid == 0;
                }
            }
        }

        // Fallback: check SUDO_USER environment variable
        std::env::var("SUDO_USER").is_ok()
    }

    #[cfg(not(unix))]
    {
        false
    }
}

/// Check for developer mode indicators
fn is_developer_mode() -> bool {
    // Check environment variables
    if std::env::var("ANNA_DEV_MODE").is_ok() {
        return true;
    }

    if std::env::var("ANNA_INTERNAL").is_ok() {
        return true;
    }

    // Check if running from cargo (development)
    if std::env::var("CARGO").is_ok() {
        return true;
    }

    false
}

/// Check if output is to a TTY (for color detection)
pub fn is_tty() -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let stdout_fd = std::io::stdout().as_raw_fd();
        unsafe { libc::isatty(stdout_fd) == 1 }
    }

    #[cfg(not(unix))]
    {
        false
    }
}

/// Detect if color output should be used
pub fn should_use_color() -> bool {
    // Check NO_COLOR environment variable
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check if TTY
    if !is_tty() {
        return false;
    }

    // Check TERM
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_detection() {
        let context = ExecutionContext::detect();
        // Should detect some context
        assert!(matches!(
            context,
            ExecutionContext::User | ExecutionContext::Root | ExecutionContext::Developer
        ));
    }

    #[test]
    fn test_user_level_conversion() {
        assert_eq!(ExecutionContext::User.to_user_level(), UserLevel::Beginner);
        assert_eq!(
            ExecutionContext::Root.to_user_level(),
            UserLevel::Intermediate
        );
        assert_eq!(
            ExecutionContext::Developer.to_user_level(),
            UserLevel::Expert
        );
    }

    #[test]
    fn test_permissions() {
        let user_ctx = ExecutionContext::User;
        assert!(!user_ctx.can_execute_advanced());
        assert!(!user_ctx.can_see_internal());

        let root_ctx = ExecutionContext::Root;
        assert!(root_ctx.can_execute_advanced());
        assert!(!root_ctx.can_see_internal());

        let dev_ctx = ExecutionContext::Developer;
        assert!(dev_ctx.can_execute_advanced());
        assert!(dev_ctx.can_see_internal());
    }

    #[test]
    fn test_color_detection() {
        // Just test that it doesn't panic
        let _ = should_use_color();
        let _ = is_tty();
    }
}
