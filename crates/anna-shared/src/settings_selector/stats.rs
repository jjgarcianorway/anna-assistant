// v0.0.673: Settings Selector Stats (Phase 249)
// Statistics for settings selector

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::SelectorType;
use super::result::SelectionResult;

/// Selector stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectorStats {
    /// Total selections
    pub total_selections: usize,
    /// Total selected
    pub total_selected: usize,
    /// Total scanned
    pub total_scanned: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl SelectorStats {
    /// Record selection
    pub fn record(&mut self, result: &SelectionResult, selector_type: SelectorType) {
        self.total_selections += 1;
        self.total_selected += result.total_selected;
        self.total_scanned += result.total_scanned;
        *self.by_type.entry(selector_type.to_string()).or_insert(0) += 1;
    }

    /// Average selection rate
    pub fn average_selection_rate(&self) -> f64 {
        if self.total_scanned == 0 {
            0.0
        } else {
            self.total_selected as f64 / self.total_scanned as f64
        }
    }
}
