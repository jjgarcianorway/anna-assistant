// v0.0.636: Listener Config (Phase 212)
// Configuration for settings listeners

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;
use super::types::ListenerType;

/// Listener config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    /// Listener type
    pub listener_type: ListenerType,
    /// Category filter
    pub category: Option<SettingsCategory>,
    /// Key pattern
    pub key_pattern: Option<String>,
    /// Auto start
    pub auto_start: bool,
    /// Buffer size
    pub buffer_size: usize,
}

impl ListenerConfig {
    /// Create new config
    pub fn new(listener_type: ListenerType) -> Self {
        Self {
            listener_type,
            category: None,
            key_pattern: None,
            auto_start: true,
            buffer_size: 50,
        }
    }

    /// Set category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set key pattern
    pub fn key_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.key_pattern = Some(pattern.into());
        self
    }

    /// Set auto start
    pub fn auto_start(mut self, auto: bool) -> Self {
        self.auto_start = auto;
        self
    }

    /// Set buffer size
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
}
