// v0.0.563: Settings History (Phase 139) - Manager
// Manages the history of settings changes with undo/redo support

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};
use super::types::HistoryEntry;

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
