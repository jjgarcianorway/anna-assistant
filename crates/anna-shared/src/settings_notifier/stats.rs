// v0.0.639: Settings Notifier - Stats (Phase 215)
// Notifier statistics tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::priority::NotifyPriority;

/// Notifier stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotifierStats {
    /// Total sent
    pub total_sent: usize,
    /// By priority
    pub by_priority: HashMap<String, usize>,
    /// Suppressed
    pub suppressed: usize,
}

impl NotifierStats {
    /// Record sent
    pub fn record_sent(&mut self, priority: NotifyPriority) {
        self.total_sent += 1;
        *self.by_priority.entry(priority.to_string()).or_insert(0) += 1;
    }

    /// Record suppressed
    pub fn record_suppressed(&mut self) {
        self.suppressed += 1;
    }

    /// Suppression rate
    pub fn suppression_rate(&self) -> f64 {
        let total = self.total_sent + self.suppressed;
        if total == 0 {
            0.0
        } else {
            self.suppressed as f64 / total as f64
        }
    }
}
