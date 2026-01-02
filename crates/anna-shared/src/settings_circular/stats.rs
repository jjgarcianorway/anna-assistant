// v0.0.717: Settings Circular - Stats (Phase 293)
// Circular statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::CircularType;
use super::notice::CircularNotice;

/// Circular stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircularStats {
    /// Total circulars
    pub total_circulars: usize,
    /// Active circulars
    pub active: usize,
    /// Policy circulars
    pub policy_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CircularStats {
    /// Update from notices
    pub fn update(&mut self, notices: &[CircularNotice], circular_type: CircularType) {
        self.total_circulars = notices.len();
        self.active = notices.iter().filter(|n| n.active).count();
        if circular_type == CircularType::Policy {
            self.policy_count = notices.len();
        }
        *self.by_type.entry(circular_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_circulars == 0 { 0.0 } else { self.active as f64 / self.total_circulars as f64 * 100.0 }
    }
}
