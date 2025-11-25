//! System health monitoring
//!
//! Phase 0.9: Health checks for services, packages, and logs
//! Beta.279: Historian v1 integration
//! Citation: [archwiki:System_maintenance]

use super::types::{HealthReport, HealthStatus, LogIssue, PackageStatus, ServiceStatus};
use crate::historian::{Historian, HistoryEvent};
use anyhow::{Context, Result};
use chrono::Utc;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

/// Check overall system health
pub async fn check_health() -> Result<HealthReport> {
    info!("Starting health check");

    // Check services
    let services = check_services().await?;

    // Check packages
    let packages = check_packages().await?;

    // Analyze logs
    let log_issues = analyze_logs().await?;

    // Beta.267: Collect network monitoring data
    let network_monitoring = Some(anna_common::network_monitoring::NetworkMonitoring::detect());

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

    let report = HealthReport {
        timestamp: Utc::now(),
        overall_status,
        services,
        packages,
        log_issues,
        network_monitoring, // Beta.267
        recommendations,
        message,
        citation: "[archwiki:System_maintenance]".to_string(),
    };

    // Beta.279: Append to historian after successful health check
    append_to_historian(&report).await;

    Ok(report)
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

    // v6.37.0: Fixed to check both official repos and AUR
    let output = Command::new("yay")
        .arg("-Qu")
        .output()
        .or_else(|_| Command::new("pacman").arg("-Qu").output())
        .or_else(|_| Command::new("checkupdates").output());

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

// ============================================================================
// Beta.279: Historian Integration
// ============================================================================

/// Lazy-initialized global historian
static HISTORIAN: once_cell::sync::Lazy<Arc<Mutex<Option<Historian>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(None)));

/// Get or initialize the global historian
async fn get_historian() -> Result<Arc<Mutex<Option<Historian>>>> {
    let mut hist_lock = HISTORIAN.lock().await;

    if hist_lock.is_none() {
        // Initialize historian with default Anna state directory
        let state_dir = std::path::PathBuf::from("/var/lib/anna/state");
        match Historian::new(&state_dir) {
            Ok(historian) => {
                info!("Historian initialized successfully");
                *hist_lock = Some(historian);
            }
            Err(e) => {
                error!("Failed to initialize historian: {}. History will not be recorded.", e);
                // Don't set historian - will try again on next health check
            }
        }
    }

    drop(hist_lock);
    Ok(HISTORIAN.clone())
}

/// Append health report to historian
///
/// This function is called after each successful health check.
/// Failures are logged but never crash the daemon.
async fn append_to_historian(report: &HealthReport) {
    // Get historian instance
    let historian_arc = match get_historian().await {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to get historian: {}. Skipping history append.", e);
            return;
        }
    };

    let mut hist_lock = historian_arc.lock().await;
    let historian = match hist_lock.as_mut() {
        Some(h) => h,
        None => {
            // Historian failed to initialize
            return;
        }
    };

    // Build history event from health report
    let event = build_history_event(report, historian);

    // Append to history
    if let Err(e) = historian.append(&event) {
        error!("Failed to append to historian: {}. History will be incomplete.", e);
    }
}

/// Build a history event from a health report
///
/// Extracts compact metrics from the report for historian storage.
fn build_history_event(report: &HealthReport, historian: &Historian) -> HistoryEvent {
    // Count failed and degraded services
    let failed_services_count = report
        .services
        .iter()
        .filter(|s| s.state == "failed")
        .count() as u16;

    let degraded_services_count = report
        .services
        .iter()
        .filter(|s| s.state == "degraded" || s.sub == "degraded")
        .count() as u16;

    // Extract disk usage
    // For now, use simple heuristics - in a real implementation,
    // we'd parse actual disk usage from telemetry
    let disk_root_usage_pct = estimate_disk_usage(&report);
    let disk_other_max_usage_pct = 0; // Simplified for v1

    // Extract network metrics if available
    let (network_packet_loss_pct, network_latency_ms) = if let Some(ref net) = report.network_monitoring {
        // Use internet packet loss as primary metric (fallback to gateway or DNS)
        let loss_pct = net.packet_loss.internet_loss_percent
            .or(net.packet_loss.gateway_loss_percent)
            .or(net.packet_loss.dns_loss_percent)
            .unwrap_or(0.0);
        let loss = (loss_pct).min(100.0).max(0.0) as u8;

        // Use internet latency as primary metric (fallback to gateway or DNS)
        let latency_value = net.latency.internet_latency_ms
            .or(net.latency.gateway_latency_ms)
            .or(net.latency.dns_latency_ms)
            .unwrap_or(0.0);
        let latency = (latency_value).min(65535.0).max(0.0) as u16;

        (loss, latency)
    } else {
        (0, 0)
    };

    // Resource flags - simplified heuristic based on health status
    let high_cpu_flag = report.overall_status == HealthStatus::Critical &&
                        report.message.to_lowercase().contains("cpu");
    let high_memory_flag = report.overall_status == HealthStatus::Critical &&
                           report.message.to_lowercase().contains("memory");

    // Get system info
    let kernel_version = get_kernel_version();
    let hostname = get_hostname();
    let boot_id = get_boot_id();

    // Detect kernel change
    let kernel_changed = historian
        .last_kernel_version()
        .map(|last| last != kernel_version)
        .unwrap_or(false);

    // Device hotplug - simplified (would need udev monitoring for real detection)
    let device_hotplug_flag = false;

    HistoryEvent {
        schema_version: 1,
        timestamp_utc: report.timestamp,
        kernel_version,
        hostname,
        disk_root_usage_pct,
        disk_other_max_usage_pct,
        failed_services_count,
        degraded_services_count,
        high_cpu_flag,
        high_memory_flag,
        network_packet_loss_pct,
        network_latency_ms,
        boot_id,
        kernel_changed,
        device_hotplug_flag,
    }
}

/// Estimate disk usage from health report
/// Simplified - returns moderate usage estimate
fn estimate_disk_usage(_report: &HealthReport) -> u8 {
    // In a real implementation, we'd parse df output or use sysinfo
    // For now, return a reasonable default
    50
}

/// Get current kernel version
fn get_kernel_version() -> String {
    let output = Command::new("uname")
        .arg("-r")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Get hostname
fn get_hostname() -> String {
    let output = Command::new("hostname")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// Get boot ID from /proc/sys/kernel/random/boot_id
fn get_boot_id() -> String {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string()
}
