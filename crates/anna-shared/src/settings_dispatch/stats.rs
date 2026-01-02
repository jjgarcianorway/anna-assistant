// v0.0.714: Settings Dispatch Stats (Phase 290)
// Statistics tracking for dispatch operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::{DispatchType, DispatchStatus};
use super::item::DispatchItem;

/// Dispatch stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispatchStats {
    /// Total dispatches
    pub total_dispatches: usize,
    /// Completed dispatches
    pub completed: usize,
    /// Failed dispatches
    pub failed: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DispatchStats {
    /// Update from items
    pub fn update(&mut self, items: &[DispatchItem], dispatch_type: DispatchType) {
        self.total_dispatches = items.len();
        self.completed = items.iter().filter(|i| i.status == DispatchStatus::Completed).count();
        self.failed = items.iter().filter(|i| i.status == DispatchStatus::Failed).count();
        *self.by_type.entry(dispatch_type.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_dispatches == 0 { 0.0 } else { self.completed as f64 / self.total_dispatches as f64 * 100.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_update() {
        let mut s = DispatchStats::default();
        let mut item = DispatchItem::new("i1", "target", "payload");
        item.complete();
        s.update(&[item], DispatchType::Immediate);
        assert_eq!(s.total_dispatches, 1);
        assert_eq!(s.completed, 1);
    }
}
