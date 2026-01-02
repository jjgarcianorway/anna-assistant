// v0.0.589: Settings Throttling - Throttler (Phase 165)
// Main throttler implementation

use std::collections::HashMap;
use crate::unified_settings::SettingsCategory;
use super::rate_limit::RateLimit;
use super::tracker::{RequestTracker, ThrottleKey};
use super::types::{ThrottleAction, ThrottleResult, ThrottleStats};

/// Settings throttler
#[derive(Debug, Clone, Default)]
pub struct SettingsThrottler {
    /// Rate limits by action
    limits: HashMap<ThrottleAction, RateLimit>,
    /// Request trackers
    trackers: HashMap<ThrottleKey, RequestTracker>,
    /// Global enabled flag
    enabled: bool,
    /// Blocked actions
    blocked: Vec<ThrottleAction>,
}

impl SettingsThrottler {
    /// Create new throttler
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Set rate limit for action
    pub fn set_limit(&mut self, action: ThrottleAction, limit: RateLimit) {
        self.limits.insert(action, limit);
    }

    /// Get rate limit for action
    pub fn get_limit(&self, action: ThrottleAction) -> Option<&RateLimit> {
        self.limits.get(&action)
    }

    /// Block an action
    pub fn block(&mut self, action: ThrottleAction) {
        if !self.blocked.contains(&action) {
            self.blocked.push(action);
        }
    }

    /// Unblock an action
    pub fn unblock(&mut self, action: ThrottleAction) {
        self.blocked.retain(|a| *a != action);
    }

    /// Check if action is blocked
    pub fn is_blocked(&self, action: ThrottleAction) -> bool {
        self.blocked.contains(&action)
    }

    /// Check and record request
    pub fn check(&mut self, action: ThrottleAction, category: Option<SettingsCategory>) -> ThrottleResult {
        if !self.enabled {
            return ThrottleResult::Allowed;
        }

        if self.is_blocked(action) {
            return ThrottleResult::Blocked;
        }

        let limit = match self.limits.get(&action).or_else(|| self.limits.get(&ThrottleAction::Any)) {
            Some(l) => l.clone(),
            None => return ThrottleResult::Allowed,
        };

        let key = ThrottleKey { action, category };
        let tracker = self.trackers.entry(key).or_default();

        tracker.clean(limit.window_secs);

        let current = tracker.count() as u32;
        let max_with_burst = limit.max_requests + limit.burst;

        if current >= max_with_burst {
            return ThrottleResult::Limited;
        }

        tracker.add();

        if current >= limit.max_requests {
            tracker.burst_used += 1;
        }

        ThrottleResult::Allowed
    }

    /// Check without recording
    pub fn would_limit(&self, action: ThrottleAction, category: Option<SettingsCategory>) -> bool {
        if !self.enabled {
            return false;
        }

        if self.is_blocked(action) {
            return true;
        }

        let limit = match self.limits.get(&action).or_else(|| self.limits.get(&ThrottleAction::Any)) {
            Some(l) => l,
            None => return false,
        };

        let key = ThrottleKey { action, category };
        if let Some(tracker) = self.trackers.get(&key) {
            let cutoff = chrono::Utc::now() - chrono::Duration::seconds(limit.window_secs as i64);
            let current = tracker.timestamps.iter().filter(|t| **t > cutoff).count() as u32;
            current >= limit.max_requests + limit.burst
        } else {
            false
        }
    }

    /// Enable throttling
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable throttling
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Reset all trackers
    pub fn reset(&mut self) {
        self.trackers.clear();
    }

    /// Get stats for action
    pub fn stats(&self, action: ThrottleAction) -> ThrottleStats {
        let mut total = 0;
        for (key, tracker) in &self.trackers {
            if key.action == action {
                total += tracker.count();
            }
        }
        ThrottleStats {
            action,
            requests: total,
            limit: self.limits.get(&action).cloned(),
        }
    }
}
