// v0.0.583: Settings Diagnostics Module
// Diagnostics and health checking for settings

pub mod report;
pub mod runner;
pub mod types;
pub mod utils;

// Re-export public API to preserve backwards compatibility
pub use report::DiagnosticReport;
pub use runner::SettingsDiagnostics;
pub use types::{DiagnosticIssue, DiagnosticSeverity, DiagnosticType};
pub use utils::{format_diagnostics, is_diagnostics_query, settings_diagnostics_fun_fact};
