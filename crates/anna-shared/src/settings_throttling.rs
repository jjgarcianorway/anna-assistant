// v0.0.589: Settings Throttling (Phase 165)
// Rate limiting for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Throttle action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThrottleAction {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Export operation
    Export,
    /// Import operation
    Import,
    /// Sync operation
    Sync,
    /// Any operation
    Any,
}

impl std::fmt::Display for ThrottleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Export => write!(f, "export"),
            Self::Import => write!(f, "import"),
            Self::Sync => write!(f, "sync"),
            Self::Any => write!(f, "any"),
        }
    }
}

/// Throttle result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottleResult {
    /// Allowed
    Allowed,
    /// Rate limited
    Limited,
    /// Blocked
    Blocked,
}

impl std::fmt::Display for ThrottleResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allowed => write!(f, "allowed"),
            Self::Limited => write!(f, "limited"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

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

/// Request tracking entry
#[derive(Debug, Clone, Default)]
struct RequestTracker {
    /// Request timestamps
    timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    /// Current burst used
    burst_used: u32,
}

impl RequestTracker {
    /// Clean old timestamps
    fn clean(&mut self, window_secs: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(window_secs as i64);
        self.timestamps.retain(|t| *t > cutoff);
    }

    /// Add request
    fn add(&mut self) {
        self.timestamps.push(chrono::Utc::now());
    }

    /// Current count
    fn count(&self) -> usize {
        self.timestamps.len()
    }
}

/// Throttle key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ThrottleKey {
    action: ThrottleAction,
    category: Option<SettingsCategory>,
}

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

/// Throttle statistics
#[derive(Debug, Clone)]
pub struct ThrottleStats {
    /// Action
    pub action: ThrottleAction,
    /// Request count
    pub requests: usize,
    /// Limit configuration
    pub limit: Option<RateLimit>,
}

/// Format throttle stats
pub fn format_throttle(throttler: &SettingsThrottler) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Throttling ===\n\n");
    output.push_str(&format!("Enabled: {}\n\n", throttler.is_enabled()));

    for action in [ThrottleAction::Read, ThrottleAction::Write, ThrottleAction::Sync] {
        let stats = throttler.stats(action);
        let limit_str = stats.limit
            .map(|l| format!("{}/{} per {}s", stats.requests, l.max_requests, l.window_secs))
            .unwrap_or_else(|| "unlimited".to_string());
        output.push_str(&format!("{}: {}\n", action, limit_str));
    }

    output
}

/// Check if query is about throttling
pub fn is_throttling_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("throttl")
        || lower.contains("rate limit")
        || lower.contains("limit")
}

/// Fun fact about throttling
pub fn settings_throttling_fun_fact() -> &'static str {
    "Anna can throttle settings changes to prevent accidental rapid modifications!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_throttle_action_display() {
        assert_eq!(format!("{}", ThrottleAction::Read), "read");
        assert_eq!(format!("{}", ThrottleAction::Write), "write");
    }

    #[test]
    fn test_throttle_result_display() {
        assert_eq!(format!("{}", ThrottleResult::Allowed), "allowed");
        assert_eq!(format!("{}", ThrottleResult::Limited), "limited");
    }

    #[test]
    fn test_rate_limit_default() {
        let limit = RateLimit::default();
        assert_eq!(limit.max_requests, 100);
        assert_eq!(limit.window_secs, 60);
    }

    #[test]
    fn test_rate_limit_new() {
        let limit = RateLimit::new(50, 30).burst(5);
        assert_eq!(limit.max_requests, 50);
        assert_eq!(limit.burst, 5);
    }

    #[test]
    fn test_throttler_new() {
        let throttler = SettingsThrottler::new();
        assert!(throttler.is_enabled());
    }

    #[test]
    fn test_throttler_set_limit() {
        let mut throttler = SettingsThrottler::new();
        throttler.set_limit(ThrottleAction::Write, RateLimit::new(10, 60));
        assert!(throttler.get_limit(ThrottleAction::Write).is_some());
    }

    #[test]
    fn test_throttler_block() {
        let mut throttler = SettingsThrottler::new();
        throttler.block(ThrottleAction::Import);
        assert!(throttler.is_blocked(ThrottleAction::Import));
        throttler.unblock(ThrottleAction::Import);
        assert!(!throttler.is_blocked(ThrottleAction::Import));
    }

    #[test]
    fn test_throttler_check_allowed() {
        let mut throttler = SettingsThrottler::new();
        let result = throttler.check(ThrottleAction::Read, None);
        assert_eq!(result, ThrottleResult::Allowed);
    }

    #[test]
    fn test_throttler_check_blocked() {
        let mut throttler = SettingsThrottler::new();
        throttler.block(ThrottleAction::Write);
        let result = throttler.check(ThrottleAction::Write, None);
        assert_eq!(result, ThrottleResult::Blocked);
    }

    #[test]
    fn test_throttler_disable() {
        let mut throttler = SettingsThrottler::new();
        throttler.disable();
        assert!(!throttler.is_enabled());
    }

    #[test]
    fn test_format_throttle() {
        let throttler = SettingsThrottler::new();
        let output = format_throttle(&throttler);
        assert!(output.contains("Throttling"));
    }

    #[test]
    fn test_is_throttling_query() {
        assert!(is_throttling_query("enable rate limiting"));
        assert!(is_throttling_query("throttle writes"));
        assert!(!is_throttling_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_throttling_fun_fact();
        assert!(fact.contains("throttle"));
    }
}
