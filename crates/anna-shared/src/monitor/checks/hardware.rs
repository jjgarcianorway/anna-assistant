//! Hardware monitoring checks (thermal, SMART).

use std::process::Command;

use crate::monitor::types::{Issue, IssueType, Severity};

/// Check thermal/temperature status
pub fn check_thermal() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check for thermal throttling
    let output = Command::new("journalctl")
        .args(["-k", "--since", "1 hour ago", "-q", "--no-pager"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("thermal")
            && (stdout.contains("throttl") || stdout.contains("critical"))
        {
            issues.push(Issue {
                issue_type: IssueType::ThermalThrottling,
                severity: Severity::Warning,
                summary: "Thermal throttling detected".to_string(),
                details: "CPU/GPU is throttling due to high temperature.".to_string(),
                suggested_fix: Some(
                    "Check cooling: sensors, clean fans, improve airflow".to_string(),
                ),
                detected_at: chrono::Utc::now().to_rfc3339(),
                notified: false,
                acknowledged: false,
            });
        }
    }

    // Check current temps if sensors available
    if let Ok(output) = Command::new("sensors").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Look for high temperatures (crude check)
        for line in stdout.lines() {
            if line.contains("°C") || line.contains("°F") {
                // Extract temperature value
                if let Some(temp_str) = line
                    .split_whitespace()
                    .find(|s| s.starts_with('+') && s.contains('°'))
                {
                    let temp_val: Option<f32> = temp_str
                        .trim_start_matches('+')
                        .split('°')
                        .next()
                        .and_then(|s| s.parse().ok());

                    if let Some(temp) = temp_val {
                        if temp > 90.0 {
                            issues.push(Issue {
                                issue_type: IssueType::ThermalThrottling,
                                severity: Severity::Critical,
                                summary: format!("High temperature: {}°C", temp),
                                details: "Component running very hot.".to_string(),
                                suggested_fix: Some("Improve cooling immediately".to_string()),
                                detected_at: chrono::Utc::now().to_rfc3339(),
                                notified: false,
                                acknowledged: false,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    issues
}

/// Check SMART health for drives
pub fn check_smart_health() -> Vec<Issue> {
    let mut issues = Vec::new();

    // Get list of block devices
    let lsblk = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME,TYPE"])
        .output();

    if let Ok(output) = lsblk {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "disk" {
                let device = format!("/dev/{}", parts[0]);

                // Try smartctl (may require root)
                if let Ok(smart) = Command::new("smartctl").args(["-H", &device]).output() {
                    let smart_out = String::from_utf8_lossy(&smart.stdout);
                    if smart_out.contains("FAILED") || smart_out.contains("FAILING") {
                        issues.push(Issue {
                            issue_type: IssueType::HardwareError,
                            severity: Severity::Critical,
                            summary: format!("SMART failure: {}", parts[0]),
                            details: format!(
                                "Drive {} is reporting SMART failures. Backup immediately!",
                                device
                            ),
                            suggested_fix: Some(format!(
                                "Run: smartctl -a {} and backup data",
                                device
                            )),
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
