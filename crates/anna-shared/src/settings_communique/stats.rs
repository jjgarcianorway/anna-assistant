// v0.0.715: Settings Communique - Stats (Phase 291)
// Communique statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::CommuniqueType;
use super::message::CommuniqueMessage;

/// Communique stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommuniqueStats {
    /// Total messages
    pub total_messages: usize,
    /// Read messages
    pub read_messages: usize,
    /// Urgent messages
    pub urgent_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CommuniqueStats {
    /// Update from messages
    pub fn update(&mut self, messages: &[CommuniqueMessage], communique_type: CommuniqueType) {
        self.total_messages = messages.len();
        self.read_messages = messages.iter().filter(|m| m.read).count();
        if communique_type == CommuniqueType::Urgent {
            self.urgent_count = messages.len();
        }
        *self.by_type.entry(communique_type.to_string()).or_insert(0) += 1;
    }

    /// Read rate
    pub fn read_rate(&self) -> f64 {
        if self.total_messages == 0 { 0.0 } else { self.read_messages as f64 / self.total_messages as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = CommuniqueStats::default();
        let mut msg = CommuniqueMessage::new("m1", "Subject", "Body");
        msg.mark_read();
        s.update(&[msg], CommuniqueType::Official);
        assert_eq!(s.total_messages, 1);
        assert_eq!(s.read_messages, 1);
    }
}
