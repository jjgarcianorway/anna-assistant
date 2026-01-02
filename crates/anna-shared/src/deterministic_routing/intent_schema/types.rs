//! Ticket Intent Schema Types - v0.0.439.
//!
//! Core type definitions for canonical intents, departments, and risk levels.

use serde::{Deserialize, Serialize};

/// Canonical intent types that map deterministically to departments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalIntent {
    // Performance intents
    BootPerf,
    MemStatus,
    CpuLoad,
    IoWait,

    // Storage intents
    DiskUsage,
    MountHealth,
    SmartStatus,
    BtrfsHealth,

    // Services intents
    SvcFailed,
    SvcHealth,
    SvcStatus,
    LogsRecentErrors,
    TimerStatus,

    // Network intents
    NetHealth,
    DnsHealth,
    WifiStatus,
    RouteStatus,

    // Hardware intents
    GpuInfo,
    GpuDriver,
    HardwareSensors,
    CpuInfo,
    AudioHealth,
    UsbDevices,
    PciDevices,

    // Desktop intents
    SessionDesktop,
    EditorConfig,
    ShellConfig,
    ThemeConfig,

    // Security intents
    SecurityFirewall,
    PermissionCheck,
    VulnCheck,

    // Package intents
    PkgInventory,
    PkgUpdates,
    PkgSearch,

    // Fallback
    Unknown,
}

impl CanonicalIntent {
    /// Parse from string (case-insensitive).
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().replace('-', "_").as_str() {
            "boot_perf" | "bootperf" | "boot" => Self::BootPerf,
            "mem_status" | "memstatus" | "memory" | "ram" => Self::MemStatus,
            "cpu_load" | "cpuload" => Self::CpuLoad,
            "io_wait" | "iowait" => Self::IoWait,

            "disk_usage" | "diskusage" | "disk" => Self::DiskUsage,
            "mount_health" | "mounthealth" | "mounts" => Self::MountHealth,
            "smart_status" | "smartstatus" | "smart" => Self::SmartStatus,
            "btrfs_health" | "btrfshealth" | "btrfs" => Self::BtrfsHealth,

            "svc_failed" | "svcfailed" | "failed_services" => Self::SvcFailed,
            "svc_health" | "svchealth" | "service_health" => Self::SvcHealth,
            "svc_status" | "svcstatus" | "service" => Self::SvcStatus,
            "logs_recent_errors" | "logsrecenterrors" | "errors" | "logs" => Self::LogsRecentErrors,
            "timer_status" | "timerstatus" | "timers" => Self::TimerStatus,

            "net_health" | "nethealth" | "network" => Self::NetHealth,
            "dns_health" | "dnshealth" | "dns" => Self::DnsHealth,
            "wifi_status" | "wifistatus" | "wifi" => Self::WifiStatus,
            "route_status" | "routestatus" | "routing" => Self::RouteStatus,

            "gpu_info" | "gpuinfo" | "gpu" => Self::GpuInfo,
            "gpu_driver" | "gpudriver" => Self::GpuDriver,
            "hardware_sensors" | "hardwaresensors" | "sensors" | "temperature" => {
                Self::HardwareSensors
            }
            "cpu_info" | "cpuinfo" => Self::CpuInfo,
            "audio_health" | "audiohealth" | "audio" | "sound" => Self::AudioHealth,
            "usb_devices" | "usbdevices" | "usb" => Self::UsbDevices,
            "pci_devices" | "pcidevices" | "pci" => Self::PciDevices,

            "session_desktop" | "sessiondesktop" | "desktop" => Self::SessionDesktop,
            "editor_config" | "editorconfig" | "editor" => Self::EditorConfig,
            "shell_config" | "shellconfig" | "shell" => Self::ShellConfig,
            "theme_config" | "themeconfig" | "theme" => Self::ThemeConfig,

            "security_firewall" | "securityfirewall" | "firewall" => Self::SecurityFirewall,
            "permission_check" | "permissioncheck" | "permissions" => Self::PermissionCheck,
            "vuln_check" | "vulncheck" | "vulnerabilities" => Self::VulnCheck,

            "pkg_inventory" | "pkginventory" | "packages" => Self::PkgInventory,
            "pkg_updates" | "pkgupdates" | "updates" => Self::PkgUpdates,
            "pkg_search" | "pkgsearch" => Self::PkgSearch,

            _ => Self::Unknown,
        }
    }

    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::BootPerf => "boot_perf",
            Self::MemStatus => "mem_status",
            Self::CpuLoad => "cpu_load",
            Self::IoWait => "io_wait",
            Self::DiskUsage => "disk_usage",
            Self::MountHealth => "mount_health",
            Self::SmartStatus => "smart_status",
            Self::BtrfsHealth => "btrfs_health",
            Self::SvcFailed => "svc_failed",
            Self::SvcHealth => "svc_health",
            Self::SvcStatus => "svc_status",
            Self::LogsRecentErrors => "logs_recent_errors",
            Self::TimerStatus => "timer_status",
            Self::NetHealth => "net_health",
            Self::DnsHealth => "dns_health",
            Self::WifiStatus => "wifi_status",
            Self::RouteStatus => "route_status",
            Self::GpuInfo => "gpu_info",
            Self::GpuDriver => "gpu_driver",
            Self::HardwareSensors => "hardware_sensors",
            Self::CpuInfo => "cpu_info",
            Self::AudioHealth => "audio_health",
            Self::UsbDevices => "usb_devices",
            Self::PciDevices => "pci_devices",
            Self::SessionDesktop => "session_desktop",
            Self::EditorConfig => "editor_config",
            Self::ShellConfig => "shell_config",
            Self::ThemeConfig => "theme_config",
            Self::SecurityFirewall => "security_firewall",
            Self::PermissionCheck => "permission_check",
            Self::VulnCheck => "vuln_check",
            Self::PkgInventory => "pkg_inventory",
            Self::PkgUpdates => "pkg_updates",
            Self::PkgSearch => "pkg_search",
            Self::Unknown => "unknown",
        }
    }
}

/// Department that handles a ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Department {
    Performance,
    Storage,
    Services,
    Network,
    Security,
    Hardware,
    Desktop,
}

impl Department {
    /// Parse from string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "performance" | "perf" => Some(Self::Performance),
            "storage" | "disk" => Some(Self::Storage),
            "services" | "service" | "svc" => Some(Self::Services),
            "network" | "net" => Some(Self::Network),
            "security" | "sec" => Some(Self::Security),
            "hardware" | "hw" => Some(Self::Hardware),
            "desktop" | "de" => Some(Self::Desktop),
            _ => None,
        }
    }

    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Performance => "Performance",
            Self::Storage => "Storage",
            Self::Services => "Services",
            Self::Network => "Network",
            Self::Security => "Security",
            Self::Hardware => "Hardware",
            Self::Desktop => "Desktop",
        }
    }
}

/// Risk level for the ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Read-only operations (probes, status checks).
    ReadOnly,
    /// Safe changes (enabling a service, changing a setting).
    SafeChange,
    /// Risky changes (format, delete, kernel updates).
    RiskyChange,
}

impl RiskLevel {
    /// Get label for display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::SafeChange => "safe_change",
            Self::RiskyChange => "risky_change",
        }
    }
}

impl Default for RiskLevel {
    fn default() -> Self {
        Self::ReadOnly
    }
}
