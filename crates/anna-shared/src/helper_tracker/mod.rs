//! Helper Tracker - Phase 83
//!
//! Tracks helpers (tools) installed by Anna vs user.
//! VISION.md: "Track what helpers she installed vs user installed"
//! Anna-installed helpers can be removed on uninstall, user-installed are preserved.

pub mod formatting;
pub mod tracker;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

// Re-export main types and functions for backward compatibility
pub use formatting::{
    format_helper_tracker, format_helper_tracker_compact, format_helper_tracker_oneline,
    helper_fun_fact,
};
pub use tracker::HelperTracker;
pub use types::{HelperPurpose, HelperRecord, InstallerSource};
pub use utils::{detect_purpose, is_helper_query};
