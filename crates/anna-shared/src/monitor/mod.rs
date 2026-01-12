//! Proactive Monitoring - Anna watches your system and warns about issues.
//!
//! This module runs periodic checks and alerts the user BEFORE problems occur:
//! - Disk space running low
//! - Failed systemd services
//! - High memory/CPU usage
//! - Security issues (outdated packages, open ports)
//! - Configuration problems
//! - Journal errors
//! - Hardware changes (USB/PCI devices added/removed)
//! - Config file changes (SSH, sudoers, etc.)
//!
//! Anna doesn't wait for you to ask - she proactively monitors.

mod baseline;
mod checks;
mod learning;
mod store;
mod types;

pub use baseline::{BaselineChanges, SystemBaseline, UsbDevice, PciDevice};
pub use checks::{run_checks, update_baseline};
pub use learning::{SystemLearning, DetectedChanges, PackageTransaction, PackageAction, PerfSample};
pub use store::{format_issues_summary, issues_path, IssueStore};
pub use types::{Issue, IssueType, MonitorResults, MonitorThresholds, Severity};
