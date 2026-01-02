// v0.0.563: Settings History (Phase 139) - Types
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
