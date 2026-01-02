//! Tests for config change tracker

#[cfg(test)]
mod tests {
    use super::super::{
        formatters::{format_config_tracker, format_config_tracker_compact, format_config_tracker_oneline},
        types::{ChangeType, ConfigCategory, ConfigChangeRecord, ConfigChangeTracker},
        utils::{config_fun_fact, detect_category, is_config_tracker_query},
    };

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
