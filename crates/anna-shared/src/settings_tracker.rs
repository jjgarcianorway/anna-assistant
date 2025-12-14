// v0.0.638: Settings Tracker (Phase 214)
// Tracker for settings usage and access patterns

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Track type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TrackType {
    /// Read tracking
    #[default]
    Read,
    /// Write tracking
    Write,
    /// Access tracking
    Access,
    /// Modify tracking
    Modify,
    /// Delete tracking
    Delete,
}

impl std::fmt::Display for TrackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Access => write!(f, "access"),
            Self::Modify => write!(f, "modify"),
            Self::Delete => write!(f, "delete"),
        }
    }
}

/// Track level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum TrackLevel {
    /// None
    None,
    /// Basic
    #[default]
    Basic,
    /// Detailed
    Detailed,
    /// Full
    Full,
    /// Debug
    Debug,
}

impl std::fmt::Display for TrackLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Basic => write!(f, "basic"),
            Self::Detailed => write!(f, "detailed"),
            Self::Full => write!(f, "full"),
            Self::Debug => write!(f, "debug"),
        }
    }
}

/// Tracker config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerConfig {
    /// Track type
    pub track_type: TrackType,
    /// Level
    pub level: TrackLevel,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Enabled
    pub enabled: bool,
    /// Retention days
    pub retention_days: u32,
}

impl TrackerConfig {
    /// Create new config
    pub fn new(track_type: TrackType) -> Self {
        Self {
            track_type,
            level: TrackLevel::Basic,
            category: None,
            enabled: true,
            retention_days: 30,
        }
    }

    /// Set level
    pub fn level(mut self, level: TrackLevel) -> Self {
        self.level = level;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set retention days
    pub fn retention_days(mut self, days: u32) -> Self {
        self.retention_days = days;
        self
    }
}

/// Track event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEvent {
    /// Event ID
    pub id: String,
    /// Track type
    pub track_type: TrackType,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Source
    pub source: String,
    /// Timestamp
    pub timestamp: u64,
}

impl TrackEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        track_type: TrackType,
        category: SettingsCategory,
        key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            track_type,
            category,
            key: key.into(),
            source: String::new(),
            timestamp: 0,
        }
    }

    /// Set source
    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}

/// Tracker stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackerStats {
    /// Total events
    pub total_events: usize,
    /// Read events
    pub reads: usize,
    /// Write events
    pub writes: usize,
    /// Other events
    pub other: usize,
}

impl TrackerStats {
    /// Record event
    pub fn record(&mut self, track_type: TrackType) {
        self.total_events += 1;
        match track_type {
            TrackType::Read | TrackType::Access => self.reads += 1,
            TrackType::Write | TrackType::Modify | TrackType::Delete => self.writes += 1,
        }
    }

    /// Read ratio
    pub fn read_ratio(&self) -> f64 {
        if self.total_events == 0 {
            0.0
        } else {
            self.reads as f64 / self.total_events as f64
        }
    }
}

/// Settings tracker
#[derive(Debug, Clone, Default)]
pub struct SettingsTracker {
    /// Config
    config: TrackerConfig,
    /// Events
    events: Vec<TrackEvent>,
    /// Stats
    stats: TrackerStats,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self::new(TrackType::Read)
    }
}

impl SettingsTracker {
    /// Create new tracker
    pub fn new(config: TrackerConfig) -> Self {
        Self {
            config,
            events: Vec::new(),
            stats: TrackerStats::default(),
        }
    }

    /// Track event
    pub fn track(&mut self, event: TrackEvent) {
        if self.config.enabled {
            self.stats.record(event.track_type);
            self.events.push(event);
        }
    }

    /// Get events
    pub fn events(&self) -> &[TrackEvent] {
        &self.events
    }

    /// Get stats
    pub fn stats(&self) -> &TrackerStats {
        &self.stats
    }

    /// Event count
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Clear events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable
    pub fn enable(&mut self) {
        self.config.enabled = true;
    }

    /// Disable
    pub fn disable(&mut self) {
        self.config.enabled = false;
    }

    /// List by type
    pub fn list_by_type(&self, track_type: TrackType) -> Vec<&TrackEvent> {
        self.events.iter().filter(|e| e.track_type == track_type).collect()
    }

    /// List by category
    pub fn list_by_category(&self, category: SettingsCategory) -> Vec<&TrackEvent> {
        self.events.iter().filter(|e| e.category == category).collect()
    }
}

/// Settings tracker registry
#[derive(Debug, Clone, Default)]
pub struct SettingsTrackerRegistry {
    /// Trackers by ID
    trackers: HashMap<String, SettingsTracker>,
}

impl SettingsTrackerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register tracker
    pub fn register(&mut self, id: impl Into<String>, tracker: SettingsTracker) {
        self.trackers.insert(id.into(), tracker);
    }

    /// Unregister tracker
    pub fn unregister(&mut self, id: &str) -> bool {
        self.trackers.remove(id).is_some()
    }

    /// Get tracker
    pub fn get(&self, id: &str) -> Option<&SettingsTracker> {
        self.trackers.get(id)
    }

    /// Get tracker mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsTracker> {
        self.trackers.get_mut(id)
    }

    /// Tracker count
    pub fn count(&self) -> usize {
        self.trackers.len()
    }

    /// List enabled
    pub fn list_enabled(&self) -> Vec<&SettingsTracker> {
        self.trackers.values().filter(|t| t.is_enabled()).collect()
    }
}

/// Format tracker registry
pub fn format_tracker_registry(registry: &SettingsTrackerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Tracker Registry:\n");
    output.push_str(&format!("  Trackers: {}\n", registry.count()));
    output
}

/// Check if query is about tracker
pub fn is_tracker_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("tracker") || lower.contains("track settings") || lower.contains("usage pattern")
}

/// Fun fact about tracker
pub fn tracker_fun_fact() -> &'static str {
    "Anna's settings trackers monitor access patterns for optimization!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_type_display() {
        assert_eq!(format!("{}", TrackType::Read), "read");
        assert_eq!(format!("{}", TrackType::Write), "write");
    }

    #[test]
    fn test_level_display() {
        assert_eq!(format!("{}", TrackLevel::Basic), "basic");
        assert_eq!(format!("{}", TrackLevel::Full), "full");
    }

    #[test]
    fn test_config_new() {
        let c = TrackerConfig::new(TrackType::Read);
        assert!(c.enabled);
        assert_eq!(c.retention_days, 30);
    }

    #[test]
    fn test_config_builder() {
        let c = TrackerConfig::new(TrackType::Write)
            .level(TrackLevel::Detailed)
            .retention_days(7);
        assert_eq!(c.level, TrackLevel::Detailed);
        assert_eq!(c.retention_days, 7);
    }

    #[test]
    fn test_event_new() {
        let e = TrackEvent::new("e1", TrackType::Read, SettingsCategory::Privacy, "key");
        assert!(e.source.is_empty());
    }

    #[test]
    fn test_event_source() {
        let e = TrackEvent::new("e1", TrackType::Read, SettingsCategory::Privacy, "key")
            .source("app");
        assert_eq!(e.source, "app");
    }

    #[test]
    fn test_stats_record() {
        let mut s = TrackerStats::default();
        s.record(TrackType::Read);
        s.record(TrackType::Write);
        assert_eq!(s.total_events, 2);
        assert_eq!(s.reads, 1);
        assert_eq!(s.writes, 1);
    }

    #[test]
    fn test_tracker_new() {
        let t = SettingsTracker::new(TrackerConfig::new(TrackType::Read));
        assert!(t.is_enabled());
    }

    #[test]
    fn test_tracker_track() {
        let mut t = SettingsTracker::new(TrackerConfig::new(TrackType::Read));
        t.track(TrackEvent::new("e1", TrackType::Read, SettingsCategory::Privacy, "key"));
        assert_eq!(t.event_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsTrackerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsTrackerRegistry::new();
        r.register("t1", SettingsTracker::new(TrackerConfig::new(TrackType::Read)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_tracker_query() {
        assert!(is_tracker_query("settings tracker"));
        assert!(!is_tracker_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = tracker_fun_fact();
        assert!(fact.contains("tracker"));
    }
}
