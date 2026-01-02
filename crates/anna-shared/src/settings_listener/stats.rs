// v0.0.636: Listener Stats (Phase 212)
// Statistics tracking for settings listeners

use serde::{Deserialize, Serialize};

/// Listener stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListenerStats {
    /// Total received
    pub total_received: usize,
    /// Processed count
    pub processed: usize,
    /// Filtered count
    pub filtered: usize,
    /// Dropped count
    pub dropped: usize,
}

impl ListenerStats {
    /// Record received
    pub fn record_received(&mut self) {
        self.total_received += 1;
    }

    /// Record processed
    pub fn record_processed(&mut self) {
        self.processed += 1;
    }

    /// Record filtered
    pub fn record_filtered(&mut self) {
        self.total_received += 1;
        self.filtered += 1;
    }

    /// Record dropped
    pub fn record_dropped(&mut self) {
        self.total_received += 1;
        self.dropped += 1;
    }

    /// Processing rate
    pub fn processing_rate(&self) -> f64 {
        if self.total_received == 0 {
            1.0
        } else {
            self.processed as f64 / self.total_received as f64
        }
    }
}
