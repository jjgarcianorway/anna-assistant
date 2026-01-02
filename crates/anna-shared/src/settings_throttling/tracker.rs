// v0.0.589: Settings Throttling - Request Tracker (Phase 165)
// Request tracking and throttle keys

use crate::unified_settings::SettingsCategory;
use super::types::ThrottleAction;

/// Request tracking entry
#[derive(Debug, Clone, Default)]
pub(super) struct RequestTracker {
    /// Request timestamps
    pub timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    /// Current burst used
    pub burst_used: u32,
}

impl RequestTracker {
    /// Clean old timestamps
    pub fn clean(&mut self, window_secs: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(window_secs as i64);
        self.timestamps.retain(|t| *t > cutoff);
    }

    /// Add request
    pub fn add(&mut self) {
        self.timestamps.push(chrono::Utc::now());
    }

    /// Current count
    pub fn count(&self) -> usize {
        self.timestamps.len()
    }
}

/// Throttle key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ThrottleKey {
    pub action: ThrottleAction,
    pub category: Option<SettingsCategory>,
}
