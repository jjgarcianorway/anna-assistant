// v0.0.708: Settings Memo (Phase 284)
// Memo statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{MemoType, MemoStatus};
use super::message::MemoMessage;

/// Memo stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoStats {
    /// Total memos
    pub total_memos: usize,
    /// Sent memos
    pub sent_memos: usize,
    /// Read memos
    pub read_memos: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MemoStats {
    /// Update from memos
    pub fn update(&mut self, messages: &[MemoMessage], memo_type: MemoType) {
        self.total_memos = messages.len();
        self.sent_memos = messages.iter().filter(|m| matches!(m.status, MemoStatus::Sent | MemoStatus::Read)).count();
        self.read_memos = messages.iter().filter(|m| m.status == MemoStatus::Read).count();
        *self.by_type.entry(memo_type.to_string()).or_insert(0) += 1;
    }

    /// Read rate
    pub fn read_rate(&self) -> f64 {
        if self.sent_memos == 0 { 0.0 } else { self.read_memos as f64 / self.sent_memos as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = MemoStats::default();
        let mut msg = MemoMessage::new("m1", "Subject", "Body");
        msg.send();
        s.update(&[msg], MemoType::Internal);
        assert_eq!(s.total_memos, 1);
        assert_eq!(s.sent_memos, 1);
    }
}
