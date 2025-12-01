//! Error Type Classification

use serde::{Deserialize, Serialize};

/// Classification of error types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Runtime crash or unexpected exit
    Crash,
    /// Permission denied
    Permission,
    /// Missing file or directory
    MissingFile,
    /// Dependency failure
    Dependency,
    /// Configuration error
    Configuration,
    /// Network error
    Network,
    /// Resource exhaustion (OOM, disk full)
    Resource,
    /// Timeout
    Timeout,
    /// Segfault or memory corruption
    Segfault,
    /// Service failure
    ServiceFailure,
    /// Intrusion attempt detected
    Intrusion,
    /// Generic error (when no specific type matches)
    Other,
}

impl ErrorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorType::Crash => "crash",
            ErrorType::Permission => "permission",
            ErrorType::MissingFile => "missing_file",
            ErrorType::Dependency => "dependency",
            ErrorType::Configuration => "configuration",
            ErrorType::Network => "network",
            ErrorType::Resource => "resource",
            ErrorType::Timeout => "timeout",
            ErrorType::Segfault => "segfault",
            ErrorType::ServiceFailure => "service_failure",
            ErrorType::Intrusion => "intrusion",
            ErrorType::Other => "other",
        }
    }

    /// Detect error type from log message
    pub fn detect_from_message(msg: &str) -> Self {
        let lower = msg.to_lowercase();

        // Permission errors
        if lower.contains("permission denied")
            || lower.contains("access denied")
            || lower.contains("operation not permitted")
            || lower.contains("eacces")
            || lower.contains("eperm")
        {
            return ErrorType::Permission;
        }

        // Missing file errors
        if lower.contains("no such file")
            || lower.contains("file not found")
            || lower.contains("enoent")
            || lower.contains("not found:")
            || lower.contains("missing:")
        {
            return ErrorType::MissingFile;
        }

        // Segfault/memory errors
        if lower.contains("segmentation fault")
            || lower.contains("segfault")
            || lower.contains("sigsegv")
            || lower.contains("core dumped")
            || lower.contains("memory corruption")
            || lower.contains("double free")
            || lower.contains("heap corruption")
        {
            return ErrorType::Segfault;
        }

        // Crash/exit errors
        if lower.contains("crashed")
            || lower.contains("fatal")
            || lower.contains("abort")
            || lower.contains("killed")
            || lower.contains("sigkill")
            || lower.contains("sigterm")
            || lower.contains("unexpected exit")
        {
            return ErrorType::Crash;
        }

        // Dependency errors
        if lower.contains("dependency")
            || lower.contains("missing library")
            || lower.contains("cannot find")
            || lower.contains("unable to resolve")
            || lower.contains("unmet dependency")
        {
            return ErrorType::Dependency;
        }

        // Configuration errors
        if lower.contains("config")
            || lower.contains("invalid setting")
            || lower.contains("parse error")
            || lower.contains("syntax error")
            || lower.contains("malformed")
        {
            return ErrorType::Configuration;
        }

        // Network errors
        if lower.contains("connection refused")
            || lower.contains("network unreachable")
            || lower.contains("host unreachable")
            || lower.contains("dns")
            || lower.contains("socket")
            || lower.contains("econnrefused")
        {
            return ErrorType::Network;
        }

        // Resource errors
        if lower.contains("out of memory")
            || lower.contains("oom")
            || lower.contains("disk full")
            || lower.contains("no space left")
            || lower.contains("resource exhausted")
            || lower.contains("too many open files")
        {
            return ErrorType::Resource;
        }

        // Timeout errors
        if lower.contains("timeout") || lower.contains("timed out") {
            return ErrorType::Timeout;
        }

        // Service errors
        if lower.contains("service failed")
            || lower.contains("unit failed")
            || lower.contains("failed to start")
            || lower.contains("activation failed")
        {
            return ErrorType::ServiceFailure;
        }

        // Intrusion/auth errors
        if lower.contains("authentication failure")
            || lower.contains("failed password")
            || lower.contains("invalid user")
            || lower.contains("failed login")
            || lower.contains("not in sudoers")
            || lower.contains("pam:")
        {
            return ErrorType::Intrusion;
        }

        ErrorType::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_detection() {
        assert_eq!(
            ErrorType::detect_from_message("Permission denied"),
            ErrorType::Permission
        );
        assert_eq!(
            ErrorType::detect_from_message("No such file or directory"),
            ErrorType::MissingFile
        );
        assert_eq!(
            ErrorType::detect_from_message("Segmentation fault (core dumped)"),
            ErrorType::Segfault
        );
        assert_eq!(
            ErrorType::detect_from_message("Connection refused"),
            ErrorType::Network
        );
        assert_eq!(
            ErrorType::detect_from_message("Out of memory"),
            ErrorType::Resource
        );
    }
}
