// v0.0.638: Settings Tracker (Phase 214)
// Track settings changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TrackingLevel {
    #[default]
    None,
    Basic,
    Full,
}

impl std::fmt::Display for TrackingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Basic => write!(f, "basic"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackerConfig {
    pub level: TrackingLevel,
    pub enabled: bool,
}

impl TrackerConfig {
    pub fn new(level: TrackingLevel) -> Self {
        Self { level, enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedChange {
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SettingsTracker {
    config: TrackerConfig,
    changes: Vec<TrackedChange>,
}

impl SettingsTracker {
    pub fn new(config: TrackerConfig) -> Self {
        Self { config, changes: Vec::new() }
    }

    pub fn track(&mut self, key: &str, old: Option<&str>, new: Option<&str>) {
        if self.config.enabled {
            self.changes.push(TrackedChange {
                key: key.to_string(),
                old_value: old.map(|s| s.to_string()),
                new_value: new.map(|s| s.to_string()),
            });
        }
    }

    pub fn changes(&self) -> &[TrackedChange] {
        &self.changes
    }
}

#[derive(Debug, Clone, Default)]
pub struct TrackerRegistry {
    trackers: HashMap<String, SettingsTracker>,
}

impl TrackerRegistry {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, id: impl Into<String>, tracker: SettingsTracker) {
        self.trackers.insert(id.into(), tracker);
    }
    pub fn get(&self, id: &str) -> Option<&SettingsTracker> { self.trackers.get(id) }
    pub fn count(&self) -> usize { self.trackers.len() }
}

pub fn is_tracker_query(query: &str) -> bool {
    query.to_lowercase().contains("track")
}

pub fn tracker_fun_fact() -> &'static str {
    "Anna tracks settings changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_display() {
        assert_eq!(format!("{}", TrackingLevel::Basic), "basic");
    }

    #[test]
    fn test_tracker_new() {
        let t = SettingsTracker::new(TrackerConfig::default());
        assert!(t.changes().is_empty());
    }

    #[test]
    fn test_registry() {
        let mut r = TrackerRegistry::new();
        r.register("t1", SettingsTracker::new(TrackerConfig::default()));
        assert_eq!(r.count(), 1);
    }
}
