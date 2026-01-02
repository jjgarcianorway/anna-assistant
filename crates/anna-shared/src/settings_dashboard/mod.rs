// v0.0.579: Settings Dashboard (Phase 155)
// Unified dashboard for settings overview and management

mod changes;
mod dashboard;
mod formatting;
mod summary;
mod types;

// Re-export all public types to preserve the original API
pub use changes::RecentChange;
pub use dashboard::SettingsDashboard;
pub use formatting::{format_dashboard, is_dashboard_query, settings_dashboard_fun_fact};
pub use summary::{CategorySummary, DashboardStats};
pub use types::{DashboardSection, HealthLevel, QuickAction};
