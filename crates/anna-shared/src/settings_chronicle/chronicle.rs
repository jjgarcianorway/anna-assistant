// v0.0.692: Settings Chronicle Main (Phase 268)
// Main chronicle implementation

use super::config::ChronicleConfig;
use super::history::ChronicleHistory;
use super::record::ChronicleRecord;
use super::stats::ChronicleStats;
use super::types::{ChronicleEvent, ChronicleMode};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
