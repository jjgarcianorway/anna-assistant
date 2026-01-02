// v0.0.640: Settings Report Generator (Phase 216)
// Generator for settings status and health reports

mod types;
mod config;
mod report;
mod stats;
mod reporter;
mod registry;
mod utils;

#[cfg(test)]
mod tests;

// Re-export all public types to preserve the original API
pub use types::{ReportType, ReportFormat};
pub use config::ReporterConfig;
pub use report::{Report, ReportSection};
pub use stats::ReporterStats;
pub use reporter::SettingsReporter;
pub use registry::SettingsReporterRegistry;
pub use utils::{format_reporter_registry, is_reporter_query, reporter_fun_fact};
