//! Config Change Tracker - Phase 82
//!
//! Tracks configuration file changes made by Anna.
//! VISION.md mentions Anna editing config files and keeping track of changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of configuration change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeType {
    Add,
    Modify,
    Delete,
    Replace,
    Append,
    Comment,
    Uncomment,
}

impl ChangeType {
    pub fn symbol(&self) -> &'static str {
        match self {
            ChangeType::Add => "+",
            ChangeType::Modify => "~",
            ChangeType::Delete => "-",
            ChangeType::Replace => "=",
            ChangeType::Append => ">",
            ChangeType::Comment => "#",
            ChangeType::Uncomment => "!",
        }
    }

    pub fn verb(&self) -> &'static str {
        match self {
            ChangeType::Add => "added",
            ChangeType::Modify => "modified",
            ChangeType::Delete => "deleted",
            ChangeType::Replace => "replaced",
            ChangeType::Append => "appended",
            ChangeType::Comment => "commented",
            ChangeType::Uncomment => "uncommented",
        }
    }
}

/// Category of config file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigCategory {
    Shell,
    Editor,
    Git,
    System,
    Service,
    Application,
    Network,
    Security,
    Other,
}

impl ConfigCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ConfigCategory::Shell => "Shell",
            ConfigCategory::Editor => "Editor",
            ConfigCategory::Git => "Git",
            ConfigCategory::System => "System",
            ConfigCategory::Service => "Service",
            ConfigCategory::Application => "Application",
            ConfigCategory::Network => "Network",
            ConfigCategory::Security => "Security",
            ConfigCategory::Other => "Other",
        }
    }
}

/// A single config change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeRecord {
    /// Unique change ID
    pub id: String,
    /// File path that was changed
    pub file_path: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Category of config
    pub category: ConfigCategory,
    /// What was changed (line/setting name)
    pub target: String,
    /// Old value (if applicable)
    pub old_value: Option<String>,
    /// New value (if applicable)
    pub new_value: Option<String>,
    /// Timestamp
    pub timestamp: u64,
    /// Associated ticket ID
    pub ticket_id: Option<String>,
    /// Reason for change
    pub reason: Option<String>,
    /// Whether user confirmed
    pub user_confirmed: bool,
    /// Backup ID if backup was created
    pub backup_id: Option<String>,
    /// Whether change was rolled back
    pub rolled_back: bool,
}

/// Config change tracker
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigChangeTracker {
    /// All change records
    pub records: Vec<ConfigChangeRecord>,
    /// Count by file
    pub by_file: HashMap<String, u64>,
    /// Count by category
    pub by_category: HashMap<String, u64>,
    /// Count by change type
    pub by_type: HashMap<String, u64>,
    /// Total rollbacks
    pub rollback_count: u64,
}

impl ConfigChangeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a config change
    pub fn record(&mut self, change: ConfigChangeRecord) {
        *self.by_file.entry(change.file_path.clone()).or_insert(0) += 1;
        *self.by_category.entry(change.category.name().to_string()).or_insert(0) += 1;
        *self.by_type.entry(format!("{:?}", change.change_type)).or_insert(0) += 1;

        self.records.push(change);
    }

    /// Mark a change as rolled back
    pub fn mark_rolled_back(&mut self, id: &str) -> bool {
        let found = self.records.iter().position(|r| r.id == id);
        if let Some(idx) = found {
            self.records[idx].rolled_back = true;
            self.rollback_count += 1;
            true
        } else {
            false
        }
    }

    /// Get recent changes
    pub fn recent(&self, limit: usize) -> Vec<&ConfigChangeRecord> {
        self.records.iter().rev().take(limit).collect()
    }

    /// Get changes for a file
    pub fn for_file(&self, path: &str) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.file_path == path).collect()
    }

    /// Get changes by category
    pub fn by_config_category(&self, category: ConfigCategory) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.category == category).collect()
    }

    /// Get changes by type
    pub fn by_change_type(&self, change_type: ChangeType) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.change_type == change_type).collect()
    }

    /// Get rolled back changes
    pub fn rolled_back(&self) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| r.rolled_back).collect()
    }

    /// Get active (not rolled back) changes
    pub fn active(&self) -> Vec<&ConfigChangeRecord> {
        self.records.iter().filter(|r| !r.rolled_back).collect()
    }

    /// Total change count
    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    /// Unique files changed
    pub fn unique_files(&self) -> usize {
        self.by_file.len()
    }

    /// Most changed file
    pub fn most_changed_file(&self) -> Option<(&str, u64)> {
        self.by_file
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// Most common category
    pub fn most_common_category(&self) -> Option<(&str, u64)> {
        self.by_category
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, v)| (k.as_str(), *v))
    }
}

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

/// Format config change tracker for display
pub fn format_config_tracker(tracker: &ConfigChangeTracker) -> String {
    let mut lines = vec!["=== Configuration Change History ===".to_string()];
    lines.push(String::new());

    if tracker.records.is_empty() {
        lines.push("No config changes yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total changes: {}", tracker.total_count()));
    lines.push(format!("Unique files: {}", tracker.unique_files()));
    lines.push(format!("Rollbacks: {}", tracker.rollback_count));

    // By category
    if !tracker.by_category.is_empty() {
        lines.push(String::new());
        lines.push("By category:".to_string());
        for (cat, count) in &tracker.by_category {
            lines.push(format!("  {}: {}", cat, count));
        }
    }

    // Most changed
    if let Some((file, count)) = tracker.most_changed_file() {
        lines.push(String::new());
        lines.push(format!("Most changed: {} ({} changes)", file, count));
    }

    // Recent changes
    let recent = tracker.recent(5);
    if !recent.is_empty() {
        lines.push(String::new());
        lines.push("Recent changes:".to_string());
        for change in recent {
            let symbol = change.change_type.symbol();
            let rolled_back = if change.rolled_back { " [rolled back]" } else { "" };
            lines.push(format!(
                "  [{}] {} - {}{}",
                symbol, change.file_path, change.target, rolled_back
            ));
        }
    }

    lines.join("\n")
}

/// Format config tracker compact
pub fn format_config_tracker_compact(tracker: &ConfigChangeTracker) -> String {
    format!(
        "Config: {} changes | {} files | {} rollbacks",
        tracker.total_count(),
        tracker.unique_files(),
        tracker.rollback_count
    )
}

/// Format config tracker one-line
pub fn format_config_tracker_oneline(tracker: &ConfigChangeTracker) -> String {
    format!(
        "{} config changes ({} files)",
        tracker.total_count(),
        tracker.unique_files()
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_change(file: &str, change_type: ChangeType) -> ConfigChangeRecord {
        ConfigChangeRecord {
            id: format!("CHG-{}", file.len()),
            file_path: file.to_string(),
            change_type,
            category: detect_category(file),
            target: "test_setting".to_string(),
            old_value: Some("old".to_string()),
            new_value: Some("new".to_string()),
            timestamp: 1234567890,
            ticket_id: None,
            reason: Some("test".to_string()),
            user_confirmed: true,
            backup_id: Some("BKP-001".to_string()),
            rolled_back: false,
        }
    }

    #[test]
    fn test_change_type() {
        assert_eq!(ChangeType::Add.symbol(), "+");
        assert_eq!(ChangeType::Modify.verb(), "modified");
    }

    #[test]
    fn test_config_category() {
        assert_eq!(ConfigCategory::Shell.name(), "Shell");
        assert_eq!(ConfigCategory::Editor.name(), "Editor");
    }

    #[test]
    fn test_detect_category() {
        assert_eq!(detect_category("/home/user/.bashrc"), ConfigCategory::Shell);
        assert_eq!(detect_category("/home/user/.vimrc"), ConfigCategory::Editor);
        assert_eq!(detect_category("/home/user/.gitconfig"), ConfigCategory::Git);
        assert_eq!(detect_category("/etc/systemd/system/foo.service"), ConfigCategory::Service);
    }

    #[test]
    fn test_config_tracker_record() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));

        assert_eq!(tracker.total_count(), 1);
        assert_eq!(tracker.unique_files(), 1);
    }

    #[test]
    fn test_mark_rolled_back() {
        let mut tracker = ConfigChangeTracker::new();
        let mut change = make_change("/home/user/.bashrc", ChangeType::Add);
        change.id = "CHG-001".to_string();
        tracker.record(change);

        assert!(tracker.mark_rolled_back("CHG-001"));
        assert_eq!(tracker.rollback_count, 1);
        assert_eq!(tracker.rolled_back().len(), 1);
    }

    #[test]
    fn test_for_file() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Modify));
        tracker.record(make_change("/home/user/.vimrc", ChangeType::Add));

        assert_eq!(tracker.for_file("/home/user/.bashrc").len(), 2);
        assert_eq!(tracker.for_file("/home/user/.vimrc").len(), 1);
    }

    #[test]
    fn test_by_config_category() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));
        tracker.record(make_change("/home/user/.vimrc", ChangeType::Add));

        assert_eq!(tracker.by_config_category(ConfigCategory::Shell).len(), 1);
        assert_eq!(tracker.by_config_category(ConfigCategory::Editor).len(), 1);
    }

    #[test]
    fn test_most_changed_file() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Modify));
        tracker.record(make_change("/home/user/.vimrc", ChangeType::Add));

        let (file, count) = tracker.most_changed_file().unwrap();
        assert_eq!(file, "/home/user/.bashrc");
        assert_eq!(count, 2);
    }

    #[test]
    fn test_format_config_tracker() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));

        let output = format_config_tracker(&tracker);
        assert!(output.contains("Configuration Change History"));
        assert!(output.contains("Total changes: 1"));
    }

    #[test]
    fn test_is_config_tracker_query() {
        assert!(is_config_tracker_query("show config changes"));
        assert!(is_config_tracker_query("what configuration files changed?"));
        assert!(is_config_tracker_query("config history"));
        assert!(!is_config_tracker_query("what is my disk space?"));
    }

    #[test]
    fn test_config_fun_fact() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));

        let fact = config_fun_fact(&tracker);
        assert!(!fact.is_empty());
    }

    #[test]
    fn test_format_compact_oneline() {
        let mut tracker = ConfigChangeTracker::new();
        tracker.record(make_change("/home/user/.bashrc", ChangeType::Add));

        let compact = format_config_tracker_compact(&tracker);
        assert!(compact.contains("Config: 1 changes"));

        let oneline = format_config_tracker_oneline(&tracker);
        assert!(oneline.contains("1 config changes"));
    }
}
