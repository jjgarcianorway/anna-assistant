// v0.0.719: Settings Edict - Stats
// Statistics tracking for edicts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{EdictType, EdictStatus};
use super::proclamation::EdictProclamation;

/// Edict stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdictStats {
    /// Total edicts
    pub total_edicts: usize,
    /// Active edicts
    pub active: usize,
    /// Revoked edicts
    pub revoked: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EdictStats {
    /// Update from proclamations
    pub fn update(&mut self, proclamations: &[EdictProclamation], edict_type: EdictType) {
        self.total_edicts = proclamations.len();
        self.active = proclamations.iter().filter(|p| p.status == EdictStatus::Active).count();
        self.revoked = proclamations.iter().filter(|p| p.status == EdictStatus::Revoked).count();
        *self.by_type.entry(edict_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_edicts == 0 { 0.0 } else { self.active as f64 / self.total_edicts as f64 * 100.0 }
    }
}
