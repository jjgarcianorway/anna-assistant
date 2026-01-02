//! System Monitoring (v0.0.469).
//!
//! Proactive system monitoring capabilities per VISION.md:
//! "Impressive system monitoring capabilities"
//!
//! Checks system metrics and triggers alarms when conditions are met.

mod checks;
mod evaluators;
mod platform;
mod types;

// Re-export public API
pub use checks::{
    check_disk_usage, check_failed_services, check_load_average, check_memory_usage,
    check_swap_usage, run_all_checks,
};
pub use evaluators::{check_conditional_alarms, evaluate_condition};
pub use types::{format_monitor_results, CheckType, MonitorResult};
