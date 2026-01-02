// v0.0.681: Iterator Configuration (Phase 257)
// Configuration for settings iteration

use serde::{Deserialize, Serialize};
use super::types::{IterationOrder, IterationFilter};

/// Iterator config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IteratorConfig {
    /// Iteration order
    pub order: IterationOrder,
    /// Iteration filter
    pub filter: IterationFilter,
    /// Batch size
    pub batch_size: usize,
    /// Skip count
    pub skip: usize,
    /// Take count (0 = all)
    pub take: usize,
}

impl IteratorConfig {
    /// Create new config
    pub fn new(order: IterationOrder) -> Self {
        Self {
            order,
            filter: IterationFilter::None,
            batch_size: 100,
            skip: 0,
            take: 0,
        }
    }

    /// Set filter
    pub fn filter(mut self, filter: IterationFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Set skip
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = skip;
        self
    }

    /// Set take
    pub fn take(mut self, take: usize) -> Self {
        self.take = take;
        self
    }
}

impl Default for IteratorConfig {
    fn default() -> Self {
        Self::new(IterationOrder::Natural)
    }
}
