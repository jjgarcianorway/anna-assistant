// v0.0.716: Settings Missive Stats (Phase 292)
// Statistics tracking for missive system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::MissiveType;
use super::letter::MissiveLetter;

/// Missive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissiveStats {
    /// Total missives
    pub total_missives: usize,
    /// Delivered missives
    pub delivered: usize,
    /// Priority missives
    pub priority_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MissiveStats {
    /// Update from letters
    pub fn update(&mut self, letters: &[MissiveLetter], missive_type: MissiveType) {
        self.total_missives = letters.len();
        self.delivered = letters.iter().filter(|l| l.delivered).count();
        *self.by_type.entry(missive_type.to_string()).or_insert(0) += 1;
    }

    /// Delivery rate
    pub fn delivery_rate(&self) -> f64 {
        if self.total_missives == 0 { 0.0 } else { self.delivered as f64 / self.total_missives as f64 * 100.0 }
    }
}
