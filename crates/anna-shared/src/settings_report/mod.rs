// v0.0.712: Settings Report Module (Phase 288)
// Formal reports on settings changes and status

mod types;
mod config;
mod section;
mod stats;
mod report;
mod registry;
mod utils;

// Re-export all public items to maintain the original API
pub use types::{ReportType, ReportFrequency};
pub use config::ReportConfig;
pub use section::{ReportSection, ReportAppendix};
pub use stats::ReportStats;
pub use report::SettingsReport;
pub use registry::ReportRegistry;
pub use utils::{format_report_registry, is_report_query, report_fun_fact};
