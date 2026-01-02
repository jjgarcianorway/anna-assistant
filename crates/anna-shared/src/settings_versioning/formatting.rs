// v0.0.588: Settings Versioning Formatting (Phase 164)
// Formatting and utility functions for versioning

use super::history::VersionHistory;
use super::types::SettingsVersion;

/// Format version
pub fn format_version(version: &SettingsVersion) -> String {
    let mut output = String::new();

    output.push_str(&format!("Version {} - {}\n", version.version, version.message));
    output.push_str(&format!(
        "  {} | {} changes\n",
        version.timestamp.format("%Y-%m-%d %H:%M"),
        version.change_count()
    ));

    for change in &version.changes {
        output.push_str(&format!(
            "  {} {} {}\n",
            change.change_type, change.category, change.key
        ));
    }

    output
}

/// Format history summary
pub fn format_history(history: &VersionHistory) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Version History ===\n\n");
    output.push_str(&format!(
        "Current: v{} | Total: {} versions\n\n",
        history.current_version(),
        history.count()
    ));

    for version in history.recent(10) {
        output.push_str(&format!(
            "v{}: {} ({} changes)\n",
            version.version,
            version.message,
            version.change_count()
        ));
    }

    output
}

/// Check if query is about versioning
pub fn is_versioning_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("version")
        || lower.contains("history")
        || lower.contains("changelog")
        || lower.contains("compare")
}

/// Fun fact about versioning
pub fn settings_versioning_fun_fact() -> &'static str {
    "Anna tracks every settings change so you can always see what changed and when!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_versioning::types::SettingsVersion;
    use crate::settings_versioning::history::VersionHistory;

    #[test]
    fn test_format_version() {
        let version = SettingsVersion::new(1, "Test version");
        let output = format_version(&version);
        assert!(output.contains("Version 1"));
    }

    #[test]
    fn test_format_history() {
        let history = VersionHistory::new();
        let output = format_history(&history);
        assert!(output.contains("History"));
    }

    #[test]
    fn test_is_versioning_query() {
        assert!(is_versioning_query("show version history"));
        assert!(is_versioning_query("compare versions"));
        assert!(!is_versioning_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_versioning_fun_fact();
        assert!(fact.contains("change"));
    }
}
