//! Aggregated Stats Dashboard (v0.0.491).
//!
//! Provides a unified view of all statistics.
//! Combines data from multiple stats modules into one dashboard.

pub mod builder;
pub mod formatting;
pub mod query;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use builder::DashboardBuilder;
pub use formatting::{format_dashboard, format_dashboard_compact, format_dashboard_oneline};
pub use query::{detect_section, is_dashboard_query};
pub use types::{DashboardSection, StatMetric, StatTrend, StatsDashboard};
