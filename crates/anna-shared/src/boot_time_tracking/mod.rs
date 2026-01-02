//! Boot Time Tracking - Phase 76
//!
//! Tracks system boot times and analyzes trends for greeting messages.
//! VISION.md mentions: "Your boot time has increased by 2 seconds..."

mod types;
mod tracker;
mod parsing;
mod formatting;
mod query;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{BootRecord, BootTimeTracker, BootTrend, SlowService};
pub use parsing::parse_systemd_analyze;
pub use formatting::{
    boot_time_greeting, format_boot_stats, format_boot_stats_compact, format_boot_stats_oneline,
};
pub use query::{boot_time_fun_fact, is_boot_time_query};
