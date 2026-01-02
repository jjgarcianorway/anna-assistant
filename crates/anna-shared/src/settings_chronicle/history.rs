// v0.0.692: Settings Chronicle History (Phase 268)
// Track history of changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::ChronicleRecord;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_chronicle::types::ChronicleEvent;

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
}
