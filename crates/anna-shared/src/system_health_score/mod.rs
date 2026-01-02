//! System Health Score (Phase 74)
//!
//! Provides a unified system health score combining multiple metrics
//! into a single actionable health assessment.

mod formatting;
mod metric_builders;
mod types;
mod utils;

#[cfg(test)]
mod tests;

// Re-export public API
pub use formatting::{
    format_health_score, format_health_score_compact, format_health_score_oneline, health_bar,
};
pub use metric_builders::{
    cpu_health, daemon_health, disk_health, memory_health, network_health, services_health,
};
pub use types::{HealthCategory, HealthGrade, HealthMetric, SystemHealthScore};
pub use utils::{health_summary_message, is_health_query};
