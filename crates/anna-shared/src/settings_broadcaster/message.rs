// v0.0.635: Settings Broadcaster Message (Phase 211)
// Broadcast message types

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::BroadcastChannel;

/// Broadcast message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// Message ID
    pub id: String,
    /// Channel
    pub channel: BroadcastChannel,
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Payload
    pub payload: String,
    /// Timestamp
    pub timestamp: u64,
}

impl BroadcastMessage {
    /// Create new message
    pub fn new(
        id: impl Into<String>,
        channel: BroadcastChannel,
        category: SettingsCategory,
        key: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            channel,
            category,
            key: key.into(),
            payload: payload.into(),
            timestamp: 0,
        }
    }

    /// Set timestamp
    pub fn timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }
}
