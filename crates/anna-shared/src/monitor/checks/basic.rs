//! Basic system monitoring checks (disk, memory, services, journal, updates).

use std::path::PathBuf;
use std::process::Command;

use crate::monitor::types::{Issue, IssueType, MonitorThresholds, Severity};

/// Check disk space on all mounted filesystems
pub fn check_disk_space(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("df")
        .args([
            "--output=target,pcent",
            "-x",
            "tmpfs",
            "-x",
            "devtmpfs",
            "-x",
            "squashfs",
        ])
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
                            suggested_fix: Some(
                                "Consider running: paccache -rk2 && pacman -Sc".to_string(),
                            ),
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
pub fn check_memory(thresholds: &MonitorThresholds) -> Vec<Issue> {
    let mut issues = Vec::new();

    let output = Command::new("free").args(["-m"]).output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(total), Ok(used)) =
                        (parts[1].parse::<u64>(), parts[2].parse::<u64>())
                    {
                        let percent = (used * 100 / total) as u8;
                        if percent >= thresholds.memory_warning_percent {
                            issues.push(Issue {
                                issue_type: IssueType::MemoryHigh,
                                severity: Severity::Warning,
                                summary: format!(
                                    "Memory {}% used ({}/{}MB)",
                                    percent, used, total
                                ),
                                details: "High memory usage detected. System may become slow."
                                    .to_string(),
                                suggested_fix: Some(
                                    "Check: ps aux --sort=-%mem | head -10".to_string(),
                                ),
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
pub fn check_failed_services() -> Vec<Issue> {
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
pub fn check_journal_errors() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for critical errors in last hour
    let output = Command::new("journalctl")
        .args([
            "-p",
            "err",
            "--since",
            "1 hour ago",
            "-q",
            "--no-pager",
            "-n",
            "5",
        ])
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
pub fn check_updates(thresholds: &MonitorThresholds) -> Vec<Issue> {
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
                    severity: if days > 14 {
                        Severity::Warning
                    } else {
                        Severity::Info
                    },
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
