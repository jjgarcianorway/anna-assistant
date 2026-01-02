//! Dependency Tracker - Phase 93
//!
//! Tracks software dependencies Anna manages.
//! VISION.md: Know what packages depend on what for safe removals.

mod formatting;
mod tests;
mod tracker;
mod types;

// Re-export all public types and functions
pub use formatting::{
    dependency_fun_fact, format_dependency_tracker, format_dependency_tracker_compact,
    format_dependency_tracker_oneline, is_dependency_query,
};
pub use tracker::DependencyTracker;
pub use types::{DependencyRecord, DependencyStatus, DependencyType};
