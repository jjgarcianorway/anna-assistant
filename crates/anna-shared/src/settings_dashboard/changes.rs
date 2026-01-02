// v0.0.579: Settings Dashboard Changes (Phase 155)
// Recent change tracking

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Recent change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentChange {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Category
    pub category: SettingsCategory,
    /// Setting name
    pub setting: String,
    /// Old value summary
    pub old_value: String,
    /// New value summary
    pub new_value: String,
}

impl RecentChange {
    /// Create new change
    pub fn new(
        category: SettingsCategory,
        setting: impl Into<String>,
        old_value: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            category,
            setting: setting.into(),
            old_value: old_value.into(),
            new_value: new_value.into(),
        }
    }

    /// Age of change
    pub fn age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_change_new() {
        let change = RecentChange::new(
            SettingsCategory::Personality,
            "mode",
            "Casual",
            "Professional",
        );
        assert_eq!(change.category, SettingsCategory::Personality);
        assert_eq!(change.setting, "mode");
    }
}
