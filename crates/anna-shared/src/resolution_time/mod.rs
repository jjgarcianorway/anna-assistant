//! Resolution Time Tracking (v0.0.487).
//!
//! Tracks how long it takes to resolve user requests.
//! Provides statistics on fastest and slowest resolutions.

pub mod formatting;
pub mod queries;
pub mod record;
pub mod stats;
pub mod summary;

#[cfg(test)]
mod tests;

// Re-export main types
pub use formatting::{format_duration_ms, format_resolution_times, format_resolution_times_compact};
pub use queries::{is_resolution_time_query, resolution_time_fun_fact};
pub use record::ResolutionRecord;
pub use stats::{CategoryStats, ResolutionTimeTracker};
pub use summary::ResolutionTimeSummary;
