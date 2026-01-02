//! Uptime Tracking (v0.0.492).
//!
//! Tracks Anna's uptime and availability.
//! Provides installation date tracking and session statistics.

pub mod record;
pub mod tracker;
pub mod formatting;

#[cfg(test)]
mod tests;

// Re-export public API
pub use record::UptimeRecord;
pub use tracker::{UptimeTracker, UptimeSummary};
pub use formatting::{
    format_duration_secs,
    format_uptime,
    format_uptime_compact,
    format_uptime_oneline,
    uptime_fun_fact,
    is_uptime_query,
};
