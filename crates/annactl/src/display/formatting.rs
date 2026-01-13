//! Formatting utilities for time, duration, and display helpers.

use std::process::Command;

/// Format duration in human-readable form
pub fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Format RFC3339 timestamp as relative time (e.g., "2m ago", "in 30s")
pub fn format_time_ago(rfc3339: &str) -> String {
    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(rfc3339) {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(ts.with_timezone(&chrono::Utc));
        let secs = diff.num_seconds();
        if secs < 0 {
            format!("in {}", format_duration((-secs) as u64))
        } else {
            format!("{} ago", format_duration(secs as u64))
        }
    } else {
        rfc3339.to_string()
    }
}

/// Get user groups from system
#[allow(dead_code)]
pub fn get_user_groups() -> String {
    Command::new("groups")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// Get list of helpers with provenance (name, installed_by_anna)
pub fn get_helpers_list() -> Vec<(String, bool)> {
    let deps_path = anna_shared::paths::paths().installed_deps_file();

    let anna_installed: std::collections::HashSet<String> = if deps_path.exists() {
        std::fs::read_to_string(&deps_path)
            .ok()
            .map(|c| c.lines().filter(|l| !l.is_empty()).map(|s| s.to_string()).collect())
            .unwrap_or_default()
    } else {
        std::collections::HashSet::new()
    };

    let tools = ["nethogs", "iotop", "htop", "lsof", "strace", "bc", "jq", "yq", "fzf"];
    let mut result = Vec::new();

    for tool in tools {
        if Command::new("which")
            .arg(tool)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let by_anna = anna_installed.contains(tool);
            result.push((tool.to_string(), by_anna));
        }
    }

    result
}

/// Count local documentation files (wiki, man, help)
pub fn count_local_docs() -> (usize, usize, usize) {
    use anna_shared::docs::{man_cache_dir, help_cache_dir};
    use anna_shared::wiki::wiki_articles_dir;

    let wiki_count = std::fs::read_dir(wiki_articles_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let man_count = std::fs::read_dir(man_cache_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    let help_count = std::fs::read_dir(help_cache_dir())
        .map(|entries| entries.filter_map(|e| e.ok()).count())
        .unwrap_or(0);

    (wiki_count, man_count, help_count)
}

/// Check if debug mode is enabled in config
pub fn is_debug_mode() -> bool {
    anna_shared::config::AnnaConfig::load()
        .map(|c| c.debug_mode)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(7200), "2h 0m");
        assert_eq!(format_duration(86399), "23h 59m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1d 0h");
        assert_eq!(format_duration(172800), "2d 0h");
        assert_eq!(format_duration(90000), "1d 1h");
    }

    #[test]
    fn test_format_time_ago_past() {
        let now = chrono::Utc::now();
        let past = now - chrono::Duration::seconds(120);
        let result = format_time_ago(&past.to_rfc3339());
        assert!(result.contains("ago"));
        assert!(result.contains("2m"));
    }

    #[test]
    fn test_format_time_ago_future() {
        let now = chrono::Utc::now();
        let future = now + chrono::Duration::seconds(60);
        let result = format_time_ago(&future.to_rfc3339());
        assert!(result.contains("in "));
    }

    #[test]
    fn test_format_time_ago_invalid() {
        let result = format_time_ago("not-a-date");
        assert_eq!(result, "not-a-date");
    }
}
