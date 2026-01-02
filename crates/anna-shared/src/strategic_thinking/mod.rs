//! Strategic Thinking Tracker - Phase 91
//!
//! Tracks senior strategic thinking during idle time.
//! VISION.md: "Seniors can think strategically about improvements during idle time"
//! "If interrupted, Anna can resume later"

mod types;
mod task;
mod tracker;
mod formatting;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public types and functions to maintain the public API
pub use types::{ThinkingCategory, ThinkingPriority, ThinkingStatus};
pub use task::ThinkingTask;
pub use tracker::StrategicThinkingTracker;
pub use formatting::{
    format_strategic_tracker,
    format_strategic_tracker_compact,
    format_strategic_tracker_oneline,
    strategic_fun_fact,
};
pub use utils::is_strategic_query;
