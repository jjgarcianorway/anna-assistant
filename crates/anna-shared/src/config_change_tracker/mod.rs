//! Config Change Tracker - Phase 82
//!
//! Tracks configuration file changes made by Anna.
//! VISION.md mentions Anna editing config files and keeping track of changes.

mod types;
mod utils;
mod formatters;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use types::{
    ChangeType,
    ConfigCategory,
    ConfigChangeRecord,
    ConfigChangeTracker,
};

// Re-export utility functions
pub use utils::{
    config_fun_fact,
    detect_category,
    is_config_tracker_query,
};

// Re-export formatting functions
pub use formatters::{
    format_config_tracker,
    format_config_tracker_compact,
    format_config_tracker_oneline,
};
