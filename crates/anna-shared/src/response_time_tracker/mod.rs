// v0.0.538: Response Time Tracker Module (Phase 114)
// Tracks response times for "shortest/longest reply" per VISION.md

mod formatting;
mod record;
mod tracker;
mod types;

#[cfg(test)]
mod tests;

// Re-export public API to preserve compatibility
pub use formatting::{
    format_response_time, format_tracker_summary, is_response_time_query, response_time_fun_fact,
};
pub use record::ResponseTimeRecord;
pub use tracker::ResponseTimeTracker;
pub use types::{ComplexityLevel, ResponseType, TimeDistribution};
