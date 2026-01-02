// v0.0.635: Settings Broadcaster Listener (Phase 211)
// Listener information and management

use serde::{Deserialize, Serialize};

use super::types::BroadcastChannel;

/// Listener info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerInfo {
    /// ID
    pub id: String,
    /// Name
    pub name: String,
    /// Channel
    pub channel: BroadcastChannel,
    /// Registered timestamp
    pub registered_at: u64,
    /// Message count
    pub message_count: usize,
}

impl ListenerInfo {
    /// Create new listener
    pub fn new(id: impl Into<String>, name: impl Into<String>, channel: BroadcastChannel) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            channel,
            registered_at: 0,
            message_count: 0,
        }
    }

    /// Set registered timestamp
    pub fn registered_at(mut self, ts: u64) -> Self {
        self.registered_at = ts;
        self
    }

    /// Record message
    pub fn record_message(&mut self) {
        self.message_count += 1;
    }
}
