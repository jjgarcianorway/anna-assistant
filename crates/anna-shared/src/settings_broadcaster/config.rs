// v0.0.635: Settings Broadcaster Config (Phase 211)
// Configuration for the settings broadcaster

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::{BroadcastChannel, BroadcastMode};

/// Broadcaster config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcasterConfig {
    /// Channel
    pub channel: BroadcastChannel,
    /// Mode
    pub mode: BroadcastMode,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Active
    pub active: bool,
    /// Max listeners
    pub max_listeners: usize,
}

impl BroadcasterConfig {
    /// Create new config
    pub fn new(channel: BroadcastChannel) -> Self {
        Self {
            channel,
            mode: BroadcastMode::Sync,
            category: None,
            active: true,
            max_listeners: 100,
        }
    }

    /// Set mode
    pub fn mode(mut self, mode: BroadcastMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set max listeners
    pub fn max_listeners(mut self, max: usize) -> Self {
        self.max_listeners = max;
        self
    }
}

impl Default for BroadcasterConfig {
    fn default() -> Self {
        Self::new(BroadcastChannel::Default)
    }
}
