//! Staff performance statistics for Service Desk Theatre.
//!
//! v0.0.107: Tracks per-staff metrics like tickets handled, success rates.
//! v0.0.315: XP penalties for poor performance, bonus/penalty in record_ticket.
//!
//! Storage: /etc/anna/staff_stats.json (system-wide)

mod levels;
mod metrics;
mod stats;

#[cfg(test)]
mod tests;

// Re-export public API to preserve compatibility
pub use levels::{level_title, xp_to_level};
pub use metrics::StaffMetrics;
pub use stats::{FeedbackResult, StaffStats};
