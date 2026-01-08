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

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::config::anna_data_dir;

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

/// Run all monitoring checks
pub fn run_checks(thresholds: &MonitorThresholds) -> MonitorResults {
    let start = std::time::Instant::now();
    let mut issues = Vec::new();

    // Check disk space
    issues.extend(check_disk_space(thresholds));

    // Check memory
    issues.extend(check_memory(thresholds));

    // Check failed services
    issues.extend(check_failed_services());

    // Check journal for recent errors
    issues.extend(check_journal_errors());

    // Check for security updates
    issues.extend(check_updates(thresholds));

    MonitorResults {
        issues,
        checked_at: chrono::Utc::now().to_rfc3339(),
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

/// Check disk space on all mounted filesystems
fn check_disk_space(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("df")
        .args(["--output=target,pcent", "-x", "tmpfs", "-x", "devtmpfs", "-x", "squashfs"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mount = parts[0];
                let percent_str = parts[1].trim_end_matches('%');
                if let Ok(percent) = percent_str.parse::<u8>() {
                    if percent >= thresholds.disk_critical_percent {
                        issues.push(Issue {
                            issue_type: IssueType::DiskSpaceLow,
                            severity: Severity::Critical,
                            summary: format!("{} is {}% full", mount, percent),
                            details: format!(
                                "Filesystem {} has only {}% free space. This can cause system instability.",
                                mount,
                                100 - percent
                            ),
                            suggested_fix: Some(format!(
                                "Run: du -sh {}/* | sort -rh | head -10 to find large directories",
                                mount
                            )),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    } else if percent >= thresholds.disk_warning_percent {
                        issues.push(Issue {
                            issue_type: IssueType::DiskSpaceLow,
                            severity: Severity::Warning,
                            summary: format!("{} is {}% full", mount, percent),
                            details: format!(
                                "Filesystem {} is getting full. Consider cleaning up.",
                                mount
                            ),
                            suggested_fix: Some("Consider running: paccache -rk2 && pacman -Sc".to_string()),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    }
                }
            }
        }
    }

    issues
}

/// Check memory usage
fn check_memory(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("free").args(["-m"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) = (
                        parts[1].parse::<u64>(),
                        parts[2].parse::<u64>(),
                    ) {
                        let percent = (used * 100 / total) as u8;
                        if percent >= thresholds.memory_warning_percent {
                            issues.push(Issue {
                                issue_type: IssueType::MemoryHigh,
                                severity: Severity::Warning,
                                summary: format!("Memory {}% used ({}/{}MB)", percent, used, total),
                                details: "High memory usage detected. System may become slow.".to_string(),
                                suggested_fix: Some("Check: ps aux --sort=-%mem | head -10".to_string()),
                                detected_at: chrono::Utc::now().to_rfc3339(),
                                notified: false,
                                acknowledged: false,
                            });
                        }
                    }
                }
            }
        }
    }

    issues
}

/// Check for failed systemd services
fn check_failed_services() -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("systemctl")
        .args(["--failed", "--no-legend", "--plain"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                let service = parts[0];
                // Skip user services when running as root
                if !service.contains("user@") {
                    issues.push(Issue {
                        issue_type: IssueType::ServiceFailed,
                        severity: Severity::Warning,
                        summary: format!("Service {} failed", service),
                        details: format!("Systemd service {} is in failed state.", service),
                        suggested_fix: Some(format!(
                            "Check: journalctl -u {} -n 50 --no-pager",
                            service
                        )),
                        detected_at: chrono::Utc::now().to_rfc3339(),
                        notified: false,
                        acknowledged: false,
                    });
                }
            }
        }
    }

    issues
}

/// Check journal for recent errors
fn check_journal_errors() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for critical errors in last hour
    let output = Command::new("journalctl")
        .args(["-p", "err", "--since", "1 hour ago", "-q", "--no-pager", "-n", "5"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_count = stdout.lines().count();

        if error_count > 3 {
            issues.push(Issue {
                issue_type: IssueType::JournalErrors,
                severity: Severity::Info,
                summary: format!("{} errors in journal (last hour)", error_count),
                details: "Multiple errors logged in the last hour.".to_string(),
                suggested_fix: Some("Check: journalctl -p err --since '1 hour ago'".to_string()),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    issues
}

/// Check for needed updates
fn check_updates(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check when pacman was last synced
    let sync_db = PathBuf::from("/var/lib/pacman/sync");
    if let Ok(metadata) = std::fs::metadata(&sync_db) {
        if let Ok(modified) = metadata.modified() {
            let age = std::time::SystemTime::now()
                .duration_since(modified)
                .unwrap_or_default();

            let days = age.as_secs() / 86400;
            if days >= thresholds.update_warning_days as u64 {
                issues.push(Issue {
                    issue_type: IssueType::SecurityUpdates,
                    severity: if days > 14 { Severity::Warning } else { Severity::Info },
                    summary: format!("System not updated in {} days", days),
                    details: "Regular updates are important for security.".to_string(),
                    suggested_fix: Some("Run: pacman -Syu".to_string()),
                    detected_at: chrono::Utc::now().to_rfc3339(),
                    notified: false,
                    acknowledged: false,
                });
            }
        }
    }

    issues
}

/// Store for persistent issues tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IssueStore {
    /// Currently active issues
    pub active_issues: Vec<Issue>,

    /// Historical issues (resolved)
    pub history: Vec<Issue>,

    /// When the store was last updated
    pub last_updated: Option<String>,
}

impl IssueStore {
    /// Load from disk
    pub fn load() -> Result<Self> {
        let path = issues_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let store: IssueStore = serde_json::from_str(&content)?;
            Ok(store)
        } else {
            Ok(IssueStore::default())
        }
    }

    /// Save to disk
    pub fn save(&self) -> Result<()> {
        let path = issues_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Update with new check results
    pub fn update(&mut self, results: MonitorResults) {
        // Mark old issues that are no longer present as resolved
        for old_issue in &mut self.active_issues {
            let still_present = results.issues.iter().any(|new| {
                new.issue_type == old_issue.issue_type && new.summary == old_issue.summary
            });

            if !still_present {
                old_issue.acknowledged = true;
                self.history.push(old_issue.clone());
            }
        }

        // Keep only issues that are still present or new
        self.active_issues = results.issues;
        self.last_updated = Some(chrono::Utc::now().to_rfc3339());

        // Limit history size
        if self.history.len() > 100 {
            self.history = self.history.split_off(self.history.len() - 100);
        }
    }

    /// Get issues that haven't been notified yet
    pub fn get_unnotified(&self) -> Vec<&Issue> {
        self.active_issues.iter().filter(|i| !i.notified).collect()
    }

    /// Mark issues as notified
    pub fn mark_notified(&mut self) {
        for issue in &mut self.active_issues {
            issue.notified = true;
        }
    }

    /// Get critical issues
    pub fn get_critical(&self) -> Vec<&Issue> {
        self.active_issues
            .iter()
            .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
            .collect()
    }

    /// Acknowledge an issue
    pub fn acknowledge(&mut self, summary: &str) {
        if let Some(issue) = self.active_issues.iter_mut().find(|i| i.summary == summary) {
            issue.acknowledged = true;
        }
    }
}

/// Get issues storage path
pub fn issues_path() -> PathBuf {
    anna_data_dir().join("issues.json")
}

/// Format issues for display
pub fn format_issues_summary(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "No issues detected.".to_string();
    }

    let mut output = String::new();
    let critical: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Critical).collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Warning).collect();
    let info: Vec<_> = issues.iter().filter(|i| i.severity == Severity::Info).collect();

    if !critical.is_empty() {
        output.push_str("🔴 CRITICAL:\n");
        for issue in critical {
            output.push_str(&format!("  • {}\n", issue.summary));
        }
    }

    if !warnings.is_empty() {
        output.push_str("🟡 WARNINGS:\n");
        for issue in warnings {
            output.push_str(&format!("  • {}\n", issue.summary));
        }
    }

    if !info.is_empty() {
        output.push_str("ℹ️ INFO:\n");
        for issue in info {
            output.push_str(&format!("  • {}\n", issue.summary));
        }
    }

    output
}
