//! Team Performance Display (Phase 71)
//!
//! Provides display functions for showing team performance metrics,
//! comparing teams, and tracking improvement over time.

mod team_id;
mod team_metrics;
mod team_performance;
mod formatters;

// Re-export all public items
pub use team_id::TeamId;
pub use team_metrics::{TeamMetrics, team_grade};
pub use team_performance::TeamPerformance;
pub use formatters::{
    format_duration_ms,
    format_team_performance,
    format_team_performance_compact,
    format_team_performance_oneline,
    team_performance_fun_fact,
    is_team_performance_query,
};
