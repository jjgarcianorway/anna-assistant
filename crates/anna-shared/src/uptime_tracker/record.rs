//! Uptime record tracking.

use serde::{Deserialize, Serialize};

/// Uptime record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UptimeRecord {
    /// Session start timestamp
    pub start_time: u64,
    /// Session end timestamp (None if still running)
    pub end_time: Option<u64>,
    /// Duration in seconds
    pub duration_secs: u64,
    /// Was this a clean shutdown
    pub clean_shutdown: bool,
}

impl UptimeRecord {
    /// Create new session record
    pub fn start(timestamp: u64) -> Self {
        Self {
            start_time: timestamp,
            end_time: None,
            duration_secs: 0,
            clean_shutdown: false,
        }
    }

    /// End the session
    pub fn end(&mut self, timestamp: u64, clean: bool) {
        self.end_time = Some(timestamp);
        self.duration_secs = timestamp.saturating_sub(self.start_time);
        self.clean_shutdown = clean;
    }

    /// Check if session is active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get duration in hours
    pub fn duration_hours(&self) -> f64 {
        self.duration_secs as f64 / 3600.0
    }
}
