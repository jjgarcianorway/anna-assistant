// v0.0.563: Settings History (Phase 139) - Tests
// Tests for settings history functionality

#[cfg(test)]
mod tests {
    use crate::settings_history::{HistoryEntry, SettingsHistory};
    use crate::unified_settings::{SettingsCategory, UnifiedSettings};

    #[test]
    fn test_history_entry_new() {
        let before = UnifiedSettings::default();
        let after = UnifiedSettings::default();
        let entry = HistoryEntry::new("Test change", before, after);
        assert_eq!(entry.description, "Test change");
        assert!(entry.category.is_none());
    }

    #[test]
    fn test_history_entry_with_category() {
        let before = UnifiedSettings::default();
        let after = UnifiedSettings::default();
        let entry = HistoryEntry::new("Test", before, after)
            .with_category(SettingsCategory::Privacy);
        assert_eq!(entry.category, Some(SettingsCategory::Privacy));
    }

    #[test]
    fn test_history_new() {
        let history = SettingsHistory::new();
        assert!(history.is_empty());
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_record() {
        let mut history = SettingsHistory::new();
        let before = UnifiedSettings::default();
        let after = UnifiedSettings::default();

        history.record("First change", before, after);

        assert_eq!(history.count(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_undo() {
        let mut history = SettingsHistory::new();
        let before = UnifiedSettings::default();
        let mut after = UnifiedSettings::default();
        after.learning.enable();

        history.record("Enable learning", before.clone(), after);

        let restored = history.undo();
        assert!(restored.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_history_redo() {
        let mut history = SettingsHistory::new();
        let before = UnifiedSettings::default();
        let mut after = UnifiedSettings::default();
        after.learning.enable();

        history.record("Enable learning", before, after.clone());
        history.undo();

        let restored = history.redo();
        assert!(restored.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_max_size() {
        let mut history = SettingsHistory::with_max_size(3);
        let settings = UnifiedSettings::default();

        for i in 0..5 {
            history.record(format!("Change {}", i), settings.clone(), settings.clone());
        }

        assert_eq!(history.count(), 3);
    }

    #[test]
    fn test_history_truncate_on_new_change() {
        let mut history = SettingsHistory::new();
        let settings = UnifiedSettings::default();

        history.record("Change 1", settings.clone(), settings.clone());
        history.record("Change 2", settings.clone(), settings.clone());
        history.undo(); // Go back to position 1

        history.record("Change 3", settings.clone(), settings.clone());

        // Change 2 should be gone, only Change 1 and Change 3 remain
        assert_eq!(history.count(), 2);
    }

    #[test]
    fn test_history_recent() {
        let mut history = SettingsHistory::new();
        let settings = UnifiedSettings::default();

        for i in 0..5 {
            history.record(format!("Change {}", i), settings.clone(), settings.clone());
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_history_by_category() {
        let mut history = SettingsHistory::new();
        let settings = UnifiedSettings::default();

        history.record_with_category("Privacy change", SettingsCategory::Privacy,
            settings.clone(), settings.clone());
        history.record_with_category("Model change", SettingsCategory::Model,
            settings.clone(), settings.clone());

        let privacy = history.by_category(SettingsCategory::Privacy);
        assert_eq!(privacy.len(), 1);
    }
}
