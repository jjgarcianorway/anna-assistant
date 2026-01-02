//! Helper functions for error categorization and query detection

use super::ErrorCategory;

/// Check if query is asking about errors
pub fn is_error_summary_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "show errors",
        "list errors",
        "error summary",
        "what errors",
        "any errors",
        "show warnings",
        "all warnings",
        "list warnings",
        "system errors",
        "recent errors",
        "error log",
        "problems",
        "issues",
        "what's wrong",
        "whats wrong",
    ];

    keywords.iter().any(|kw| q.contains(kw))
}

/// Categorize an error message automatically
pub fn categorize_error(message: &str) -> ErrorCategory {
    let msg = message.to_lowercase();

    if msg.contains("disk")
        || msg.contains("memory")
        || msg.contains("cpu")
        || msg.contains("kernel")
    {
        return ErrorCategory::System;
    }

    if msg.contains("service")
        || msg.contains("daemon")
        || msg.contains("systemd")
        || msg.contains("unit")
    {
        return ErrorCategory::Service;
    }

    if msg.contains("network")
        || msg.contains("connection")
        || msg.contains("dns")
        || msg.contains("socket")
    {
        return ErrorCategory::Network;
    }

    if msg.contains("config")
        || msg.contains("configuration")
        || msg.contains("setting")
    {
        return ErrorCategory::Config;
    }

    if msg.contains("package")
        || msg.contains("install")
        || msg.contains("pacman")
        || msg.contains("apt")
    {
        return ErrorCategory::Package;
    }

    if msg.contains("permission")
        || msg.contains("denied")
        || msg.contains("access")
        || msg.contains("sudo")
    {
        return ErrorCategory::Permission;
    }

    if msg.contains("llm")
        || msg.contains("model")
        || msg.contains("ollama")
        || msg.contains("inference")
    {
        return ErrorCategory::Llm;
    }

    if msg.contains("recipe") || msg.contains("execute") || msg.contains("command failed") {
        return ErrorCategory::Recipe;
    }

    ErrorCategory::Other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_error_summary_query() {
        assert!(is_error_summary_query("show me any errors"));
        assert!(is_error_summary_query("list all warnings"));
        assert!(is_error_summary_query("what's wrong with the system?"));
        assert!(is_error_summary_query("are there any issues?"));
        assert!(!is_error_summary_query("how do I install vim?"));
    }

    #[test]
    fn test_categorize_error() {
        assert_eq!(
            categorize_error("Disk space running low"),
            ErrorCategory::System
        );
        assert_eq!(
            categorize_error("Failed to start docker.service"),
            ErrorCategory::Service
        );
        assert_eq!(
            categorize_error("Network connection timeout"),
            ErrorCategory::Network
        );
        assert_eq!(
            categorize_error("Invalid configuration file"),
            ErrorCategory::Config
        );
        assert_eq!(
            categorize_error("pacman package not found"),
            ErrorCategory::Package
        );
        assert_eq!(
            categorize_error("Permission denied"),
            ErrorCategory::Permission
        );
        assert_eq!(
            categorize_error("LLM model failed to load"),
            ErrorCategory::Llm
        );
        assert_eq!(
            categorize_error("Recipe execution failed"),
            ErrorCategory::Recipe
        );
        assert_eq!(
            categorize_error("Something unknown happened"),
            ErrorCategory::Other
        );
    }
}
