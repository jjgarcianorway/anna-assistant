// v0.0.589: Settings Throttling - Utilities (Phase 165)
// Helper functions for throttling

use super::throttler::SettingsThrottler;
use super::types::ThrottleAction;

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
