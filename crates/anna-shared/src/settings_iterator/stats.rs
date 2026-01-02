// v0.0.681: Iterator Statistics (Phase 257)
// Statistics tracking for iterator usage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::result::IterationResult;

/// Iterator stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IteratorStats {
    /// Total iterations
    pub total_iterations: usize,
    /// Total items iterated
    pub total_items: usize,
    /// Total filtered
    pub total_filtered: usize,
    /// By order
    pub by_order: HashMap<String, usize>,
}

impl IteratorStats {
    /// Record iteration
    pub fn record(&mut self, result: &IterationResult) {
        self.total_iterations += 1;
        self.total_items += result.total_count;
        self.total_filtered += result.filtered_count;
        *self.by_order.entry(result.order.to_string()).or_insert(0) += 1;
    }

    /// Average items per iteration
    pub fn average_items(&self) -> f64 {
        if self.total_iterations == 0 {
            0.0
        } else {
            self.total_items as f64 / self.total_iterations as f64
        }
    }
}
