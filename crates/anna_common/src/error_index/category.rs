//! Log Category - v5.2.3 Universal Error Model
//!
//! Categories are inferred from log text and patterns, not hardcoded.

use super::error_type::ErrorType;
use serde::{Deserialize, Serialize};

/// Log entry category (v5.2.3 - Universal Error Model)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogCategory {
    /// Startup/initialization failures
    Startup,
    /// Runtime errors during normal operation
    Runtime,
    /// Configuration parsing or validation errors
    Config,
    /// Dependency failures (other units, libs, services)
    Dependency,
    /// Intrusion/authentication failures
    Intrusion,
    /// Filesystem issues (missing, permissions, space)
    Filesystem,
    /// Network connectivity issues
    Network,
    /// Permission/access denied
    Permission,
    /// Other/unclassified
    Other,
}

impl LogCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Startup => "startup",
            LogCategory::Runtime => "runtime",
            LogCategory::Config => "config",
            LogCategory::Dependency => "dependency",
            LogCategory::Intrusion => "intrusion",
            LogCategory::Filesystem => "filesystem",
            LogCategory::Network => "network",
            LogCategory::Permission => "permission",
            LogCategory::Other => "other",
        }
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            LogCategory::Startup => "Startup",
            LogCategory::Runtime => "Runtime",
            LogCategory::Config => "Config",
            LogCategory::Dependency => "Dependency",
            LogCategory::Intrusion => "Intrusion",
            LogCategory::Filesystem => "Filesystem",
            LogCategory::Network => "Network",
            LogCategory::Permission => "Permission",
            LogCategory::Other => "Other",
        }
    }

    /// v5.2.3: Detect category from error type
    pub fn from_error_type(error_type: &ErrorType) -> Self {
        match error_type {
            ErrorType::Intrusion => LogCategory::Intrusion,
            ErrorType::ServiceFailure => LogCategory::Startup,
            ErrorType::Crash | ErrorType::Segfault | ErrorType::Timeout => LogCategory::Runtime,
            ErrorType::Configuration => LogCategory::Config,
            ErrorType::Dependency => LogCategory::Dependency,
            ErrorType::Network => LogCategory::Network,
            ErrorType::Permission => LogCategory::Permission,
            ErrorType::MissingFile | ErrorType::Resource => LogCategory::Filesystem,
            ErrorType::Other => LogCategory::Other,
        }
    }

    /// v5.2.3: Detect category directly from message (pattern-based)
    pub fn detect_from_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();

        // Startup patterns
        if lower.contains("failed to start")
            || lower.contains("starting")
            || lower.contains("activation failed")
            || lower.contains("unit entered failed state")
            || lower.contains("start request")
        {
            return LogCategory::Startup;
        }

        // Intrusion patterns
        if lower.contains("authentication failure")
            || lower.contains("failed password")
            || lower.contains("invalid user")
            || lower.contains("not in sudoers")
            || lower.contains("pam:")
            || lower.contains("brute force")
            || lower.contains("failed login")
        {
            return LogCategory::Intrusion;
        }

        // Config patterns
        if lower.contains("config")
            || lower.contains("parse error")
            || lower.contains("syntax error")
            || lower.contains("invalid directive")
            || lower.contains("unsupported option")
            || lower.contains("deprecated")
        {
            return LogCategory::Config;
        }

        // Dependency patterns
        if lower.contains("dependency")
            || lower.contains("requires")
            || lower.contains("failed to load")
            || lower.contains("missing library")
            || lower.contains("cannot find module")
        {
            return LogCategory::Dependency;
        }

        // Filesystem patterns
        if lower.contains("no such file")
            || lower.contains("disk full")
            || lower.contains("no space left")
            || lower.contains("read-only filesystem")
            || lower.contains("i/o error")
        {
            return LogCategory::Filesystem;
        }

        // Permission patterns
        if lower.contains("permission denied")
            || lower.contains("access denied")
            || lower.contains("operation not permitted")
            || lower.contains("selinux")
            || lower.contains("apparmor")
        {
            return LogCategory::Permission;
        }

        // Network patterns
        if lower.contains("connection refused")
            || lower.contains("network unreachable")
            || lower.contains("host unreachable")
            || lower.contains("dns")
            || lower.contains("socket")
        {
            return LogCategory::Network;
        }

        // Runtime patterns (crashes, signals, etc.)
        if lower.contains("segfault")
            || lower.contains("crashed")
            || lower.contains("core dumped")
            || lower.contains("signal")
            || lower.contains("killed")
            || lower.contains("oom")
        {
            return LogCategory::Runtime;
        }

        LogCategory::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_detect_intrusion() {
        assert_eq!(
            LogCategory::detect_from_message("authentication failure for root"),
            LogCategory::Intrusion
        );
        assert_eq!(
            LogCategory::detect_from_message("Failed password for invalid user admin"),
            LogCategory::Intrusion
        );
        assert_eq!(
            LogCategory::detect_from_message("pam: authentication failed"),
            LogCategory::Intrusion
        );
    }

    #[test]
    fn test_category_detect_filesystem() {
        assert_eq!(
            LogCategory::detect_from_message("No such file or directory: /etc/foo"),
            LogCategory::Filesystem
        );
        assert_eq!(
            LogCategory::detect_from_message("disk full, no space left on device"),
            LogCategory::Filesystem
        );
    }

    #[test]
    fn test_category_detect_config() {
        assert_eq!(
            LogCategory::detect_from_message("config error: invalid directive"),
            LogCategory::Config
        );
        assert_eq!(
            LogCategory::detect_from_message("parse error in configuration file"),
            LogCategory::Config
        );
    }

    #[test]
    fn test_category_detect_startup() {
        assert_eq!(
            LogCategory::detect_from_message("Failed to start nginx.service"),
            LogCategory::Startup
        );
        assert_eq!(
            LogCategory::detect_from_message("unit entered failed state after activation"),
            LogCategory::Startup
        );
    }

    #[test]
    fn test_category_detect_runtime() {
        assert_eq!(
            LogCategory::detect_from_message("segfault at 0x0000"),
            LogCategory::Runtime
        );
        assert_eq!(
            LogCategory::detect_from_message("process crashed with signal 11"),
            LogCategory::Runtime
        );
        assert_eq!(
            LogCategory::detect_from_message("OOM killer invoked"),
            LogCategory::Runtime
        );
    }

    #[test]
    fn test_category_detect_permission() {
        assert_eq!(
            LogCategory::detect_from_message("permission denied: /var/log/secure"),
            LogCategory::Permission
        );
        assert_eq!(
            LogCategory::detect_from_message("operation not permitted"),
            LogCategory::Permission
        );
    }

    #[test]
    fn test_category_detect_network() {
        assert_eq!(
            LogCategory::detect_from_message("connection refused to 192.168.1.1:443"),
            LogCategory::Network
        );
        assert_eq!(
            LogCategory::detect_from_message("network unreachable"),
            LogCategory::Network
        );
    }

    #[test]
    fn test_category_detect_dependency() {
        assert_eq!(
            LogCategory::detect_from_message("dependency failed for postgresql.service"),
            LogCategory::Dependency
        );
        assert_eq!(
            LogCategory::detect_from_message("missing library: libssl.so.1.1"),
            LogCategory::Dependency
        );
    }
}
