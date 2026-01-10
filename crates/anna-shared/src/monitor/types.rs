//! Monitor type definitions.

use serde::{Deserialize, Serialize};

/// Types of issues Anna can detect
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    /// Disk space below threshold
    DiskSpaceLow,
    /// Memory usage high
    MemoryHigh,
    /// CPU load high
    CpuHigh,
    /// Systemd service failed
    ServiceFailed,
    /// Packages need security updates
    SecurityUpdates,
    /// Journal contains errors
    JournalErrors,
    /// Configuration issue detected
    ConfigIssue,
    /// Permission problem
    PermissionIssue,
    /// Network issue
    NetworkIssue,
    /// Custom detected issue
    Custom(String),
}

/// Severity of an issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum Severity {
    /// Informational - good to know
    Info,
    /// Warning - should be addressed soon
    Warning,
    /// Critical - needs immediate attention
    Critical,
}

/// A detected issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Type of issue
    pub issue_type: IssueType,

    /// Severity level
    pub severity: Severity,

    /// Human-readable summary
    pub summary: String,

    /// Detailed description
    pub details: String,

    /// Suggested fix (if known)
    pub suggested_fix: Option<String>,

    /// When this was detected
    pub detected_at: String,

    /// Whether user has been notified
    pub notified: bool,

    /// Whether user has acknowledged/dismissed
    pub acknowledged: bool,
}

/// Results of a monitoring check
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MonitorResults {
    /// All detected issues
    pub issues: Vec<Issue>,

    /// When the check was run
    pub checked_at: String,

    /// How long the check took (ms)
    pub duration_ms: u64,
}

/// Monitoring thresholds (configurable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorThresholds {
    /// Disk usage percentage to trigger warning
    pub disk_warning_percent: u8,
    /// Disk usage percentage to trigger critical
    pub disk_critical_percent: u8,
    /// Memory usage percentage to trigger warning
    pub memory_warning_percent: u8,
    /// CPU load average to trigger warning (1-min)
    pub cpu_warning_load: f32,
    /// Maximum days since last update
    pub update_warning_days: u32,
}

impl Default for MonitorThresholds {
    fn default() -> Self {
        Self {
            disk_warning_percent: 85,
            disk_critical_percent: 95,
            memory_warning_percent: 90,
            cpu_warning_load: 4.0, // 4x CPU count is concerning
            update_warning_days: 7,
        }
    }
}
