//! Probe spine types (v0.0.193).

use serde::{Deserialize, Serialize};

/// Evidence kinds that can be gathered from the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Cpu,
    CpuTemperature,
    Memory,
    Disk,
    BlockDevices,
    Gpu,
    Audio,
    Network,
    Processes,
    Services,
    Journal,
    Packages,
    ToolExists,
    BootTime,
    System,
}

impl std::fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Cpu => "cpu",
            Self::CpuTemperature => "cpu_temperature",
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::BlockDevices => "block_devices",
            Self::Gpu => "gpu",
            Self::Audio => "audio",
            Self::Network => "network",
            Self::Processes => "processes",
            Self::Services => "services",
            Self::Journal => "journal",
            Self::Packages => "packages",
            Self::ToolExists => "tool_exists",
            Self::BootTime => "boot_time",
            Self::System => "system",
        };
        write!(f, "{}", s)
    }
}

/// Probe identifiers for system queries.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeId {
    Lscpu,
    Sensors,
    Free,
    Df,
    Lsblk,
    LspciAudio,
    PactlCards,
    IpAddr,
    TopMemory,
    TopCpu,
    FailedUnits,
    IsActive(String),
    JournalErrors,
    JournalWarnings,
    PacmanQ(String),
    PacmanCount,
    CommandV(String),
    SystemdAnalyze,
    Uname,
    Custom(String),
}

impl std::fmt::Display for ProbeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lscpu => write!(f, "lscpu"),
            Self::Sensors => write!(f, "sensors"),
            Self::Free => write!(f, "free"),
            Self::Df => write!(f, "df"),
            Self::Lsblk => write!(f, "lsblk"),
            Self::LspciAudio => write!(f, "lspci_audio"),
            Self::PactlCards => write!(f, "pactl_cards"),
            Self::IpAddr => write!(f, "ip_addr"),
            Self::TopMemory => write!(f, "top_memory"),
            Self::TopCpu => write!(f, "top_cpu"),
            Self::FailedUnits => write!(f, "failed_units"),
            Self::IsActive(s) => write!(f, "is_active:{}", s),
            Self::JournalErrors => write!(f, "journal_errors"),
            Self::JournalWarnings => write!(f, "journal_warnings"),
            Self::PacmanQ(p) => write!(f, "pacman_q:{}", p),
            Self::PacmanCount => write!(f, "pacman_count"),
            Self::CommandV(c) => write!(f, "command_v:{}", c),
            Self::SystemdAnalyze => write!(f, "systemd_analyze"),
            Self::Uname => write!(f, "uname"),
            Self::Custom(c) => write!(f, "custom:{}", c),
        }
    }
}

/// Route capability - what the deterministic path can/cannot do.
#[derive(Debug, Clone)]
pub struct RouteCapability {
    pub can_answer_deterministically: bool,
    pub required_evidence: Vec<EvidenceKind>,
    pub spine_probes: Vec<ProbeId>,
    pub evidence_required: bool,
}

impl Default for RouteCapability {
    fn default() -> Self {
        Self {
            can_answer_deterministically: false,
            required_evidence: vec![],
            spine_probes: vec![],
            evidence_required: true,
        }
    }
}

/// Urgency level for probe reduction decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Urgency {
    #[default]
    Normal,
    /// User explicitly asked for detailed info
    Detailed,
    /// Quick check, minimal probes
    Quick,
}
