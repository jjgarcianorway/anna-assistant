// v0.0.589: Settings Throttling - Rate Limit (Phase 165)
// Rate limit configuration

use serde::{Deserialize, Serialize};

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Max requests
    pub max_requests: u32,
    /// Time window in seconds
    pub window_secs: u64,
    /// Burst size (extra requests allowed)
    pub burst: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            burst: 10,
        }
    }
}

impl RateLimit {
    /// Create new rate limit
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            burst: 0,
        }
    }

    /// Set burst size
    pub fn burst(mut self, burst: u32) -> Self {
        self.burst = burst;
        self
    }
}
