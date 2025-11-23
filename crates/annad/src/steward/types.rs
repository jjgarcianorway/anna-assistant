//! Steward types and data structures
//!
//! Phase 0.9: System Steward types
//! Beta.271: Proactive Issue Summary
//! Citation: [archwiki:System_maintenance]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Proactive issue summary (Beta.271)
///
/// User-safe representation of a correlated issue from the proactive engine.
/// All internal Rust types are mapped to strings for safe transmission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveIssueSummary {
    /// Root cause type (user-safe string label, not Rust enum)
    /// Examples: "network_routing_conflict", "disk_pressure", "memory_pressure"
    pub root_cause: String,

    /// Severity level: "critical", "warning", "info", "trend"
    pub severity: String,

    /// Human-readable one-line summary
    pub summary: String,

    /// Optional rule ID for future remediation mapping
    pub rule_id: Option<String>,

    /// Confidence level (0.7-1.0)
    pub confidence: f32,

    /// When this issue was first detected
    pub first_seen: String, // ISO 8601

    /// When this issue was last updated
    pub last_seen: String, // ISO 8601
}

/// Steward configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StewardConfig {
    /// Enable automatic updates
    pub auto_update: bool,
    /// Update check interval (seconds)
    pub update_interval: u64,
    /// Create snapshots before updates
    pub snapshot_enabled: bool,
}

impl Default for StewardConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            update_interval: 86400, // 24 hours
            snapshot_enabled: true,
        }
    }
}

/// System health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
    /// Overall health status
    pub overall_status: HealthStatus,
    /// Service statuses
    pub services: Vec<ServiceStatus>,
    /// Package statuses
    pub packages: Vec<PackageStatus>,
    /// Log analysis
    pub log_issues: Vec<LogIssue>,
    /// Network monitoring data (Beta.267)
    pub network_monitoring: Option<anna_common::network_monitoring::NetworkMonitoring>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Summary message
    pub message: String,
    /// Arch Wiki citation
    pub citation: String,
}

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Critical,
}

/// Service status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Service name
    pub name: String,
    /// Active state (active, inactive, failed)
    pub state: String,
    /// Load state (loaded, not-found)
    pub load: String,
    /// Sub state (running, dead, exited)
    pub sub: String,
    /// Description
    pub description: String,
}

/// Package status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageStatus {
    /// Package name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Update available
    pub update_available: bool,
    /// New version (if update available)
    pub new_version: Option<String>,
}

/// Log issue detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssue {
    /// Severity (error, warning, critical)
    pub severity: String,
    /// Issue description
    pub message: String,
    /// Log source (systemd, kernel, application)
    pub source: String,
    /// First occurrence
    pub first_seen: DateTime<Utc>,
    /// Count
    pub count: u32,
}

/// System update report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReport {
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
    /// Was this a dry-run?
    pub dry_run: bool,
    /// Update success
    pub success: bool,
    /// Packages updated
    pub packages_updated: Vec<PackageUpdate>,
    /// Services restarted
    pub services_restarted: Vec<String>,
    /// Snapshot created
    pub snapshot_path: Option<String>,
    /// Summary message
    pub message: String,
    /// Arch Wiki citation
    pub citation: String,
}

/// Package update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdate {
    /// Package name
    pub name: String,
    /// Old version
    pub old_version: String,
    /// New version
    pub new_version: String,
    /// Size change (bytes)
    pub size_change: i64,
}

/// System audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
    /// Overall compliance status
    pub compliant: bool,
    /// Integrity checks
    pub integrity: Vec<IntegrityStatus>,
    /// Security findings
    pub security_findings: Vec<SecurityFinding>,
    /// Configuration issues
    pub config_issues: Vec<ConfigIssue>,
    /// Summary message
    pub message: String,
    /// Arch Wiki citation
    pub citation: String,
}

/// Integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityStatus {
    /// Component name
    pub component: String,
    /// Check type (signature, checksum, ownership)
    pub check_type: String,
    /// Pass/fail
    pub passed: bool,
    /// Details
    pub details: String,
}

/// Security finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    /// Severity (low, medium, high, critical)
    pub severity: String,
    /// Finding description
    pub description: String,
    /// Recommendation
    pub recommendation: String,
    /// Arch Wiki reference
    pub reference: String,
}

/// Configuration issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigIssue {
    /// File path
    pub file: String,
    /// Issue description
    pub issue: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
}
