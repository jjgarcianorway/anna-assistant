// v0.0.785: Settings Retreat - Stats (Phase 361)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::RetreatType;
use super::visitor::RetreatVisitor;

/// Retreat stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetreatStats {
    /// Total visitors
    pub total_visitors: usize,
    /// Relaxed visitors
    pub relaxed: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl RetreatStats {
    /// Update from visitors
    pub fn update(&mut self, visitors: &[RetreatVisitor], retreat_type: RetreatType) {
        self.total_visitors = visitors.len();
        self.relaxed = visitors.iter().filter(|v| v.relaxed).count();
        *self.by_type.entry(retreat_type.to_string()).or_insert(0) += 1;
    }

    /// Relaxation rate
    pub fn relaxation_rate(&self) -> f64 {
        if self.total_visitors == 0 { 0.0 } else { self.relaxed as f64 / self.total_visitors as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = RetreatStats::default();
        let visitor = RetreatVisitor::new("v1", "Title", "Content");
        s.update(&[visitor], RetreatType::Peaceful);
        assert_eq!(s.total_visitors, 1);
        assert_eq!(s.relaxed, 1);
    }
}
