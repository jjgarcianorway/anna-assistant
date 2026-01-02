// v0.0.692: Settings Chronicle Stats (Phase 268)
// Statistics tracking for chronicle

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::record::ChronicleRecord;
use super::types::ChronicleEvent;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_record() {
        let mut s = ChronicleStats::default();
        s.record(&ChronicleRecord::new("key", ChronicleEvent::Changed, 1));
        assert_eq!(s.changes, 1);
    }
}
