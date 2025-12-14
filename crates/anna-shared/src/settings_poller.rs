// v0.0.637: Settings Poller (Phase 213)
// Poller for settings changes with interval support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Watcher type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WatcherType {
    /// Polling watcher
    #[default]
    Polling,
    /// Event-based watcher
    EventBased,
    /// Hybrid watcher
    Hybrid,
    /// Lazy watcher
    Lazy,
    /// Eager watcher
    Eager,
}

impl std::fmt::Display for WatcherType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polling => write!(f, "polling"),
            Self::EventBased => write!(f, "event_based"),
            Self::Hybrid => write!(f, "hybrid"),
            Self::Lazy => write!(f, "lazy"),
            Self::Eager => write!(f, "eager"),
        }
    }
}

/// Watch interval
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WatchInterval {
    /// Immediate
    Immediate,
    /// Fast (100ms)
    Fast,
    /// Normal (1s)
    #[default]
    Normal,
    /// Slow (5s)
    Slow,
    /// Custom interval (ms)
    Custom(u64),
}

impl std::fmt::Display for WatchInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "immediate"),
            Self::Fast => write!(f, "fast"),
            Self::Normal => write!(f, "normal"),
            Self::Slow => write!(f, "slow"),
            Self::Custom(ms) => write!(f, "custom_{}ms", ms),
        }
    }
}

/// Watcher config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Watcher type
    pub watcher_type: WatcherType,
    /// Interval
    pub interval: WatchInterval,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Key pattern
    pub key_pattern: Option<String>,
    /// Active
    pub active: bool,
}

impl WatcherConfig {
    /// Create new config
    pub fn new(watcher_type: WatcherType) -> Self {
        Self {
            watcher_type,
            interval: WatchInterval::Normal,
            category: None,
            key_pattern: None,
            active: true,
        }
    }

    /// Set interval
    pub fn interval(mut self, interval: WatchInterval) -> Self {
        self.interval = interval;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key pattern
    pub fn key_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.key_pattern = Some(pattern.into());
        self
    }
}

/// Watch event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// Event ID
    pub id: String,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Timestamp
    pub timestamp: u64,
}

impl WatchEvent {
    /// Create new event
    pub fn new(
        id: impl Into<String>,
        category: SettingsCategory,
        key: impl Into<String>,
        new_value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            key: key.into(),
            old_value: None,
            new_value: new_value.into(),
            timestamp: 0,
        }
    }

    /// Set old value
    pub fn old_value(mut self, value: impl Into<String>) -> Self {
        self.old_value = Some(value.into());
        self
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    /// Is value change
    pub fn is_change(&self) -> bool {
        self.old_value.as_ref().map_or(false, |old| old != &self.new_value)
    }
}

/// Watcher instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watcher {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Config
    pub config: WatcherConfig,
    /// Created timestamp
    pub created_at: u64,
    /// Last poll timestamp
    pub last_poll: u64,
    /// Event count
    pub event_count: usize,
}

impl Watcher {
    /// Create new watcher
    pub fn new(id: impl Into<String>, name: impl Into<String>, config: WatcherConfig) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            config,
            created_at: 0,
            last_poll: 0,
            event_count: 0,
        }
    }

    /// Set created timestamp
    pub fn created_at(mut self, ts: u64) -> Self {
        self.created_at = ts;
        self
    }

    /// Is active
    pub fn is_active(&self) -> bool {
        self.config.active
    }

    /// Activate
    pub fn activate(&mut self) {
        self.config.active = true;
    }

    /// Deactivate
    pub fn deactivate(&mut self) {
        self.config.active = false;
    }

    /// Record poll
    pub fn record_poll(&mut self, ts: u64) {
        self.last_poll = ts;
    }

    /// Record event
    pub fn record_event(&mut self) {
        self.event_count += 1;
    }

    /// Matches event
    pub fn matches(&self, event: &WatchEvent) -> bool {
        if let Some(cat) = &self.config.category {
            if *cat != event.category {
                return false;
            }
        }
        if let Some(pattern) = &self.config.key_pattern {
            if !event.key.contains(pattern) {
                return false;
            }
        }
        true
    }
}

/// Watcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatcherStats {
    /// Total polls
    pub total_polls: usize,
    /// Events detected
    pub events_detected: usize,
    /// Changes detected
    pub changes_detected: usize,
}

impl WatcherStats {
    /// Record poll
    pub fn record_poll(&mut self) {
        self.total_polls += 1;
    }

    /// Record event
    pub fn record_event(&mut self) {
        self.events_detected += 1;
    }

    /// Record change
    pub fn record_change(&mut self) {
        self.changes_detected += 1;
    }

    /// Change rate
    pub fn change_rate(&self) -> f64 {
        if self.total_polls == 0 {
            0.0
        } else {
            self.changes_detected as f64 / self.total_polls as f64
        }
    }
}

/// Settings watcher registry
#[derive(Debug, Clone, Default)]
pub struct SettingsWatcherRegistry {
    /// Watchers by ID
    watchers: HashMap<String, Watcher>,
    /// Stats by watcher ID
    stats: HashMap<String, WatcherStats>,
}

impl SettingsWatcherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register watcher
    pub fn register(&mut self, watcher: Watcher) {
        let wid = watcher.id.clone();
        self.watchers.insert(wid.clone(), watcher);
        self.stats.entry(wid).or_default();
    }

    /// Unregister watcher
    pub fn unregister(&mut self, id: &str) -> bool {
        self.watchers.remove(id).is_some()
    }

    /// Get watcher
    pub fn get(&self, id: &str) -> Option<&Watcher> {
        self.watchers.get(id)
    }

    /// Get watcher mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Watcher> {
        self.watchers.get_mut(id)
    }

    /// Get stats
    pub fn get_stats(&self, id: &str) -> Option<&WatcherStats> {
        self.stats.get(id)
    }

    /// List by type
    pub fn list_by_type(&self, wtype: WatcherType) -> Vec<&Watcher> {
        self.watchers
            .values()
            .filter(|w| w.config.watcher_type == wtype)
            .collect()
    }

    /// List active
    pub fn list_active(&self) -> Vec<&Watcher> {
        self.watchers.values().filter(|w| w.is_active()).collect()
    }

    /// Watcher count
    pub fn count(&self) -> usize {
        self.watchers.len()
    }

    /// Active count
    pub fn active_count(&self) -> usize {
        self.watchers.values().filter(|w| w.is_active()).count()
    }
}

/// Format watcher registry
pub fn format_watcher_registry(registry: &SettingsWatcherRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Watcher Registry:\n");
    output.push_str(&format!("  Watchers: {}\n", registry.count()));
    output.push_str(&format!("  Active: {}\n", registry.active_count()));
    output
}

/// Check if query is about watcher
pub fn is_watcher_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("watcher") || lower.contains("watch settings") || lower.contains("poll")
}

/// Fun fact about watcher
pub fn watcher_fun_fact() -> &'static str {
    "Anna's settings watchers support both polling and event-based change detection!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_type_display() {
        assert_eq!(format!("{}", WatcherType::Polling), "polling");
        assert_eq!(format!("{}", WatcherType::EventBased), "event_based");
    }

    #[test]
    fn test_interval_display() {
        assert_eq!(format!("{}", WatchInterval::Normal), "normal");
        assert_eq!(format!("{}", WatchInterval::Custom(500)), "custom_500ms");
    }

    #[test]
    fn test_config_new() {
        let c = WatcherConfig::new(WatcherType::Polling);
        assert!(c.active);
    }

    #[test]
    fn test_config_builder() {
        let c = WatcherConfig::new(WatcherType::Hybrid)
            .interval(WatchInterval::Fast)
            .category(SettingsCategory::Privacy);
        assert_eq!(c.interval, WatchInterval::Fast);
    }

    #[test]
    fn test_event_new() {
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(e.old_value.is_none());
    }

    #[test]
    fn test_event_change() {
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "new")
            .old_value("old");
        assert!(e.is_change());
    }

    #[test]
    fn test_watcher_new() {
        let w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        assert!(w.is_active());
    }

    #[test]
    fn test_watcher_poll() {
        let mut w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        w.record_poll(1000);
        assert_eq!(w.last_poll, 1000);
    }

    #[test]
    fn test_watcher_matches() {
        let w = Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling));
        let e = WatchEvent::new("e1", SettingsCategory::Privacy, "key", "value");
        assert!(w.matches(&e));
    }

    #[test]
    fn test_stats_record() {
        let mut s = WatcherStats::default();
        s.record_poll();
        s.record_change();
        assert_eq!(s.total_polls, 1);
        assert_eq!(s.changes_detected, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsWatcherRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsWatcherRegistry::new();
        r.register(Watcher::new("w1", "Test", WatcherConfig::new(WatcherType::Polling)));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_watcher_query() {
        assert!(is_watcher_query("settings watcher"));
        assert!(!is_watcher_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = watcher_fun_fact();
        assert!(fact.contains("watcher"));
    }
}
