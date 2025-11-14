//! Telemetry - System state data collected by annad
//!
//! Defines the data structures that annad collects and annactl queries
//! All data stays local, no network exfiltration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

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
}
