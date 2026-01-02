// v0.0.635: Settings Broadcaster Stats (Phase 211)
// Statistics tracking for the broadcaster

use serde::{Deserialize, Serialize};

/// Broadcaster stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BroadcasterStats {
    /// Total broadcasts
    pub total_broadcasts: usize,
    /// Delivered count
    pub delivered: usize,
    /// Dropped count
    pub dropped: usize,
    /// Active listeners
    pub active_listeners: usize,
}

impl BroadcasterStats {
    /// Record broadcast
    pub fn record_broadcast(&mut self, listener_count: usize) {
        self.total_broadcasts += 1;
        self.delivered += listener_count;
    }

    /// Record drop
    pub fn record_drop(&mut self) {
        self.total_broadcasts += 1;
        self.dropped += 1;
    }

    /// Delivery efficiency
    pub fn delivery_efficiency(&self) -> f64 {
        if self.total_broadcasts == 0 {
            1.0
        } else {
            self.delivered as f64 / (self.delivered + self.dropped) as f64
        }
    }
}
