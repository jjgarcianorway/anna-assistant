//! Utility functions for config change tracking

use super::types::{ConfigCategory, ConfigChangeTracker};

/// Detect config category from file path
pub fn detect_category(path: &str) -> ConfigCategory {
    let path_lower = path.to_lowercase();

    if path_lower.contains(".bashrc")
        || path_lower.contains(".zshrc")
        || path_lower.contains(".profile")
        || path_lower.contains("fish/config")
    {
        ConfigCategory::Shell
    } else if path_lower.contains(".vimrc")
        || path_lower.contains("nvim")
        || path_lower.contains(".nanorc")
        || path_lower.contains("emacs")
    {
        ConfigCategory::Editor
    } else if path_lower.contains(".gitconfig") || path_lower.contains(".gitignore") {
        ConfigCategory::Git
    } else if path_lower.contains("/etc/systemd") || path_lower.contains(".service") {
        ConfigCategory::Service
    } else if path_lower.contains("/etc/network")
        || path_lower.contains("resolv.conf")
        || path_lower.contains("hosts")
    {
        ConfigCategory::Network
    } else if path_lower.contains("/etc/ssh")
        || path_lower.contains("sudoers")
        || path_lower.contains("passwd")
    {
        ConfigCategory::Security
    } else if path_lower.starts_with("/etc/") {
        ConfigCategory::System
    } else {
        ConfigCategory::Application
    }
}

/// Check if query is about config changes
pub fn is_config_tracker_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "config change",
        "config history",
        "configuration change",
        "file changes",
        "what config",
        "changed config",
        "modified config",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about config changes
pub fn config_fun_fact(tracker: &ConfigChangeTracker) -> String {
    if tracker.records.is_empty() {
        return "No config changes yet!".to_string();
    }

    let facts = [
        format!(
            "Anna has made {} configuration changes.",
            tracker.total_count()
        ),
        format!(
            "{} unique config files have been modified.",
            tracker.unique_files()
        ),
        {
            if let Some((file, count)) = tracker.most_changed_file() {
                format!("{} is the most frequently modified file ({} changes).", file, count)
            } else {
                "No file stats yet.".to_string()
            }
        },
        format!(
            "{} changes have been rolled back.",
            tracker.rollback_count
        ),
    ];

    facts[tracker.total_count() % facts.len()].clone()
}
