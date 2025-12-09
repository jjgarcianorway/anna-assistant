//! Health delta tracking for "how is my computer" queries (v0.0.225).
//!
//! v0.0.41: Fast (<1s) system health summaries from cached snapshots.
//! v0.0.42: IT-department style output, only warnings/deltas by default.
//! v0.0.225: Modularized into domain-focused submodules.

mod helpers;
mod history;
mod summary;
mod types;

// Re-export for backwards compatibility
pub use helpers::{format_disk_summary, generate_summary};
pub use history::SnapshotHistory;
pub use summary::HealthSummary;
pub use types::HealthDelta;

// Tests are in tests/health_delta_tests.rs
