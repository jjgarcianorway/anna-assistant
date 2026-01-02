// v0.0.755: Settings Block (Phase 331)
// Block statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::BlockType;
use super::plat::BlockPlat;

/// Block stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockStats {
    /// Total plats
    pub total_plats: usize,
    /// Recorded plats
    pub recorded: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BlockStats {
    /// Update from plats
    pub fn update(&mut self, plats: &[BlockPlat], block_type: BlockType) {
        self.total_plats = plats.len();
        self.recorded = plats.iter().filter(|p| p.recorded).count();
        *self.by_type.entry(block_type.to_string()).or_insert(0) += 1;
    }

    /// Recorded rate
    pub fn recorded_rate(&self) -> f64 {
        if self.total_plats == 0 { 0.0 } else { self.recorded as f64 / self.total_plats as f64 * 100.0 }
    }
}
