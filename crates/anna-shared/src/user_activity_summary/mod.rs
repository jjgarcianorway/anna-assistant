//! User Activity Summary (Phase 73)
//!
//! Tracks and displays user interaction patterns, usage statistics,
//! and activity trends over time.

mod activity_record;
mod detection;
mod formatting;
mod summary;
mod time_types;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use activity_record::ActivityRecord;
pub use detection::{detect_topic, is_activity_query};
pub use formatting::{
    activity_insight, format_activity_summary, format_activity_summary_compact,
    format_activity_summary_oneline,
};
pub use summary::UserActivitySummary;
pub use time_types::{DayOfWeek, TimeOfDay};
