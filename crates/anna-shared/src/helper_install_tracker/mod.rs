// v0.0.532: Helper Install Tracker Module (Phase 108)
// Tracks helper tools installed by Anna vs user per VISION.md

pub mod types;
pub mod record;
pub mod tracker;
pub mod format;
pub mod utils;

// Re-export all public types and functions to preserve API
pub use types::{HelperCategory, HelperInstaller, HelperStatus};
pub use record::HelperRecord;
pub use tracker::HelperInstallTracker;
pub use format::{format_helper, format_helper_compact, format_helper_oneline, format_tracker_summary};
pub use utils::{is_helper_query, helper_fun_fact};
