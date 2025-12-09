//! Event log store for stats/RPG system (v0.0.190).
//!
//! Append-only JSONL store for request events with rotation.
//! v0.0.86: Added streaks, active days, lucky team tracking.
//! v0.0.190: Modularized into domain-focused submodules.

mod aggregation;
mod store;
mod tests;
mod types;

// Re-export main types and functions
pub use aggregation::{level_title, xp_to_level, AggregatedEvents};
pub use store::EventLog;
pub use types::EventRecord;
