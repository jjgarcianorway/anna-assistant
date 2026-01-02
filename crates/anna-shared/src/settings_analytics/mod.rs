// v0.0.577: Settings Analytics (Phase 153)
// Track settings usage patterns and provide insights

mod types;
mod event;
mod stats;
mod tracker;
mod utils;

// Re-export all public types and functions to maintain the same API
pub use types::{AnalyticsPeriod, MetricType};
pub use event::AnalyticsEvent;
pub use stats::{CategoryStats, AnalyticsSummary};
pub use tracker::SettingsAnalytics;
pub use utils::{format_analytics, is_analytics_query, settings_analytics_fun_fact};
