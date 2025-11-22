//! Sysadmin Answer Composer - Beta.263
//!
//! Deterministic answer patterns for services, disk, and logs.
//! These functions compose focused, sysadmin-grade answers using real system data.

use anna_common::ipc::{BrainAnalysisData, DiagnosticInsightData};
use anna_common::systemd_health::SystemdHealth;

/// Compose a focused service health answer
///
/// Provides sysadmin-grade service health analysis with:
/// - Clear summary of service state
/// - Top failed services with details
/// - Relevant systemctl commands
pub fn compose_service_health_answer(brain: &BrainAnalysisData, systemd: &SystemdHealth) -> String {
    let failed_count = systemd.failed_units.len();

    // Build summary
    let summary = if failed_count == 0 {
        "[SUMMARY]\nService health: all core services are running.".to_string()
    } else if failed_count == 1 {
        "[SUMMARY]\nService health: 1 failed service detected.".to_string()
    } else {
        format!("[SUMMARY]\nService health: {} failed services detected.", failed_count)
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if failed_count == 0 {
        details.push_str("\nNo failed systemd units.");
    } else {
        // Show top 3 failed units
        for (i, unit) in systemd.failed_units.iter().take(3).enumerate() {
            if i == 0 {
                details.push('\n');
            }
            details.push_str(&format!(
                "\n✗ {} ({})",
                unit.name,
                unit.active_state
            ));
        }

        if failed_count > 3 {
            details.push_str(&format!("\n... and {} more", failed_count - 3));
        }
    }

    // Build commands
    let mut commands = String::from("\n\n[COMMANDS]");
    if failed_count > 0 {
        commands.push_str("\n# List all failed services:");
        commands.push_str("\nsystemctl --failed");

        // Add status command for first failed service
        if let Some(first_unit) = systemd.failed_units.first() {
            commands.push_str(&format!("\n\n# Check specific service status:"));
            commands.push_str(&format!("\nsystemctl status {}", first_unit.name));
            commands.push_str(&format!("\n\n# View service logs:"));
            commands.push_str(&format!("\njournalctl -u {} -n 50", first_unit.name));
        }
    } else {
        commands.push_str("\n# Verify service health:");
        commands.push_str("\nsystemctl --failed");
        commands.push_str("\n\n# List all services:");
        commands.push_str("\nsystemctl list-units --type=service");
    }

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused disk health answer
///
/// Provides sysadmin-grade disk analysis with:
/// - Clear summary of disk state
/// - Affected mount points with usage
/// - Relevant df/du commands
pub fn compose_disk_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract disk insights from brain analysis
    let disk_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("disk") ||
                    i.summary.to_lowercase().contains("filesystem"))
        .collect();

    let has_critical = disk_insights.iter().any(|i| i.severity == "critical");
    let has_warning = disk_insights.iter().any(|i| i.severity == "warning");

    // Build summary
    let summary = if has_critical {
        "[SUMMARY]\nDisk health: critical – filesystem usage requires attention.".to_string()
    } else if has_warning {
        "[SUMMARY]\nDisk health: warning – some filesystems approaching capacity.".to_string()
    } else {
        "[SUMMARY]\nDisk health: all monitored filesystems below 80% usage.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if disk_insights.is_empty() {
        details.push_str("\nAll filesystems are within normal usage thresholds.");
    } else {
        for insight in disk_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check filesystem usage:\n\
df -h\n\
\n\
# Check inode usage:\n\
df -i\n\
\n\
# Find large directories:\n\
du -h /var | sort -h | tail -20\n\
\n\
# Check /tmp usage:\n\
du -sh /tmp/*";

    format!("{}{}{}", summary, details, commands)
}

/// Compose a focused log health answer
///
/// Provides sysadmin-grade log analysis with:
/// - Clear summary of log state
/// - Representative error types
/// - Relevant journalctl commands
pub fn compose_log_health_answer(brain: &BrainAnalysisData) -> String {
    // Extract log-related insights
    let log_insights: Vec<_> = brain.insights.iter()
        .filter(|i| i.summary.to_lowercase().contains("log") ||
                    i.summary.to_lowercase().contains("error") ||
                    i.summary.to_lowercase().contains("journal"))
        .collect();

    let has_critical = log_insights.iter().any(|i| i.severity == "critical");
    let has_errors = !log_insights.is_empty();

    // Build summary
    let summary = if has_critical {
        "[SUMMARY]\nLog health: critical errors detected in recent system logs.".to_string()
    } else if has_errors {
        "[SUMMARY]\nLog health: warnings detected in recent system logs.".to_string()
    } else {
        "[SUMMARY]\nLog health: no critical errors in recent system logs.".to_string()
    };

    // Build details
    let mut details = String::from("\n\n[DETAILS]");

    if log_insights.is_empty() {
        details.push_str("\nNo significant errors detected in systemd journal.");
    } else {
        for insight in log_insights.iter().take(3) {
            details.push_str(&format!("\n• {}", insight.summary));
        }
    }

    // Build commands
    let commands = "\n\n[COMMANDS]\n\
# Check recent errors:\n\
journalctl -p err -n 20\n\
\n\
# Check critical errors since boot:\n\
journalctl -p crit -b\n\
\n\
# Check last hour of logs:\n\
journalctl --since \"1 hour ago\"\n\
\n\
# Check journal disk usage:\n\
journalctl --disk-usage";

    format!("{}{}{}", summary, details, commands)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anna_common::systemd_health::FailedUnit;

    #[test]
    fn test_service_health_no_failures() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("all core services are running"));
        assert!(answer.contains("[DETAILS]"));
        assert!(answer.contains("[COMMANDS]"));
        assert!(answer.contains("systemctl --failed"));
    }

    #[test]
    fn test_service_health_one_failure() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![
                FailedUnit {
                    name: "docker.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                }
            ],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("1 failed service detected"));
        assert!(answer.contains("docker.service"));
        assert!(answer.contains("systemctl status docker.service"));
        assert!(answer.contains("journalctl -u docker.service"));
    }

    #[test]
    fn test_service_health_multiple_failures() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };
        let systemd = SystemdHealth {
            failed_units: vec![
                FailedUnit {
                    name: "docker.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                },
                FailedUnit {
                    name: "NetworkManager.service".to_string(),
                    unit_type: "service".to_string(),
                    load_state: "loaded".to_string(),
                    active_state: "failed".to_string(),
                    sub_state: "failed".to_string(),
                }
            ],
            essential_timers: vec![],
            journal_disk_usage_mb: Some(100),
            journal_rotation_configured: true,
        };

        let answer = compose_service_health_answer(&brain, &systemd);

        assert!(answer.contains("2 failed services detected"));
        assert!(answer.contains("docker.service"));
        assert!(answer.contains("NetworkManager.service"));
    }

    #[test]
    fn test_disk_health_critical() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "disk-space".to_string(),
                    summary: "Disk usage critical on /: 95% full".to_string(),
                    details: "Root filesystem is at 95% capacity".to_string(),
                    evidence: "df shows 95% usage".to_string(),
                    severity: "critical".to_string(),
                    commands: vec!["df -h".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 0,
        };

        let answer = compose_disk_health_answer(&brain);

        assert!(answer.contains("[SUMMARY]"));
        assert!(answer.contains("critical"));
        assert!(answer.contains("Disk usage critical on /"));
        assert!(answer.contains("df -h"));
        assert!(answer.contains("df -i"));
    }

    #[test]
    fn test_disk_health_healthy() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_disk_health_answer(&brain);

        assert!(answer.contains("below 80% usage"));
        assert!(answer.contains("df -h"));
    }

    #[test]
    fn test_log_health_critical() {
        let brain = BrainAnalysisData {
            insights: vec![
                DiagnosticInsightData {
                    rule_id: "journal-errors".to_string(),
                    summary: "Critical errors in systemd journal".to_string(),
                    details: "Multiple critical log entries found".to_string(),
                    evidence: "Found 5 critical log entries".to_string(),
                    severity: "critical".to_string(),
                    commands: vec!["journalctl -p crit".to_string()],
                    citations: vec![],
                }
            ],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 1,
            warning_count: 0,
        };

        let answer = compose_log_health_answer(&brain);

        assert!(answer.contains("critical errors detected"));
        assert!(answer.contains("Critical errors in systemd journal"));
        assert!(answer.contains("journalctl -p err"));
    }

    #[test]
    fn test_log_health_clean() {
        let brain = BrainAnalysisData {
            insights: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            formatted_output: String::new(),
            critical_count: 0,
            warning_count: 0,
        };

        let answer = compose_log_health_answer(&brain);

        assert!(answer.contains("no critical errors"));
        assert!(answer.contains("journalctl"));
    }
}
