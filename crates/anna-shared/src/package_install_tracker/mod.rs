//! Package Installation Tracker - Phase 80
//!
//! Tracks packages installed by Anna vs user-installed packages.
//! VISION.md mentions tracking what Anna installed vs user installed.

mod formatting;
mod tracker;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types and functions to preserve the original API
pub use formatting::{
    format_package_tracker, format_package_tracker_compact, format_package_tracker_oneline,
};
pub use tracker::PackageTracker;
pub use types::{InstalledBy, PackageManager, PackageRecord};
pub use utils::{is_package_tracker_query, package_fun_fact};
