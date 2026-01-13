//! Performance monitoring checks (boot time).

use std::process::Command;

use crate::monitor::types::{Issue, IssueType, Severity};

/// Check boot time for anomalies
pub fn check_boot_time() -> Vec<Issue> {
    let mut issues = Vec::new();

    if let Ok(output) = Command::new("systemd-analyze").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract total boot time
        // Format: "Startup finished in 1.5s (firmware) + 2.3s (loader) + ... = 15.234s"
        if let Some(total) = stdout.split('=').last() {
            let total = total.trim();
            // Extract seconds
            if let Some(secs_str) = total.strip_suffix('s') {
                if let Ok(secs) = secs_str.trim().parse::<f32>() {
                    if secs > 120.0 {
                        issues.push(Issue {
                            issue_type: IssueType::SlowBoot,
                            severity: Severity::Warning,
                            summary: format!("Slow boot: {:.1}s", secs),
                            details: "Boot time exceeds 2 minutes.".to_string(),
                            suggested_fix: Some(
                                "Run: systemd-analyze blame | head -20".to_string(),
                            ),
                            detected_at: chrono::Utc::now().to_rfc3339(),
                            notified: false,
                            acknowledged: false,
                        });
                    } else if secs > 60.0 {
                        issues.push(Issue {
                            issue_type: IssueType::SlowBoot,
                            severity: Severity::Info,
                            summary: format!("Boot time: {:.1}s", secs),
                            details: "Boot time is over 1 minute.".to_string(),
                            suggested_fix: Some("Check: systemd-analyze blame".to_string()),
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
