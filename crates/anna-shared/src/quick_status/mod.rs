//! Quick Status Summary (v0.0.484).
//!
//! Provides at-a-glance system status summaries.
//! Designed for quick checks without full status display.

mod formatters;
mod helpers;
mod query;
mod tests;
mod types;

// Re-export public types
pub use types::{HealthLevel, QuickStatus, StatusItem};

// Re-export formatters
pub use formatters::{
    format_quick_status_compact, format_quick_status_full, format_quick_status_oneline,
};

// Re-export helpers
pub use helpers::{cpu_status, disk_status, memory_status, service_status};

// Re-export query detection
pub use query::is_quick_status_query;
