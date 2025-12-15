// v0.0.692: Settings Chronicle (Phase 268)
// Track settings changes over time

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Track event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ChronicleEvent {
    /// Value changed
    #[default]
    Changed,
    /// Value added
    Added,
    /// Value removed
    Removed,
    /// Value accessed
    Accessed,
}

impl std::fmt::Display for ChronicleEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changed => write!(f, "changed"),
            Self::Added => write!(f, "added"),
            Self::Removed => write!(f, "removed"),
            Self::Accessed => write!(f, "accessed"),
        }
    }
}

/// Track mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ChronicleMode {
    /// Track all changes
    #[default]
    All,
    /// Track writes only
    WritesOnly,
    /// Track specific keys
    Specific,
    /// Track patterns
    Pattern,
}

impl std::fmt::Display for ChronicleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::WritesOnly => write!(f, "writes_only"),
            Self::Specific => write!(f, "specific"),
            Self::Pattern => write!(f, "pattern"),
        }
    }
}

/// Chronicle config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleConfig {
    /// Track mode
    pub mode: ChronicleMode,
    /// Enabled
    pub enabled: bool,
    /// Max history
    pub max_history: usize,
    /// Track patterns
    pub patterns: Vec<String>,
}

impl ChronicleConfig {
    /// Create new config
    pub fn new(mode: ChronicleMode) -> Self {
        Self {
            mode,
            enabled: true,
            max_history: 100,
            patterns: Vec::new(),
        }
    }

    /// Set max history
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = max;
        self
    }

    /// Add pattern
    pub fn add_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }
}

impl Default for ChronicleConfig {
    fn default() -> Self {
        Self::new(ChronicleMode::All)
    }
}

/// Track record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleRecord {
    /// Key
    pub key: String,
    /// Event type
    pub event: ChronicleEvent,
    /// Old value
    pub old_value: Option<String>,
    /// New value
    pub new_value: Option<String>,
    /// Sequence number
    pub sequence: usize,
}

impl ChronicleRecord {
    /// Create new record
    pub fn new(key: impl Into<String>, event: ChronicleEvent, sequence: usize) -> Self {
        Self {
            key: key.into(),
            event,
            old_value: None,
            new_value: None,
            sequence,
        }
    }

    /// Set old value
    pub fn old_value(mut self, val: impl Into<String>) -> Self {
        self.old_value = Some(val.into());
        self
    }

    /// Set new value
    pub fn new_value(mut self, val: impl Into<String>) -> Self {
        self.new_value = Some(val.into());
        self
    }

    /// Is modification
    pub fn is_modification(&self) -> bool {
        matches!(self.event, ChronicleEvent::Changed | ChronicleEvent::Added | ChronicleEvent::Removed)
    }
}

/// Track history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleHistory {
    /// Records
    pub records: Vec<ChronicleRecord>,
    /// By key
    pub by_key: HashMap<String, Vec<usize>>,
}

impl ChronicleHistory {
    /// Create new history
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            by_key: HashMap::new(),
        }
    }

    /// Add record
    pub fn add(&mut self, record: ChronicleRecord) {
        let idx = self.records.len();
        self.by_key.entry(record.key.clone()).or_default().push(idx);
        self.records.push(record);
    }

    /// Get history for key
    pub fn for_key(&self, key: &str) -> Vec<&ChronicleRecord> {
        self.by_key.get(key)
            .map(|indices| indices.iter().filter_map(|&i| self.records.get(i)).collect())
            .unwrap_or_default()
    }

    /// Total records
    pub fn total(&self) -> usize {
        self.records.len()
    }
}

impl Default for ChronicleHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Chronicle stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChronicleStats {
    /// Total tracked
    pub total_tracked: usize,
    /// Changes
    pub changes: usize,
    /// Adds
    pub adds: usize,
    /// Removes
    pub removes: usize,
    /// By key
    pub by_key: HashMap<String, usize>,
}

impl ChronicleStats {
    /// Record
    pub fn record(&mut self, rec: &ChronicleRecord) {
        self.total_tracked += 1;
        match rec.event {
            ChronicleEvent::Changed => self.changes += 1,
            ChronicleEvent::Added => self.adds += 1,
            ChronicleEvent::Removed => self.removes += 1,
            ChronicleEvent::Accessed => {}
        }
        *self.by_key.entry(rec.key.clone()).or_insert(0) += 1;
    }

    /// Most active key
    pub fn most_active(&self) -> Option<(&String, &usize)> {
        self.by_key.iter().max_by_key(|(_, v)| *v)
    }
}

/// Settings chronicle
#[derive(Debug, Clone, Default)]
pub struct SettingsChronicle {
    /// Config
    config: ChronicleConfig,
    /// History
    history: ChronicleHistory,
    /// Stats
    stats: ChronicleStats,
    /// Next sequence
    next_seq: usize,
}

impl SettingsChronicle {
    /// Create new chronicle
    pub fn new(config: ChronicleConfig) -> Self {
        Self {
            config,
            history: ChronicleHistory::new(),
            stats: ChronicleStats::default(),
            next_seq: 1,
        }
    }

    /// Should track key
    fn should_track(&self, key: &str) -> bool {
        if !self.config.enabled {
            return false;
        }
        match self.config.mode {
            ChronicleMode::All | ChronicleMode::WritesOnly => true,
            ChronicleMode::Specific => self.config.patterns.contains(&key.to_string()),
            ChronicleMode::Pattern => self.config.patterns.iter().any(|p| key.contains(p)),
        }
    }

    /// Track change
    pub fn track_change(&mut self, key: &str, old: &str, new: &str) {
        if !self.should_track(key) {
            return;
        }
        let record = ChronicleRecord::new(key, ChronicleEvent::Changed, self.next_seq)
            .old_value(old)
            .new_value(new);
        self.next_seq += 1;
        self.stats.record(&record);
        self.history.add(record);
        self.trim_history();
    }

    /// Track add
    pub fn track_add(&mut self, key: &str, value: &str) {
        if !self.should_track(key) {
            return;
        }
        let record = ChronicleRecord::new(key, ChronicleEvent::Added, self.next_seq)
            .new_value(value);
        self.next_seq += 1;
        self.stats.record(&record);
        self.history.add(record);
        self.trim_history();
    }

    /// Track remove
    pub fn track_remove(&mut self, key: &str, old_value: &str) {
        if !self.should_track(key) {
            return;
        }
        let record = ChronicleRecord::new(key, ChronicleEvent::Removed, self.next_seq)
            .old_value(old_value);
        self.next_seq += 1;
        self.stats.record(&record);
        self.history.add(record);
        self.trim_history();
    }

    /// Track access
    pub fn track_access(&mut self, key: &str) {
        if !self.should_track(key) || self.config.mode == ChronicleMode::WritesOnly {
            return;
        }
        let record = ChronicleRecord::new(key, ChronicleEvent::Accessed, self.next_seq);
        self.next_seq += 1;
        self.stats.record(&record);
        self.history.add(record);
        self.trim_history();
    }

    /// Trim history
    fn trim_history(&mut self) {
        while self.history.records.len() > self.config.max_history {
            self.history.records.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &ChronicleHistory {
        &self.history
    }

    /// Get stats
    pub fn stats(&self) -> &ChronicleStats {
        &self.stats
    }
}

/// Chronicle registry
#[derive(Debug, Clone, Default)]
pub struct ChronicleRegistry {
    /// Chronicles by ID
    chronicles: HashMap<String, SettingsChronicle>,
}

impl ChronicleRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register chronicle
    pub fn register(&mut self, id: impl Into<String>, chronicle: SettingsChronicle) {
        self.chronicles.insert(id.into(), chronicle);
    }

    /// Unregister chronicle
    pub fn unregister(&mut self, id: &str) -> bool {
        self.chronicles.remove(id).is_some()
    }

    /// Get chronicle
    pub fn get(&self, id: &str) -> Option<&SettingsChronicle> {
        self.chronicles.get(id)
    }

    /// Get chronicle mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsChronicle> {
        self.chronicles.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.chronicles.len()
    }
}

/// Format chronicle registry
pub fn format_chronicle_registry(registry: &ChronicleRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Chronicle Registry:\n");
    output.push_str(&format!("  Chronicles: {}\n", registry.count()));
    output
}

/// Check if query is about chronicle
pub fn is_chronicle_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("chronicle settings") || lower.contains("settings chronicle") || lower.contains("settings changes")
}

/// Fun fact about chronicle
pub fn chronicle_fun_fact() -> &'static str {
    "Anna's settings chronicle monitors every configuration change in real-time!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_event_display() {
        assert_eq!(format!("{}", ChronicleEvent::Changed), "changed");
        assert_eq!(format!("{}", ChronicleEvent::Added), "added");
    }

    #[test]
    fn test_track_mode_display() {
        assert_eq!(format!("{}", ChronicleMode::All), "all");
        assert_eq!(format!("{}", ChronicleMode::WritesOnly), "writes_only");
    }

    #[test]
    fn test_config_new() {
        let c = ChronicleConfig::new(ChronicleMode::All);
        assert!(c.enabled);
    }

    #[test]
    fn test_config_builder() {
        let c = ChronicleConfig::new(ChronicleMode::Pattern)
            .max_history(50)
            .add_pattern("app.");
        assert_eq!(c.max_history, 50);
        assert_eq!(c.patterns.len(), 1);
    }

    #[test]
    fn test_record_new() {
        let r = ChronicleRecord::new("key", ChronicleEvent::Changed, 1);
        assert!(r.is_modification());
    }

    #[test]
    fn test_record_values() {
        let r = ChronicleRecord::new("key", ChronicleEvent::Changed, 1)
            .old_value("old")
            .new_value("new");
        assert_eq!(r.old_value, Some("old".to_string()));
    }

    #[test]
    fn test_history_new() {
        let h = ChronicleHistory::new();
        assert_eq!(h.total(), 0);
    }

    #[test]
    fn test_history_add() {
        let mut h = ChronicleHistory::new();
        h.add(ChronicleRecord::new("key", ChronicleEvent::Added, 1));
        assert_eq!(h.total(), 1);
    }

    #[test]
    fn test_history_for_key() {
        let mut h = ChronicleHistory::new();
        h.add(ChronicleRecord::new("key1", ChronicleEvent::Added, 1));
        h.add(ChronicleRecord::new("key2", ChronicleEvent::Added, 2));
        h.add(ChronicleRecord::new("key1", ChronicleEvent::Changed, 3));
        assert_eq!(h.for_key("key1").len(), 2);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ChronicleStats::default();
        s.record(&ChronicleRecord::new("key", ChronicleEvent::Changed, 1));
        assert_eq!(s.changes, 1);
    }

    #[test]
    fn test_chronicle_new() {
        let t = SettingsChronicle::new(ChronicleConfig::default());
        assert_eq!(t.stats().total_tracked, 0);
    }

    #[test]
    fn test_chronicle_track_change() {
        let mut t = SettingsChronicle::new(ChronicleConfig::default());
        t.track_change("key", "old", "new");
        assert_eq!(t.stats().changes, 1);
    }

    #[test]
    fn test_chronicle_track_add() {
        let mut t = SettingsChronicle::new(ChronicleConfig::default());
        t.track_add("key", "value");
        assert_eq!(t.stats().adds, 1);
    }

    #[test]
    fn test_chronicle_track_remove() {
        let mut t = SettingsChronicle::new(ChronicleConfig::default());
        t.track_remove("key", "old");
        assert_eq!(t.stats().removes, 1);
    }

    #[test]
    fn test_registry_new() {
        let r = ChronicleRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ChronicleRegistry::new();
        r.register("t1", SettingsChronicle::new(ChronicleConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_chronicle_query() {
        assert!(is_chronicle_query("chronicle settings"));
        assert!(!is_chronicle_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = chronicle_fun_fact();
        assert!(fact.contains("chronicle"));
    }
}
