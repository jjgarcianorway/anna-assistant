//! Learning-based monitoring checks.

use crate::monitor::learning::SystemLearning;
use crate::monitor::types::{Issue, IssueType, Severity};

/// Check for changes detected by the learning system
pub fn check_learned_changes() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Load and update learning
    let mut learning = SystemLearning::load();
    let changes = learning.update();

    // Report package installations
    if !changes.packages_installed.is_empty() {
        let pkg_list = changes.packages_installed.join(", ");
        let count = changes.packages_installed.len();
        issues.push(Issue {
            issue_type: IssueType::PackagesInstalled,
            severity: Severity::Info,
            summary: format!("{} package(s) installed", count),
            details: format!("New packages: {}", pkg_list),
            suggested_fix: Some("Run: pacman -Qe to list explicitly installed packages".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report boot time changes
    if let Some(diff) = changes.boot_time_change {
        let (summary, severity) = if diff > 0.0 {
            (
                format!("Boot {:.1}s slower than usual", diff),
                Severity::Warning,
            )
        } else {
            (
                format!("Boot {:.1}s faster than usual", diff.abs()),
                Severity::Info,
            )
        };

        issues.push(Issue {
            issue_type: IssueType::BootTimeChanged,
            severity,
            summary,
            details: "Boot time differs significantly from your system's average.".to_string(),
            suggested_fix: Some("Run: systemd-analyze blame | head -10".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report unusual commands
    for cmd in &changes.unusual_commands {
        issues.push(Issue {
            issue_type: IssueType::UnusualCommand,
            severity: Severity::Warning,
            summary: "Unusual command detected".to_string(),
            details: format!("Command '{}' matches suspicious patterns.", cmd),
            suggested_fix: Some("Review shell history for unauthorized access".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report performance anomalies
    for anomaly in &changes.performance_anomalies {
        issues.push(Issue {
            issue_type: IssueType::PerformanceAnomaly,
            severity: Severity::Warning,
            summary: anomaly.clone(),
            details: "Performance differs significantly from learned baseline.".to_string(),
            suggested_fix: Some("Check: htop or ps aux --sort=-%cpu | head -10".to_string()),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    // Report performance trend
    let trend = learning.performance_trend();
    if trend == "degrading" {
        issues.push(Issue {
            issue_type: IssueType::PerformanceDegraded,
            severity: Severity::Info,
            summary: "Performance trend: degrading".to_string(),
            details: "System performance has been gradually decreasing.".to_string(),
            suggested_fix: Some(
                "Consider: checking logs, clearing caches, reviewing recent changes".to_string(),
            ),
            detected_at: chrono::Utc::now().to_rfc3339(),
            notified: false,
            acknowledged: false,
        });
    }

    issues
}
