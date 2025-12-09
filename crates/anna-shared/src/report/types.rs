//! Report types (v0.0.189).

use crate::reliability::ReliabilityExplanation;
use crate::trace::{EvidenceKind, ProbeStats};
use crate::parsers::{BlockDevice, CpuInfo, DiskUsage, MemoryInfo, ServiceStatus};
use serde::{Deserialize, Serialize};

/// Severity level for health items (stable ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    Ok = 0,
    Warning = 1,
    Critical = 2,
}

impl std::fmt::Display for HealthSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::Warning => write!(f, "WARNING"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A single health check result with supporting evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    /// Unique identifier (e.g., "disk_root", "memory", "services")
    pub id: String,
    /// Severity level
    pub severity: HealthSeverity,
    /// Human-readable title (e.g., "Root disk usage critical")
    pub title: String,
    /// Evidence claim (e.g., "95% used (47.5 GB / 50 GB)")
    pub claim: String,
}

/// System inventory (hardware summary)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInventory {
    pub cpu_model: Option<String>,
    pub cpu_cores: Option<u32>,
    pub memory_total_bytes: Option<u64>,
    pub block_device_count: usize,
    /// Summary like "2 disks, 3 partitions"
    pub block_device_summary: String,
}

/// Complete report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemReport {
    /// 2-3 bullet points summary
    pub executive_summary: Vec<String>,
    /// Hardware inventory
    pub inventory: SystemInventory,
    /// Health checks sorted by (severity desc, id asc)
    pub health_checks: Vec<HealthItem>,
    /// Probe execution stats
    pub probe_stats: ProbeStats,
    /// Evidence kinds collected
    pub evidence_kinds: Vec<EvidenceKind>,
    /// One-line execution trace summary
    pub execution_trace_summary: String,
    /// Reliability score
    pub reliability_score: u8,
    /// Explanation if score < 80
    pub reliability_explanation: Option<ReliabilityExplanation>,
}

/// Evidence collection for report generation
#[derive(Debug, Clone, Default)]
pub struct ReportEvidence {
    pub memory: Option<MemoryInfo>,
    pub disks: Vec<DiskUsage>,
    pub block_devices: Vec<BlockDevice>,
    pub cpu: Option<CpuInfo>,
    pub failed_services: Vec<ServiceStatus>,
}
