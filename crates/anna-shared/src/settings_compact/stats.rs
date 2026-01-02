// v0.0.729: Settings Compact (Phase 305)
// Compact statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::CompactType;
use super::term::CompactTerm;

/// Compact stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompactStats {
    /// Total terms
    pub total_terms: usize,
    /// Binding terms
    pub binding: usize,
    /// Enacted count
    pub enacted_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CompactStats {
    /// Update from terms
    pub fn update(&mut self, terms: &[CompactTerm], compact_type: CompactType) {
        self.total_terms = terms.len();
        self.binding = terms.iter().filter(|t| t.binding).count();
        *self.by_type.entry(compact_type.to_string()).or_insert(0) += 1;
    }

    /// Binding rate
    pub fn binding_rate(&self) -> f64 {
        if self.total_terms == 0 { 0.0 } else { self.binding as f64 / self.total_terms as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = CompactStats::default();
        let term = CompactTerm::new("t1", "Title", "Content");
        s.update(&[term], CompactType::Interstate);
        assert_eq!(s.total_terms, 1);
        assert_eq!(s.binding, 1);
    }
}
