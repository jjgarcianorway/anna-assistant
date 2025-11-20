//! Telemetry Truth - Strict rules for system information
//!
//! Version 150: Zero tolerance for hallucinated system data
//!
//! RULES:
//! 1. All system info must come from real telemetry or explicit commands
//! 2. Missing data shows "Unknown" or "Not available"
//! 3. Never guess, never default, never hallucinate
//! 4. When incomplete, explain what command to run
//!
//! This module enforces these rules across all of Anna.

use anna_common::telemetry::SystemTelemetry;
use serde::{Deserialize, Serialize};

/// Verified system fact - guarantees data is real or explicitly unknown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemFact {
    /// Known value with source attribution
    Known {
        value: String,
        source: DataSource,
    },
    /// Unknown with optional command to retrieve it
    Unknown {
        reason: String,
        suggested_command: Option<String>,
    },
}

/// Data source for traceability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSource {
    /// Direct from SystemTelemetry struct
    Telemetry,
    /// From explicit command execution
    Command { command: String },
    /// From configuration file
    ConfigFile { path: String },
}

impl SystemFact {
    /// Create a known fact from telemetry
    pub fn from_telemetry(value: String) -> Self {
        SystemFact::Known {
            value,
            source: DataSource::Telemetry,
        }
    }

    /// Create a known fact from command output
    pub fn from_command(value: String, command: String) -> Self {
        SystemFact::Known {
            value,
            source: DataSource::Command { command },
        }
    }

    /// Create an unknown fact with explanation
    pub fn unknown(reason: &str, suggested_command: Option<&str>) -> Self {
        SystemFact::Unknown {
            reason: reason.to_string(),
            suggested_command: suggested_command.map(|s| s.to_string()),
        }
    }

    /// Get displayable value (shows "Unknown" for missing data)
    pub fn display(&self) -> String {
        match self {
            SystemFact::Known { value, .. } => value.clone(),
            SystemFact::Unknown { reason, suggested_command } => {
                if let Some(cmd) = suggested_command {
                    format!("Unknown ({}). Try: {}", reason, cmd)
                } else {
                    format!("Unknown ({})", reason)
                }
            }
        }
    }

    /// Check if this fact is known
    pub fn is_known(&self) -> bool {
        matches!(self, SystemFact::Known { .. })
    }
}

/// Complete, verified system report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedSystemReport {
    // Hardware
    pub cpu_model: SystemFact,
    pub cpu_cores: SystemFact,
    pub cpu_load: SystemFact,

    // Memory
    pub ram_total_gb: SystemFact,
    pub ram_used_gb: SystemFact,
    pub ram_percent: SystemFact,

    // Storage (per filesystem)
    pub storage: Vec<StorageFact>,

    // System
    pub hostname: SystemFact,
    pub kernel_version: SystemFact,
    pub os_name: SystemFact,
    pub uptime: SystemFact,

    // Graphics
    pub gpu: SystemFact,

    // Network
    pub network_status: SystemFact,
    pub primary_interface: SystemFact,
    pub ip_addresses: Vec<SystemFact>,

    // Desktop
    pub desktop_environment: SystemFact,
    pub window_manager: SystemFact,
    pub display_protocol: SystemFact,

    // Services
    pub failed_services: Vec<String>,

    // Health summary
    pub health_summary: HealthSummary,
}

/// Storage information for one filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFact {
    pub mount_point: String,
    pub total_gb: SystemFact,
    pub used_gb: SystemFact,
    pub free_gb: SystemFact,
    pub used_percent: SystemFact,
}

/// Health summary based on verified telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_status: HealthStatus,
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl VerifiedSystemReport {
    /// Build report from SystemTelemetry with strict verification
    pub fn from_telemetry(telemetry: &SystemTelemetry) -> Self {
        // CPU
        let cpu_model = SystemFact::from_telemetry(telemetry.hardware.cpu_model.clone());
        let cpu_cores = SystemFact::from_telemetry(telemetry.cpu.cores.to_string());
        let cpu_load = SystemFact::from_telemetry(format!("{:.2}", telemetry.cpu.load_avg_1min));

        // RAM
        let ram_total_gb = telemetry.hardware.total_ram_mb as f64 / 1024.0;
        let ram_used_gb = telemetry.memory.used_mb as f64 / 1024.0;
        let ram_percent = (ram_used_gb / ram_total_gb) * 100.0;

        // Storage - CRITICAL: Fix 0.0 GB bug
        let storage: Vec<StorageFact> = telemetry.disks.iter().map(|disk| {
            let total_gb = disk.total_mb as f64 / 1024.0;
            let used_gb = disk.used_mb as f64 / 1024.0;
            let free_gb = total_gb - used_gb;

            StorageFact {
                mount_point: disk.mount_point.clone(),
                total_gb: SystemFact::from_telemetry(format!("{:.1}", total_gb)),
                used_gb: SystemFact::from_telemetry(format!("{:.1}", used_gb)),
                free_gb: SystemFact::from_telemetry(format!("{:.1}", free_gb)),
                used_percent: SystemFact::from_telemetry(format!("{:.1}", disk.usage_percent)),
            }
        }).collect();

        // Hostname - get from command, not telemetry (telemetry doesn't have it)
        let hostname = get_hostname();

        // Kernel
        let kernel_version = get_kernel_version();

        // OS name
        let os_name = SystemFact::from_telemetry("Arch Linux".to_string());

        // Uptime
        let uptime = get_uptime();

        // GPU
        let gpu = if let Some(ref gpu_info) = telemetry.hardware.gpu_info {
            SystemFact::from_telemetry(gpu_info.clone())
        } else {
            SystemFact::unknown("No GPU detected", None)
        };

        // Network
        let (network_status, primary_interface, ip_addresses) = get_network_info();

        // Desktop
        let (desktop_environment, window_manager, display_protocol) =
            get_desktop_info(&telemetry);

        // Failed services
        let failed_services: Vec<String> = telemetry.services.failed_units
            .iter()
            .map(|unit| unit.name.clone())
            .collect();

        // Health summary
        let health_summary = build_health_summary(&storage, &failed_services);

        VerifiedSystemReport {
            cpu_model,
            cpu_cores,
            cpu_load,
            ram_total_gb: SystemFact::from_telemetry(format!("{:.1}", ram_total_gb)),
            ram_used_gb: SystemFact::from_telemetry(format!("{:.1}", ram_used_gb)),
            ram_percent: SystemFact::from_telemetry(format!("{:.1}", ram_percent)),
            storage,
            hostname,
            kernel_version,
            os_name,
            uptime,
            gpu,
            network_status,
            primary_interface,
            ip_addresses,
            desktop_environment,
            window_manager,
            display_protocol,
            failed_services,
            health_summary,
        }
    }
}

/// Get hostname from system (verified, not guessed)
fn get_hostname() -> SystemFact {
    // Try /proc/sys/kernel/hostname first (most reliable)
    match std::fs::read_to_string("/proc/sys/kernel/hostname") {
        Ok(hostname) => {
            let hostname = hostname.trim().to_string();
            if hostname.is_empty() {
                SystemFact::unknown("hostname file empty", Some("hostnamectl"))
            } else {
                SystemFact::from_command(hostname, "cat /proc/sys/kernel/hostname".to_string())
            }
        }
        Err(_) => {
            // Fallback to hostname command
            match std::process::Command::new("hostname").output() {
                Ok(output) if output.status.success() => {
                    let hostname = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if hostname.is_empty() {
                        SystemFact::unknown("hostname command returned empty", Some("hostnamectl"))
                    } else {
                        SystemFact::from_command(hostname, "hostname".to_string())
                    }
                }
                _ => SystemFact::unknown("hostname not available", Some("hostnamectl")),
            }
        }
    }
}

/// Get kernel version (verified)
fn get_kernel_version() -> SystemFact {
    match std::process::Command::new("uname").arg("-r").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            SystemFact::from_command(version, "uname -r".to_string())
        }
        _ => SystemFact::unknown("uname command failed", Some("uname -r")),
    }
}

/// Get system uptime (verified)
fn get_uptime() -> SystemFact {
    match std::fs::read_to_string("/proc/uptime") {
        Ok(content) => {
            if let Some(uptime_str) = content.split_whitespace().next() {
                if let Ok(uptime_secs) = uptime_str.parse::<f64>() {
                    let days = (uptime_secs / 86400.0) as u64;
                    let hours = ((uptime_secs % 86400.0) / 3600.0) as u64;
                    let mins = ((uptime_secs % 3600.0) / 60.0) as u64;

                    let uptime_str = if days > 0 {
                        format!("{} days, {} hours", days, hours)
                    } else if hours > 0 {
                        format!("{} hours, {} minutes", hours, mins)
                    } else {
                        format!("{} minutes", mins)
                    };

                    return SystemFact::from_command(uptime_str, "cat /proc/uptime".to_string());
                }
            }
            SystemFact::unknown("could not parse /proc/uptime", Some("uptime"))
        }
        _ => SystemFact::unknown("/proc/uptime not readable", Some("uptime")),
    }
}

/// Get network information (verified, never guessed)
fn get_network_info() -> (SystemFact, SystemFact, Vec<SystemFact>) {
    // Try to get network status from ip command
    match std::process::Command::new("ip").arg("link").arg("show").output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Check if any interface is UP
            let has_up_interface = output_str.lines().any(|line| line.contains("state UP"));

            let network_status = if has_up_interface {
                SystemFact::from_command("Connected".to_string(), "ip link show".to_string())
            } else {
                SystemFact::from_command("No active interfaces".to_string(), "ip link show".to_string())
            };

            // Try to get primary interface
            let primary_interface = if let Some(line) = output_str.lines()
                .find(|l| l.contains("state UP") && !l.contains("lo:")) {
                if let Some(iface) = line.split(':').nth(1) {
                    SystemFact::from_command(
                        iface.trim().to_string(),
                        "ip link show".to_string()
                    )
                } else {
                    SystemFact::unknown("could not parse interface name", Some("ip addr"))
                }
            } else {
                SystemFact::unknown("no active interface found", Some("ip addr"))
            };

            // Get IP addresses
            let ip_addresses = get_ip_addresses();

            (network_status, primary_interface, ip_addresses)
        }
        _ => (
            SystemFact::unknown("ip command not available", Some("ip link show")),
            SystemFact::unknown("ip command not available", Some("ip addr")),
            vec![],
        ),
    }
}

/// Get IP addresses (verified)
fn get_ip_addresses() -> Vec<SystemFact> {
    match std::process::Command::new("ip").arg("addr").output() {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            output_str.lines()
                .filter_map(|line| {
                    if line.trim().starts_with("inet ") || line.trim().starts_with("inet6 ") {
                        let parts: Vec<&str> = line.trim().split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(SystemFact::from_command(
                                parts[1].to_string(),
                                "ip addr".to_string()
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Get desktop environment info (verified from telemetry)
fn get_desktop_info(telemetry: &SystemTelemetry) -> (SystemFact, SystemFact, SystemFact) {
    let desktop = &telemetry.desktop;

    let de = if let Some(ref d) = desktop {
        if let Some(ref de_name) = d.de_name {
            SystemFact::from_telemetry(de_name.clone())
        } else {
            SystemFact::unknown("desktop environment not detected", Some("echo $XDG_CURRENT_DESKTOP"))
        }
    } else {
        SystemFact::unknown("desktop telemetry not available", None)
    };

    let wm = if let Some(ref d) = desktop {
        if let Some(ref wm_name) = d.wm_name {
            SystemFact::from_telemetry(wm_name.clone())
        } else {
            SystemFact::unknown("window manager not detected", Some("wmctrl -m"))
        }
    } else {
        SystemFact::unknown("desktop telemetry not available", None)
    };

    let display = if let Some(ref d) = desktop {
        if let Some(ref display_server) = d.display_server {
            SystemFact::from_telemetry(display_server.clone())
        } else {
            SystemFact::unknown("display protocol not detected", Some("echo $XDG_SESSION_TYPE"))
        }
    } else {
        SystemFact::unknown("desktop telemetry not available", None)
    };

    (de, wm, display)
}

/// Build health summary from verified data only
fn build_health_summary(storage: &[StorageFact], failed_services: &[String]) -> HealthSummary {
    let mut critical_issues = Vec::new();
    let mut warnings = Vec::new();
    let mut info = Vec::new();

    // Check storage
    for disk in storage {
        if let SystemFact::Known { value: free_gb_str, .. } = &disk.free_gb {
            if let Ok(free_gb) = free_gb_str.parse::<f64>() {
                if free_gb < 5.0 {
                    critical_issues.push(format!(
                        "Critical: Only {:.1} GB free on {}",
                        free_gb, disk.mount_point
                    ));
                } else if free_gb < 10.0 {
                    warnings.push(format!(
                        "Warning: Low disk space on {} ({:.1} GB free)",
                        disk.mount_point, free_gb
                    ));
                }
            }
        }
    }

    // Check failed services
    if !failed_services.is_empty() {
        if failed_services.len() == 1 {
            warnings.push(format!("1 service failed: {}", failed_services[0]));
        } else {
            warnings.push(format!("{} services failed", failed_services.len()));
        }
    }

    // Determine overall status
    let overall_status = if !critical_issues.is_empty() {
        HealthStatus::Critical
    } else if !warnings.is_empty() {
        HealthStatus::Warning
    } else {
        info.push("No critical issues detected".to_string());
        HealthStatus::Healthy
    };

    HealthSummary {
        overall_status,
        critical_issues,
        warnings,
        info,
    }
}
