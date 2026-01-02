// v0.0.584: Settings Metrics (Phase 160)
// Metrics collection and reporting for settings

mod types;
mod metric;
mod collection;
mod utils;

// Re-export all public types and functions to preserve API
pub use types::{MetricKind, MetricUnit, MetricValue};
pub use metric::Metric;
pub use collection::SettingsMetrics;
pub use utils::{format_metrics, is_metrics_query, settings_metrics_fun_fact};
