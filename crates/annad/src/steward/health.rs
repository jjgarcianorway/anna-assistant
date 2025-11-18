//! System health monitoring
//!
//! Phase 0.9: Health checks for services, packages, and logs
//! Citation: [archwiki:System_maintenance]

use super::types::{HealthReport, HealthStatus, LogIssue, PackageStatus, ServiceStatus};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use tracing::{info, warn};

/// Check overall system health
pub async fn check_health() -> Result<HealthReport> {
    info!("Starting health check");

    // Check services
    let services = check_services().await?;

    // Check packages
    let packages = check_packages().await?;

    // Analyze logs
    let log_issues = analyze_logs().await?;

    // Determine overall status
    let failed_services = services.iter().filter(|s| s.state == "failed").count();
    let critical_logs = log_issues
        .iter()
        .filter(|l| l.severity == "critical")
        .count();

    let overall_status = if failed_services > 0 || critical_logs > 0 {
        HealthStatus::Critical
    } else if !log_issues.is_empty() || packages.iter().any(|p| p.update_available) {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    // Generate recommendations
    let mut recommendations = Vec::new();
    if failed_services > 0 {
        recommendations.push(format!(
            "{} failed services detected - run 'annactl repair services-failed'",
            failed_services
        ));
    }
    if packages.iter().any(|p| p.update_available) {
        recommendations.push("Updates available - run 'annactl update'".to_string());
    }
    if !log_issues.is_empty() {
        recommendations.push(format!(
            "{} log issues detected - review with 'journalctl -p err'",
            log_issues.len()
        ));
    }

    // Generate summary message
    let message = match overall_status {
        HealthStatus::Healthy => "System health: OK".to_string(),
        HealthStatus::Degraded => format!(
            "System degraded: {} failed services, {} log issues",
            failed_services,
            log_issues.len()
        ),
        HealthStatus::Critical => format!(
            "System critical: {} failed services, {} critical logs",
            failed_services, critical_logs
        ),
    };

    Ok(HealthReport {
        timestamp: Utc::now(),
        overall_status,
        services,
        packages,
        log_issues,
        recommendations,
        message,
        citation: "[archwiki:System_maintenance]".to_string(),
    })
}

/// Check systemd services
async fn check_services() -> Result<Vec<ServiceStatus>> {
    info!("Checking systemd services");

    let output = Command::new("systemctl")
        .args(&[
            "list-units",
            "--type=service",
            "--all",
            "--no-pager",
            "--no-legend",
            "--plain",
        ])
        .output()
        .context("Failed to run systemctl")?;

    if !output.status.success() {
        warn!("systemctl failed");
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut services = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let load = parts[1].to_string();
            let active = parts[2].to_string();
            let sub = parts[3].to_string();
            let description = parts[4..].join(" ");

            // Only include important services or failed ones
            if active == "failed" || is_important_service(&name) {
                services.push(ServiceStatus {
                    name,
                    state: active,
                    load,
                    sub,
                    description,
                });
            }
        }
    }

    Ok(services)
}

/// Check if service is important to monitor
fn is_important_service(name: &str) -> bool {
    let important = [
        "NetworkManager",
        "sshd",
        "systemd-networkd",
        "systemd-resolved",
        "dbus",
        "annad",
    ];

    important.iter().any(|&svc| name.contains(svc))
}

/// Check package status
async fn check_packages() -> Result<Vec<PackageStatus>> {
    info!("Checking package updates");

    // Get list of updates available
    let output = Command::new("checkupdates").output();

    let mut packages = Vec::new();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    packages.push(PackageStatus {
                        name: parts[0].to_string(),
                        version: parts[1].to_string(),
                        update_available: true,
                        new_version: Some(parts[3].to_string()),
                    });
                }
            }
        }
    }

    Ok(packages)
}

/// Analyze system logs
async fn analyze_logs() -> Result<Vec<LogIssue>> {
    info!("Analyzing system logs");

    let output = Command::new("journalctl")
        .args(&[
            "-p",
            "err",
            "--since",
            "24 hours ago",
            "--no-pager",
            "-o",
            "json",
        ])
        .output()
        .context("Failed to run journalctl")?;

    let mut issues = Vec::new();

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Simple parsing - count unique error messages
        let mut error_counts: std::collections::HashMap<String, u32> =
            std::collections::HashMap::new();

        for line in stdout.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(message) = json.get("MESSAGE").and_then(|v| v.as_str()) {
                    *error_counts.entry(message.to_string()).or_insert(0) += 1;
                }
            }
        }

        // Convert to issues (limit to top 10)
        let mut sorted: Vec<_> = error_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        for (message, count) in sorted.iter().take(10) {
            issues.push(LogIssue {
                severity: "error".to_string(),
                message: message.clone(),
                source: "systemd".to_string(),
                first_seen: Utc::now(), // Simplified
                count: *count,
            });
        }
    }

    Ok(issues)
}
