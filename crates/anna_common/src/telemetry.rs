//! Telemetry - System state data collected by annad
//!
//! Defines the data structures that annad collects and annactl queries
//! All data stays local, no network exfiltration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use sysinfo::System;

/// Complete system telemetry snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetry {
    /// When this snapshot was taken
    pub timestamp: DateTime<Utc>,

    /// Hardware information
    pub hardware: HardwareInfo,

    /// Disk usage and health
    pub disks: Vec<DiskInfo>,

    /// Memory usage
    pub memory: MemoryInfo,

    /// CPU information and usage
    pub cpu: CpuInfo,

    /// Package management state
    pub packages: PackageInfo,

    /// Systemd service health
    pub services: ServiceInfo,

    /// Network configuration
    pub network: NetworkInfo,

    /// Security posture indicators
    pub security: SecurityInfo,

    /// Desktop environment info
    pub desktop: Option<DesktopInfo>,

    /// Boot performance
    pub boot: Option<BootInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// CPU model name
    pub cpu_model: String,

    /// Total RAM in MB
    pub total_ram_mb: u64,

    /// Machine type (laptop, desktop, server)
    pub machine_type: MachineType,

    /// Has battery
    pub has_battery: bool,

    /// Has GPU
    pub has_gpu: bool,

    /// GPU info if available
    pub gpu_info: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineType {
    Laptop,
    Desktop,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Mount point
    pub mount_point: String,

    /// Total size in MB
    pub total_mb: u64,

    /// Used space in MB
    pub used_mb: u64,

    /// Usage percentage
    pub usage_percent: f64,

    /// Filesystem type
    pub fs_type: String,

    /// SMART status if available
    pub smart_status: Option<SmartStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmartStatus {
    Healthy,
    Warning { message: String },
    Critical { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total RAM in MB
    pub total_mb: u64,

    /// Available RAM in MB
    pub available_mb: u64,

    /// Used RAM in MB
    pub used_mb: u64,

    /// Swap total in MB
    pub swap_total_mb: u64,

    /// Swap used in MB
    pub swap_used_mb: u64,

    /// Memory usage percentage
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Number of cores
    pub cores: u32,

    /// Current average load (1 minute)
    pub load_avg_1min: f64,

    /// Current average load (5 minute)
    pub load_avg_5min: f64,

    /// CPU usage percentage (recent)
    pub usage_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Total installed packages
    pub total_installed: u64,

    /// Packages with updates available
    pub updates_available: u64,

    /// Orphaned packages
    pub orphaned: u64,

    /// Pacman cache size in MB
    pub cache_size_mb: f64,

    /// Last full system update
    pub last_update: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Total systemd units
    pub total_units: u64,

    /// Failed units
    pub failed_units: Vec<FailedUnit>,

    /// Recently restarted services
    pub recently_restarted: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedUnit {
    /// Unit name
    pub name: String,

    /// Unit type (service, timer, etc.)
    pub unit_type: String,

    /// When it failed
    pub failed_since: Option<DateTime<Utc>>,

    /// Brief failure message
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Has active connection
    pub is_connected: bool,

    /// Primary interface name
    pub primary_interface: Option<String>,

    /// Firewall active
    pub firewall_active: bool,

    /// Firewall type (ufw, iptables, nftables)
    pub firewall_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    /// Failed SSH login attempts (recent)
    pub failed_ssh_attempts: u64,

    /// Unattended upgrades enabled
    pub auto_updates_enabled: bool,

    /// Security audit warnings
    pub audit_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopInfo {
    /// Desktop environment name
    pub de_name: Option<String>,

    /// Window manager name
    pub wm_name: Option<String>,

    /// Display server (X11, Wayland)
    pub display_server: Option<String>,

    /// Number of monitors
    pub monitor_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootInfo {
    /// Last boot time in seconds
    pub last_boot_time_secs: f64,

    /// Average boot time in seconds (last 10 boots)
    pub avg_boot_time_secs: Option<f64>,

    /// Boot time trend (improving, degrading, stable)
    pub trend: Option<BootTrend>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootTrend {
    Improving,
    Stable,
    Degrading,
}

/// Telemetry query interface
/// This is what annactl uses to request telemetry from annad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryQuery {
    /// Get complete current snapshot
    CurrentSnapshot,

    /// Get snapshot from specific time
    HistoricalSnapshot { timestamp: DateTime<Utc> },

    /// Get metric history
    MetricHistory {
        metric: MetricType,
        days_back: u32,
    },

    /// Get change summary since timestamp
    ChangesSince { since: DateTime<Utc> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    BootTime,
    MemoryUsage,
    DiskUsage,
    CpuLoad,
    FailedServices,
}

/// Telemetry response from annad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryResponse {
    Snapshot(SystemTelemetry),
    MetricHistory(Vec<MetricPoint>),
    ChangesSummary(ChangesSummary),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesSummary {
    /// Packages installed since timestamp
    pub packages_installed: Vec<String>,

    /// Packages removed since timestamp
    pub packages_removed: Vec<String>,

    /// Services changed state
    pub services_changed: Vec<String>,

    /// Config files modified
    pub configs_modified: Vec<String>,

    /// Boot time delta
    pub boot_time_delta_secs: Option<f64>,

    /// Memory usage delta
    pub memory_delta_mb: Option<f64>,
}

impl SystemTelemetry {
    /// Create a minimal snapshot for testing
    pub fn minimal() -> Self {
        Self {
            timestamp: Utc::now(),
            hardware: HardwareInfo {
                cpu_model: "Unknown CPU".to_string(),
                total_ram_mb: 0,
                machine_type: MachineType::Desktop,
                has_battery: false,
                has_gpu: false,
                gpu_info: None,
            },
            disks: Vec::new(),
            memory: MemoryInfo {
                total_mb: 0,
                available_mb: 0,
                used_mb: 0,
                swap_total_mb: 0,
                swap_used_mb: 0,
                usage_percent: 0.0,
            },
            cpu: CpuInfo {
                cores: 1,
                load_avg_1min: 0.0,
                load_avg_5min: 0.0,
                usage_percent: None,
            },
            packages: PackageInfo {
                total_installed: 0,
                updates_available: 0,
                orphaned: 0,
                cache_size_mb: 0.0,
                last_update: None,
            },
            services: ServiceInfo {
                total_units: 0,
                failed_units: Vec::new(),
                recently_restarted: Vec::new(),
            },
            network: NetworkInfo {
                is_connected: true,
                primary_interface: None,
                firewall_active: false,
                firewall_type: None,
            },
            security: SecurityInfo {
                failed_ssh_attempts: 0,
                auto_updates_enabled: false,
                audit_warnings: Vec::new(),
            },
            desktop: None,
            boot: None,
        }
    }

    /// Collect a live system snapshot (Task 8: Deep Caretaker v0.1)
    /// Fast, read-only telemetry collection (completes in <1 second)
    pub fn collect() -> Self {
        use crate::profile::MachineProfile;

        Self {
            timestamp: Utc::now(),
            hardware: collect_hardware_info(),
            disks: collect_disk_info(),
            memory: collect_memory_info(),
            cpu: collect_cpu_info(),
            packages: collect_package_info(),
            services: collect_service_info(),
            network: collect_network_info(),
            security: collect_security_info(),
            desktop: collect_desktop_info(),
            boot: None, // Boot info collection is expensive, skip for now
        }
    }
}

// Task 8: Telemetry collection functions (read-only, fast, safe)

fn collect_hardware_info() -> HardwareInfo {
    use std::process::Command;
    use crate::profile::MachineProfile;

    let profile = MachineProfile::detect();
    let machine_type = match profile {
        MachineProfile::Laptop => MachineType::Laptop,
        MachineProfile::Desktop => MachineType::Desktop,
        MachineProfile::ServerLike => MachineType::Server,
        MachineProfile::Unknown => MachineType::Desktop,
    };

    // Get CPU model
    let cpu_model = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("model name"))
                .and_then(|line| line.split(':').nth(1))
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Unknown CPU".to_string());

    // Get total RAM
    let total_ram_mb = System::new_all()
        .total_memory() / 1024 / 1024;

    // Check for battery
    let has_battery = std::path::Path::new("/sys/class/power_supply/BAT0").exists()
        || std::path::Path::new("/sys/class/power_supply/BAT1").exists();

    // Check for GPU (simple detection)
    let has_gpu = Command::new("lspci")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.contains("VGA") || s.contains("3D"))
        .unwrap_or(false);

    let gpu_info = if has_gpu {
        Command::new("lspci")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| {
                s.lines()
                    .find(|line| line.contains("VGA") || line.contains("3D"))
                    .map(|line| line.to_string())
            })
    } else {
        None
    };

    HardwareInfo {
        cpu_model,
        total_ram_mb,
        machine_type,
        has_battery,
        has_gpu,
        gpu_info,
    }
}

fn collect_disk_info() -> Vec<DiskInfo> {
    use std::process::Command;

    let output = Command::new("df")
        .args(&["-h", "--output=target,size,used,pcent,fstype"])
        .output()
        .ok();

    if let Some(output) = output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            return stdout
                .lines()
                .skip(1) // Skip header
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let mount_point = parts[0].to_string();
                        // Only report /, /home, /var
                        if mount_point == "/" || mount_point == "/home" || mount_point == "/var" {
                            let size_str = parts[1];
                            let used_str = parts[2];
                            let percent_str = parts[3].trim_end_matches('%');
                            let fs_type = parts[4].to_string();

                            let total_mb = parse_size_to_mb(size_str);
                            let used_mb = parse_size_to_mb(used_str);
                            let usage_percent = percent_str.parse::<f64>().unwrap_or(0.0);

                            return Some(DiskInfo {
                                mount_point,
                                total_mb,
                                used_mb,
                                usage_percent,
                                fs_type,
                                smart_status: None, // SMART requires elevated permissions
                            });
                        }
                    }
                    None
                })
                .collect();
        }
    }

    Vec::new()
}

fn parse_size_to_mb(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    if let Some(num_str) = size_str.chars().take_while(|c| c.is_numeric() || *c == '.').collect::<String>().parse::<f64>().ok() {
        if size_str.ends_with('G') {
            return (num_str * 1024.0) as u64;
        } else if size_str.ends_with('M') {
            return num_str as u64;
        } else if size_str.ends_with('K') {
            return (num_str / 1024.0) as u64;
        } else if size_str.ends_with('T') {
            return (num_str * 1024.0 * 1024.0) as u64;
        }
    }
    0
}

fn collect_memory_info() -> MemoryInfo {
    let mut sys = System::new();
    sys.refresh_memory();

    let total_mb = sys.total_memory() / 1024 / 1024;
    let available_mb = sys.available_memory() / 1024 / 1024;
    let used_mb = sys.used_memory() / 1024 / 1024;
    let swap_total_mb = sys.total_swap() / 1024 / 1024;
    let swap_used_mb = sys.used_swap() / 1024 / 1024;

    let usage_percent = if total_mb > 0 {
        (used_mb as f64 / total_mb as f64) * 100.0
    } else {
        0.0
    };

    MemoryInfo {
        total_mb,
        available_mb,
        used_mb,
        swap_total_mb,
        swap_used_mb,
        usage_percent,
    }
}

fn collect_cpu_info() -> CpuInfo {
    let mut sys = System::new();
    sys.refresh_cpu();

    let cores = sys.cpus().len() as u32;

    // Get load average from /proc/loadavg
    let (load_avg_1min, load_avg_5min) = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|content| {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 2 {
                let one_min = parts[0].parse::<f64>().ok()?;
                let five_min = parts[1].parse::<f64>().ok()?;
                Some((one_min, five_min))
            } else {
                None
            }
        })
        .unwrap_or((0.0, 0.0));

    CpuInfo {
        cores,
        load_avg_1min,
        load_avg_5min,
        usage_percent: None, // Would need sampling over time
    }
}

fn collect_package_info() -> PackageInfo {
    use std::process::Command;

    // Count installed packages
    let total_installed = Command::new("pacman")
        .args(&["-Q"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    // Count orphaned packages
    let orphaned = Command::new("pacman")
        .args(&["-Qtdq"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    // Get cache size
    let cache_size_mb = Command::new("du")
        .args(&["-sm", "/var/cache/pacman/pkg"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|num| num.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    // Check for updates (fast check, don't sync)
    let updates_available = 0; // Skip for now, requires pacman -Qu which may be slow

    PackageInfo {
        total_installed,
        updates_available,
        orphaned,
        cache_size_mb,
        last_update: None, // Would need to parse pacman logs
    }
}

fn collect_service_info() -> ServiceInfo {
    use std::process::Command;

    let mut failed_units = Vec::new();

    // Get failed services
    if let Ok(output) = Command::new("systemctl")
        .args(&["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let name = parts[0].to_string();
                    let unit_type = if name.ends_with(".service") {
                        "service".to_string()
                    } else if name.ends_with(".timer") {
                        "timer".to_string()
                    } else {
                        "unit".to_string()
                    };

                    failed_units.push(FailedUnit {
                        name,
                        unit_type,
                        failed_since: None,
                        message: None,
                    });
                }
            }
        }
    }

    // Count total units
    let total_units = Command::new("systemctl")
        .args(&["list-units", "--all", "--no-pager", "--no-legend"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.lines().count() as u64)
        .unwrap_or(0);

    ServiceInfo {
        total_units,
        failed_units,
        recently_restarted: Vec::new(), // Would need journalctl parsing
    }
}

fn collect_network_info() -> NetworkInfo {
    use std::process::Command;

    // Simple connectivity check
    let is_connected = Command::new("ip")
        .args(&["route"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.contains("default"))
        .unwrap_or(false);

    // Get primary interface
    let primary_interface = Command::new("ip")
        .args(&["route", "show", "default"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| {
            s.split_whitespace()
                .skip_while(|&w| w != "dev")
                .nth(1)
                .map(|s| s.to_string())
        });

    // Check firewall (simplified)
    let firewall_active = std::process::Command::new("systemctl")
        .args(&["is-active", "ufw"])
        .output()
        .ok()
        .map(|output| output.status.success())
        .unwrap_or(false);

    let firewall_type = if firewall_active {
        Some("ufw".to_string())
    } else {
        None
    };

    NetworkInfo {
        is_connected,
        primary_interface,
        firewall_active,
        firewall_type,
    }
}

fn collect_security_info() -> SecurityInfo {
    SecurityInfo {
        failed_ssh_attempts: 0, // Would need journalctl parsing
        auto_updates_enabled: false, // Would need to check specific services
        audit_warnings: Vec::new(),
    }
}

fn collect_desktop_info() -> Option<DesktopInfo> {
    use std::env;

    // Check if we're in a desktop session
    let display_server = if env::var("WAYLAND_DISPLAY").is_ok() {
        Some("Wayland".to_string())
    } else if env::var("DISPLAY").is_ok() {
        Some("X11".to_string())
    } else {
        None
    };

    if display_server.is_some() {
        let de_name = env::var("XDG_CURRENT_DESKTOP").ok();
        let wm_name = env::var("XDG_SESSION_DESKTOP").ok();

        Some(DesktopInfo {
            de_name,
            wm_name,
            display_server,
            monitor_count: 1, // Would need xrandr or similar
        })
    } else {
        None
    }
}
