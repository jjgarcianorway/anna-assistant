// v0.0.682: Settings Source (Phase 258)
// Represents a source of settings with priority

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::settings_collector::types::SourcePriority;

/// Settings source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsSource {
    /// Source ID
    pub id: String,
    /// Source name
    pub name: String,
    /// Priority
    pub priority: SourcePriority,
    /// Settings
    pub settings: HashMap<String, String>,
}

impl SettingsSource {
    /// Create new source
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            priority: SourcePriority::Normal,
            settings: HashMap::new(),
        }
    }

    /// With priority
    pub fn with_priority(mut self, priority: SourcePriority) -> Self {
        self.priority = priority;
        self
    }

    /// With settings
    pub fn with_settings(mut self, settings: HashMap<String, String>) -> Self {
        self.settings = settings;
        self
    }

    /// Add setting
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.settings.insert(key.into(), value.into());
    }

    /// Count
    pub fn count(&self) -> usize {
        self.settings.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_new() {
        let s = SettingsSource::new("s1", "Source 1");
        assert_eq!(s.id, "s1");
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_source_add() {
        let mut s = SettingsSource::new("s1", "Source 1");
        s.add("key", "value");
        assert_eq!(s.count(), 1);
    }

    #[test]
    fn test_source_with_priority() {
        let s = SettingsSource::new("s1", "Source 1")
            .with_priority(SourcePriority::High);
        assert_eq!(s.priority, SourcePriority::High);
    }
}
