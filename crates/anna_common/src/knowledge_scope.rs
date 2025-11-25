//! Knowledge Scope - System vs User scoped data
//!
//! v6.54.0: Identity, Persistence, and Multi-User Awareness
//!
//! This module defines how knowledge is scoped (system vs user) and provides
//! utilities for determining scope based on paths and operations.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Knowledge scope type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeScope {
    /// System-scoped (tied to machine, shared across users)
    System,

    /// User-scoped (tied to specific user profile)
    User,

    /// Both system and user relevant
    SystemAndUser,
}

impl Default for KnowledgeScope {
    fn default() -> Self {
        KnowledgeScope::System
    }
}

/// Determine scope from a file path
pub fn scope_from_path(path: &str) -> KnowledgeScope {
    let path_obj = Path::new(path);

    // Check if path is under /home or contains a home directory
    if path.starts_with("/home/") || path.contains("/home/") {
        return KnowledgeScope::User;
    }

    // Check for user-specific paths
    if path.starts_with("~/") || path.contains("$HOME") {
        return KnowledgeScope::User;
    }

    // Common user config directories
    if path.contains("/.config/")
        || path.contains("/.local/")
        || path.contains("/.cache/")
        || path.ends_with(".vimrc")
        || path.ends_with(".bashrc")
        || path.ends_with(".zshrc") {
        return KnowledgeScope::User;
    }

    // Everything else is system-scoped
    KnowledgeScope::System
}

/// Determine scope from operation type
pub fn scope_from_operation(operation: &str, paths: &[String]) -> KnowledgeScope {
    // If any path is user-scoped, the operation is user-scoped
    for path in paths {
        if scope_from_path(path) == KnowledgeScope::User {
            return KnowledgeScope::User;
        }
    }

    // Check operation type
    let op_lower = operation.to_lowercase();

    // Telemetry is always system-scoped
    if op_lower.contains("telemetry")
        || op_lower.contains("hardware")
        || op_lower.contains("system") {
        return KnowledgeScope::System;
    }

    // Preferences are user-scoped
    if op_lower.contains("preference")
        || op_lower.contains("personality")
        || op_lower.contains("greeting") {
        return KnowledgeScope::User;
    }

    // Default to system
    KnowledgeScope::System
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_from_path_system() {
        assert_eq!(scope_from_path("/etc/ssh/sshd_config"), KnowledgeScope::System);
        assert_eq!(scope_from_path("/usr/bin/vim"), KnowledgeScope::System);
        assert_eq!(scope_from_path("/var/log/syslog"), KnowledgeScope::System);
        assert_eq!(scope_from_path("/boot/grub/grub.cfg"), KnowledgeScope::System);
    }

    #[test]
    fn test_scope_from_path_user() {
        assert_eq!(scope_from_path("/home/user/.vimrc"), KnowledgeScope::User);
        assert_eq!(scope_from_path("/home/user/.config/nvim/init.vim"), KnowledgeScope::User);
        assert_eq!(scope_from_path("~/.bashrc"), KnowledgeScope::User);
        assert_eq!(scope_from_path("/home/alice/.local/share/data"), KnowledgeScope::User);
    }

    #[test]
    fn test_scope_from_operation() {
        assert_eq!(
            scope_from_operation("telemetry", &[]),
            KnowledgeScope::System
        );
        assert_eq!(
            scope_from_operation("hardware detection", &[]),
            KnowledgeScope::System
        );
        assert_eq!(
            scope_from_operation("user preference", &[]),
            KnowledgeScope::User
        );
        assert_eq!(
            scope_from_operation("edit file", &["/home/user/.vimrc".to_string()]),
            KnowledgeScope::User
        );
        assert_eq!(
            scope_from_operation("edit file", &["/etc/ssh/sshd_config".to_string()]),
            KnowledgeScope::System
        );
    }
}
