//! Expert Ticket Statistics (v0.0.489).
//!
//! Tracks tickets closed per expert (junior and senior).
//! Provides detailed statistics on expert performance.

mod level;
mod expert;
mod statistics;
mod tracker;
mod format;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public items
pub use level::ExpertLevel;
pub use expert::Expert;
pub use statistics::ExpertStatistics;
pub use tracker::{ExpertStatsTracker, ExpertStatsSummary};
pub use format::{format_expert_stats, format_expert_stats_compact};
pub use utils::{expert_stats_fun_fact, is_expert_stats_query};
