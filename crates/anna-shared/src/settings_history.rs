// v0.0.563: Settings History (Phase 139)
// Tracks changes to settings over time with undo/redo support

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// When the change was made
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Description of the change
    pub description: String,
    /// Category affected (if specific)
    pub category: Option<SettingsCategory>,
    /// Snapshot of settings before change
    pub settings_before: UnifiedSettings,
    /// Snapshot of settings after change
    pub settings_after: UnifiedSettings,
}

impl HistoryEntry {
    /// Create a new history entry
    pub fn new(
        description: impl Into<String>,
        before: UnifiedSettings,
        after: UnifiedSettings,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            description: description.into(),
            category: None,
            settings_before: before,
            settings_after: after,
        }
    }

    /// Add category
    pub fn with_category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Age of this entry
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }

    /// Is recent (within last hour)?
    pub fn is_recent(&self) -> bool {
        self.age() < chrono::Duration::hours(1)
    }
}

/// Settings history manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsHistory {
    /// History entries (oldest first)
    entries: Vec<HistoryEntry>,
    /// Current position in history
    position: usize,
    /// Maximum history size
    max_size: usize,
}

impl SettingsHistory {
    /// Create new history with default max size
    pub fn new() -> Self {
        Self {
            entries: vec![],
            position: 0,
            max_size: 100,
        }
    }

    /// Create with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: vec![],
            position: 0,
            max_size,
        }
    }

    /// Record a change
    pub fn record(
        &mut self,
        description: impl Into<String>,
        before: UnifiedSettings,
        after: UnifiedSettings,
    ) {
        // If we're not at the end, truncate future entries
        if self.position < self.entries.len() {
            self.entries.truncate(self.position);
        }

        let entry = HistoryEntry::new(description, before, after);
        self.entries.push(entry);
        self.position = self.entries.len();

        // Enforce max size
        if self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.position = self.entries.len();
        }
    }

    /// Record a change with category
    pub fn record_with_category(
        &mut self,
        description: impl Into<String>,
        category: SettingsCategory,
        before: UnifiedSettings,
        after: UnifiedSettings,
    ) {
        if self.position < self.entries.len() {
            self.entries.truncate(self.position);
        }

        let entry = HistoryEntry::new(description, before, after).with_category(category);
        self.entries.push(entry);
        self.position = self.entries.len();

        if self.entries.len() > self.max_size {
            self.entries.remove(0);
            self.position = self.entries.len();
        }
    }

    /// Can undo?
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    /// Can redo?
    pub fn can_redo(&self) -> bool {
        self.position < self.entries.len()
    }

    /// Undo last change, returns the settings to restore
    pub fn undo(&mut self) -> Option<UnifiedSettings> {
        if !self.can_undo() {
            return None;
        }

        self.position -= 1;
        Some(self.entries[self.position].settings_before.clone())
    }

    /// Redo last undone change, returns the settings to restore
    pub fn redo(&mut self) -> Option<UnifiedSettings> {
        if !self.can_redo() {
            return None;
        }

        let settings = self.entries[self.position].settings_after.clone();
        self.position += 1;
        Some(settings)
    }

    /// Get last N entries
    pub fn recent(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get all entries
    pub fn all(&self) -> &[HistoryEntry] {
        &self.entries
    }

    /// Get entries for specific category
    pub fn by_category(&self, category: SettingsCategory) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.category == Some(category))
            .collect()
    }

    /// Get current position
    pub fn current_position(&self) -> usize {
        self.position
    }

    /// Get total count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Is empty?
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = 0;
    }

    /// Get entry at position
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// Get latest entry
    pub fn latest(&self) -> Option<&HistoryEntry> {
        self.entries.last()
    }

    /// Undo count available
    pub fn undo_count(&self) -> usize {
        self.position
    }

    /// Redo count available
    pub fn redo_count(&self) -> usize {
        self.entries.len() - self.position
    }
}

/// Format history for display
pub fn format_history(history: &SettingsHistory, count: usize) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "=== Settings History ({} entries) ===\n\n",
        history.count()
    ));

    if history.is_empty() {
        output.push_str("No history entries.\n");
        return output;
    }

    output.push_str(&format!(
        "Position: {}/{} (undo: {}, redo: {})\n\n",
        history.current_position(),
        history.count(),
        history.undo_count(),
        history.redo_count()
    ));

    for (i, entry) in history.recent(count).iter().enumerate() {
        let age = format_age(entry.age());
        let cat = entry
            .category
            .map(|c| format!(" [{}]", c))
            .unwrap_or_default();
        output.push_str(&format!(
            "{}. {} - {}{}\n",
            i + 1,
            age,
            entry.description,
            cat
        ));
    }

    output
}

/// Format duration as human-readable age
fn format_age(duration: chrono::Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// Fun fact about settings history
pub fn settings_history_fun_fact() -> &'static str {
    "Anna remembers your last 100 settings changes - you can always undo and redo!"
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_format_history_empty() {
        let history = SettingsHistory::new();
        let output = format_history(&history, 10);
        assert!(output.contains("No history"));
    }

    #[test]
    fn test_format_age() {
        assert_eq!(format_age(chrono::Duration::seconds(30)), "30s ago");
        assert_eq!(format_age(chrono::Duration::minutes(5)), "5m ago");
        assert_eq!(format_age(chrono::Duration::hours(2)), "2h ago");
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_history_fun_fact();
        assert!(fact.contains("100"));
    }
}
