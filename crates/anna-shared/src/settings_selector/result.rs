// v0.0.673: Settings Selector Result (Phase 249)
// Selection result for settings selector

use serde::{Deserialize, Serialize};

/// Selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionResult {
    /// Selected entries
    pub entries: Vec<(String, String)>,
    /// Total selected
    pub total_selected: usize,
    /// Total scanned
    pub total_scanned: usize,
    /// Success
    pub success: bool,
}

impl SelectionResult {
    /// Create success result
    pub fn success(entries: Vec<(String, String)>, scanned: usize) -> Self {
        let total_selected = entries.len();
        Self {
            entries,
            total_selected,
            total_scanned: scanned,
            success: true,
        }
    }

    /// Has selections
    pub fn has_selections(&self) -> bool {
        !self.entries.is_empty()
    }

    /// Selection rate
    pub fn selection_rate(&self) -> f64 {
        if self.total_scanned == 0 {
            0.0
        } else {
            self.total_selected as f64 / self.total_scanned as f64
        }
    }
}

impl Default for SelectionResult {
    fn default() -> Self {
        Self::success(Vec::new(), 0)
    }
}
