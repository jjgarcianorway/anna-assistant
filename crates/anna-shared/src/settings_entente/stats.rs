// v0.0.734: Settings Entente (Phase 310)
// Entente statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::EntenteType;
use super::understanding::EntenteUnderstanding;

/// Entente stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntenteStats {
    /// Total understandings
    pub total_understandings: usize,
    /// Tacit understandings
    pub tacit: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl EntenteStats {
    /// Update from understandings
    pub fn update(&mut self, understandings: &[EntenteUnderstanding], entente_type: EntenteType) {
        self.total_understandings = understandings.len();
        self.tacit = understandings.iter().filter(|u| u.tacit).count();
        *self.by_type.entry(entente_type.to_string()).or_insert(0) += 1;
    }

    /// Tacit rate
    pub fn tacit_rate(&self) -> f64 {
        if self.total_understandings == 0 { 0.0 } else { self.tacit as f64 / self.total_understandings as f64 * 100.0 }
    }
}
