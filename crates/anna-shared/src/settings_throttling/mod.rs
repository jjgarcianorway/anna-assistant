// v0.0.589: Settings Throttling (Phase 165)
// Rate limiting for settings operations

mod types;
mod rate_limit;
mod tracker;
mod throttler;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{ThrottleAction, ThrottleResult, ThrottleStats};
pub use rate_limit::RateLimit;
pub use throttler::SettingsThrottler;
pub use utils::{format_throttle, is_throttling_query, settings_throttling_fun_fact};
