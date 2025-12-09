//! Type definitions for answer contract module (v0.0.209).

use serde::{Deserialize, Serialize};

/// Verbosity level for answers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verbosity {
    /// Minimal: only the exact answer requested
    Minimal,
    /// Normal: answer with brief context
    #[default]
    Normal,
    /// Teach: explain reasoning and provide educational context
    Teach,
}

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verbosity::Minimal => write!(f, "minimal"),
            Verbosity::Normal => write!(f, "normal"),
            Verbosity::Teach => write!(f, "teach"),
        }
    }
}

/// Requested field type - what the user asked for
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestedField {
    /// CPU core count only
    CpuCores,
    /// CPU model name only
    CpuModel,
    /// CPU temperature
    CpuTemp,
    /// Total RAM
    RamTotal,
    /// Free/available RAM
    RamFree,
    /// Used RAM
    RamUsed,
    /// Disk usage (specific mount or all)
    DiskUsage(Option<String>),
    /// Disk free space
    DiskFree(Option<String>),
    /// Sound card / audio device
    SoundCard,
    /// GPU info
    GpuInfo,
    /// Network interfaces
    NetworkInterfaces,
    /// Service status (specific service)
    ServiceStatus(String),
    /// Process list (top by resource)
    ProcessList,
    /// Package count
    PackageCount,
    /// Tool existence check
    ToolExists(String),
    /// Generic query (needs full answer)
    Generic,
}

impl std::fmt::Display for RequestedField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestedField::CpuCores => write!(f, "cpu_cores"),
            RequestedField::CpuModel => write!(f, "cpu_model"),
            RequestedField::CpuTemp => write!(f, "cpu_temp"),
            RequestedField::RamTotal => write!(f, "ram_total"),
            RequestedField::RamFree => write!(f, "ram_free"),
            RequestedField::RamUsed => write!(f, "ram_used"),
            RequestedField::DiskUsage(m) => {
                if let Some(mount) = m {
                    write!(f, "disk_usage:{}", mount)
                } else {
                    write!(f, "disk_usage")
                }
            }
            RequestedField::DiskFree(m) => {
                if let Some(mount) = m {
                    write!(f, "disk_free:{}", mount)
                } else {
                    write!(f, "disk_free")
                }
            }
            RequestedField::SoundCard => write!(f, "sound_card"),
            RequestedField::GpuInfo => write!(f, "gpu_info"),
            RequestedField::NetworkInterfaces => write!(f, "network_interfaces"),
            RequestedField::ServiceStatus(s) => write!(f, "service_status:{}", s),
            RequestedField::ProcessList => write!(f, "process_list"),
            RequestedField::PackageCount => write!(f, "package_count"),
            RequestedField::ToolExists(t) => write!(f, "tool_exists:{}", t),
            RequestedField::Generic => write!(f, "generic"),
        }
    }
}
