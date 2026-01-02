// v0.0.739: Settings Union (Phase 315)
// Political union for settings integration - Stats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::UnionType;
use super::provision::UnionProvision;

/// Union stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnionStats {
    /// Total provisions
    pub total_provisions: usize,
    /// Binding provisions
    pub binding: usize,
    /// Integrated count
    pub integrated_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl UnionStats {
    /// Update from provisions
    pub fn update(&mut self, provisions: &[UnionProvision], union_type: UnionType) {
        self.total_provisions = provisions.len();
        self.binding = provisions.iter().filter(|p| p.binding).count();
        *self.by_type.entry(union_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_provisions == 0 { 0.0 } else { self.binding as f64 / self.total_provisions as f64 * 100.0 }
    }
}
