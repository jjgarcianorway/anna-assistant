// v0.0.713: Settings Notice Stats (Phase 289)
// Statistics for notices

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::entry::NoticeEntry;
use super::types::{NoticeType, NoticePriority};

/// Notice stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoticeStats {
    /// Total notices
    pub total_notices: usize,
    /// Acknowledged notices
    pub acknowledged: usize,
    /// Urgent notices
    pub urgent: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl NoticeStats {
    /// Update from entries
    pub fn update(&mut self, entries: &[NoticeEntry], notice_type: NoticeType) {
        self.total_notices = entries.len();
        self.acknowledged = entries.iter().filter(|e| e.acknowledged).count();
        self.urgent = entries.iter().filter(|e| e.priority == NoticePriority::Urgent).count();
        *self.by_type.entry(notice_type.to_string()).or_insert(0) += 1;
    }

    /// Acknowledgment rate
    pub fn ack_rate(&self) -> f64 {
        if self.total_notices == 0 { 0.0 } else { self.acknowledged as f64 / self.total_notices as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = NoticeStats::default();
        let mut entry = NoticeEntry::new("e1", "Title", "Message").priority(NoticePriority::Urgent);
        entry.acknowledge();
        s.update(&[entry], NoticeType::Alert);
        assert_eq!(s.total_notices, 1);
        assert_eq!(s.acknowledged, 1);
        assert_eq!(s.urgent, 1);
    }
}
