//! Job scheduler (v0.0.430).
//!
//! Manages background job queue with priority and idle-time awareness.

mod core;
mod helpers;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public types and functions
pub use core::JobScheduler;
pub use helpers::now_timestamp;
pub use types::{JobFilter, SchedulerConfig, SchedulerStats, SchedulerStatusSummary};
