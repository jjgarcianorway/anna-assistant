//! ParsedProbeData enum and implementations (v0.0.173).

use serde::{Deserialize, Serialize};

use super::atoms::ParseError;
use super::df::DiskUsage;
use super::evidence::{AudioDevices, PackageInstalled, ToolExists};
use super::free::MemoryInfo;
use super::journalctl::{BootTimeInfo, JournalSummary};
use super::lsblk::BlockDevice;
use super::lscpu::CpuInfo;
use super::systemctl::ServiceStatus;

/// Parsed probe data or error.
/// Used internally for enrichment; not serialized over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedProbeData {
    /// Memory info from `free -h`
    Memory(MemoryInfo),
    /// Disk usage from `df -h`
    Disk(Vec<DiskUsage>),
    /// Service status list (from `systemctl --failed` or similar)
    Services(Vec<ServiceStatus>),
    /// Single service status (from `systemctl is-active`)
    Service(ServiceStatus),
    /// Block devices from `lsblk` (v0.0.22 STRUCT+)
    BlockDevices(Vec<BlockDevice>),
    /// CPU info from `lscpu` (v0.0.22 STRUCT+)
    Cpu(CpuInfo),
    /// Journal errors from `journalctl -p 3` (v0.0.35)
    JournalErrors(JournalSummary),
    /// Journal warnings from `journalctl -p 4` (v0.0.35)
    JournalWarnings(JournalSummary),
    /// Boot time from `systemd-analyze` (v0.0.35)
    BootTime(BootTimeInfo),
    /// Tool existence check (v0.45.7) - exit 1 = valid negative evidence
    Tool(ToolExists),
    /// Package installation check (v0.45.7) - exit 1 = valid negative evidence
    Package(PackageInstalled),
    /// Audio devices from lspci/pactl (v0.45.8)
    Audio(AudioDevices),
    /// Parse error with diagnostic context
    Error(ParseError),
    /// Probe type not supported for structured parsing
    Unsupported,
}

impl ParsedProbeData {
    /// Check if this is an error variant.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Get the error if this is an error variant.
    pub fn as_error(&self) -> Option<&ParseError> {
        match self {
            Self::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Get memory info if this is the Memory variant.
    pub fn as_memory(&self) -> Option<&MemoryInfo> {
        match self {
            Self::Memory(m) => Some(m),
            _ => None,
        }
    }

    /// Get disk usage if this is the Disk variant.
    pub fn as_disk(&self) -> Option<&Vec<DiskUsage>> {
        match self {
            Self::Disk(d) => Some(d),
            _ => None,
        }
    }

    /// Get services if this is the Services variant.
    pub fn as_services(&self) -> Option<&Vec<ServiceStatus>> {
        match self {
            Self::Services(s) => Some(s),
            _ => None,
        }
    }

    /// Get single service if this is the Service variant.
    pub fn as_service(&self) -> Option<&ServiceStatus> {
        match self {
            Self::Service(s) => Some(s),
            _ => None,
        }
    }

    /// Get block devices if this is the BlockDevices variant.
    pub fn as_block_devices(&self) -> Option<&Vec<BlockDevice>> {
        match self {
            Self::BlockDevices(b) => Some(b),
            _ => None,
        }
    }

    /// Get CPU info if this is the Cpu variant.
    pub fn as_cpu(&self) -> Option<&CpuInfo> {
        match self {
            Self::Cpu(c) => Some(c),
            _ => None,
        }
    }

    /// Get tool existence if this is the Tool variant (v0.45.7).
    pub fn as_tool(&self) -> Option<&ToolExists> {
        match self {
            Self::Tool(t) => Some(t),
            _ => None,
        }
    }

    /// Get package installation if this is the Package variant (v0.45.7).
    pub fn as_package(&self) -> Option<&PackageInstalled> {
        match self {
            Self::Package(p) => Some(p),
            _ => None,
        }
    }

    /// Get audio devices if this is the Audio variant (v0.45.8).
    pub fn as_audio(&self) -> Option<&AudioDevices> {
        match self {
            Self::Audio(a) => Some(a),
            _ => None,
        }
    }

    /// Check if this represents valid evidence (not error/unsupported) (v0.45.7).
    pub fn is_valid_evidence(&self) -> bool {
        !matches!(self, Self::Error(_) | Self::Unsupported)
    }
}
