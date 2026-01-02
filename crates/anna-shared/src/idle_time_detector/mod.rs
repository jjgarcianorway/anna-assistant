//! Idle Time Detector - Phase 85
//!
//! Detects when machine is idle for background research tasks.
//! VISION.md: "Investigate when machine is idle"

mod formatters;
mod tests;
mod tracker;
mod types;
mod utils;

// Re-export types
pub use types::{ActivityLevel, IdleConfig, IdlePeriod, IdleState};

// Re-export tracker
pub use tracker::IdleTimeTracker;

// Re-export formatters
pub use formatters::{format_idle_tracker, format_idle_tracker_compact, format_idle_tracker_oneline};

// Re-export utils
pub use utils::{idle_fun_fact, is_idle_query};
