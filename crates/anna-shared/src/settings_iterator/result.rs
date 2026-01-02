// v0.0.681: Iteration Result (Phase 257)
// Result of settings iteration

use serde::{Deserialize, Serialize};
use super::item::IterationItem;
use super::types::IterationOrder;

/// Iteration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    /// Items
    pub items: Vec<IterationItem>,
    /// Total count
    pub total_count: usize,
    /// Filtered count
    pub filtered_count: usize,
    /// Order used
    pub order: IterationOrder,
}

impl IterationResult {
    /// Create new result
    pub fn new(items: Vec<IterationItem>, total: usize, order: IterationOrder) -> Self {
        let filtered_count = items.len();
        Self {
            items,
            total_count: total,
            filtered_count,
            order,
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get item
    pub fn get(&self, index: usize) -> Option<&IterationItem> {
        self.items.get(index)
    }

    /// Iterate
    pub fn iter(&self) -> impl Iterator<Item = &IterationItem> {
        self.items.iter()
    }
}

impl Default for IterationResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, IterationOrder::Natural)
    }
}
