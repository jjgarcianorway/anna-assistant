//! SMART Disk Health Monitoring - Predict disk failures before they happen.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, warn, info};

const DISK_HEALTH_FILE: &str = "/var/lib/anna/disk_health.json";

/// SMART attribute that indicates disk health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: String,
    pub current: u64,
    pub worst: u64,
    pub threshold: u64,
    pub raw_value: u64,
    pub failing: bool,
}

/// Disk health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskHealth {
    pub device: String,
    pub health_status: HealthStatus,
    pub attributes: Vec<SmartAttribute>,
    pub last_check: String,
    pub predicted_failure_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Failing,
    Unknown,
}

/// Check SMART health for all disks
pub fn check_disk_health() -> Vec<DiskHealth> {
    let mut results = Vec::new();

    // Get list of disks
    let disks = get_disk_list();

    for disk in disks {
        if let Some(health) = check_disk(&disk) {
            results.push(health);
        }
    }

    results
}

/// Get list of available disks
fn get_disk_list() -> Vec<String> {
    let mut disks = Vec::new();

    // Check /dev/sd* devices
    if let Ok(entries) = std::fs::read_dir("/dev") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("sd") && name.len() == 3 {
                disks.push(format!("/dev/{}", name));
            } else if name.starts_with("nvme") && name.contains("n1") && !name.contains("p") {
                disks.push(format!("/dev/{}", name));
            }
        }
    }

    disks
}

/// Check SMART health for a specific disk
fn check_disk(device: &str) -> Option<DiskHealth> {
    debug!("Checking SMART health for {}", device);

    // Try smartctl command
    let output = std::process::Command::new("smartctl")
        .args(["-A", device])
        .output()
        .ok()?;

    if !output.status.success() {
        warn!("Failed to get SMART data for {}", device);
        return None;
    }

    let smart_data = String::from_utf8_lossy(&output.stdout);
    let attributes = parse_smart_attributes(&smart_data);

    // Determine health status
    let health_status = if attributes.iter().any(|a| a.failing) {
        HealthStatus::Failing
    } else if attributes.iter().any(|a| is_warning_attribute(a)) {
        HealthStatus::Warning
    } else if !attributes.is_empty() {
        HealthStatus::Healthy
    } else {
        HealthStatus::Unknown
    };

    // Predict failure based on critical attributes
    let predicted_failure = predict_failure_time(&attributes);

    Some(DiskHealth {
        device: device.to_string(),
        health_status,
        attributes,
        last_check: chrono::Utc::now().to_rfc3339(),
        predicted_failure_days: predicted_failure,
    })
}

/// Parse SMART attributes from smartctl output
fn parse_smart_attributes(output: &str) -> Vec<SmartAttribute> {
    let mut attributes = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 10 && parts[0].chars().all(|c| c.is_ascii_digit()) {
            if let Ok(id) = parts[0].parse::<u8>() {
                attributes.push(SmartAttribute {
                    id,
                    name: parts[1].to_string(),
                    current: parts[3].parse().unwrap_or(0),
                    worst: parts[4].parse().unwrap_or(0),
                    threshold: parts[5].parse().unwrap_or(0),
                    raw_value: parts[9].parse().unwrap_or(0),
                    failing: parts[8].contains("FAILING_NOW"),
                });
            }
        }
    }

    attributes
}

/// Check if attribute indicates warning condition
fn is_warning_attribute(attr: &SmartAttribute) -> bool {
    // Critical SMART attributes that indicate problems
    match attr.id {
        5 => attr.raw_value > 0,   // Reallocated Sectors Count
        10 => attr.raw_value > 0,  // Spin Retry Count
        184 => attr.raw_value > 0, // End-to-End Error
        187 => attr.raw_value > 0, // Reported Uncorrectable Errors
        188 => attr.raw_value > 0, // Command Timeout
        196 => attr.raw_value > 10, // Reallocation Event Count
        197 => attr.raw_value > 0, // Current Pending Sector Count
        198 => attr.raw_value > 0, // Uncorrectable Sector Count
        _ => attr.current < attr.threshold + 10,
    }
}

/// Predict disk failure time based on attribute trends
fn predict_failure_time(attributes: &[SmartAttribute]) -> Option<u32> {
    // Check for critical failing attributes
    for attr in attributes {
        match attr.id {
            5 if attr.raw_value > 5 => return Some(30),   // Reallocated sectors increasing
            197 if attr.raw_value > 0 => return Some(7),  // Pending sectors (imminent failure)
            198 if attr.raw_value > 0 => return Some(3),  // Uncorrectable sectors (very soon)
            _ => {}
        }
    }

    None
}

/// Generate health report
pub fn generate_health_report(health_checks: &[DiskHealth]) -> String {
    let mut report = String::new();

    report.push_str("## Disk Health Report\n\n");

    for disk in health_checks {
        match disk.health_status {
            HealthStatus::Healthy => {
                report.push_str(&format!("✓ {}: Healthy\n", disk.device));
            }
            HealthStatus::Warning => {
                report.push_str(&format!("⚠ {}: Warning - degraded performance detected\n", disk.device));

                // List concerning attributes
                for attr in &disk.attributes {
                    if is_warning_attribute(attr) {
                        report.push_str(&format!("  - {}: {} (threshold: {})\n",
                            attr.name, attr.current, attr.threshold));
                    }
                }
            }
            HealthStatus::Failing => {
                report.push_str(&format!("🔴 {}: FAILING - backup data immediately!\n", disk.device));

                if let Some(days) = disk.predicted_failure_days {
                    report.push_str(&format!("  Predicted failure in {} days\n", days));
                }
            }
            HealthStatus::Unknown => {
                report.push_str(&format!("? {}: Unknown (SMART data unavailable)\n", disk.device));
            }
        }
    }

    report
}
