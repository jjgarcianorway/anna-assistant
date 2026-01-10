//! Proactive Monitoring - Anna watches your system and warns about issues.
//!
//! This module runs periodic checks and alerts the user BEFORE problems occur:
//! - Disk space running low
//! - Failed systemd services
//! - High memory/CPU usage
//! - Security issues (outdated packages, open ports)
//! - Configuration problems
//! - Journal errors
//!
//! Anna doesn't wait for you to ask - she proactively monitors.

mod checks;
mod store;
mod types;

pub use checks::run_checks;
pub use store::{format_issues_summary, issues_path, IssueStore};
pub use types::{Issue, IssueType, MonitorResults, MonitorThresholds, Severity};
