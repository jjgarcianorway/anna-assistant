//! Evidence kinds and detection (v0.0.259).
//!
//! v0.0.259: Added LoadAverage and NetworkInfo for fast path.

use serde::{Deserialize, Serialize};

/// Parsed evidence kinds present in the response (v0.45.4: extended)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Memory,
    Disk,
    BlockDevices,
    Cpu,
    /// CPU temperature (sensors)
    CpuTemperature,
    Services,
    /// Journal errors/warnings (v0.0.34)
    Journal,
    /// Failed systemd units (v0.0.34)
    FailedUnits,
    /// Audio/sound cards (v0.45.4)
    Audio,
    /// Network interfaces (v0.45.4)
    Network,
    /// Running processes (v0.45.4)
    Processes,
    /// Installed packages (v0.45.4)
    Packages,
    /// Tool existence check (v0.45.4)
    ToolExists,
    /// Boot time (v0.45.4)
    BootTime,
    /// System info (uname) (v0.45.4)
    System,
    /// v0.0.259: Load average (uptime)
    LoadAverage,
    /// v0.0.259: Network connectivity info
    NetworkInfo,
}

impl std::fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Memory => "memory",
            Self::Disk => "disk",
            Self::BlockDevices => "block_devices",
            Self::Cpu => "cpu",
            Self::CpuTemperature => "cpu_temperature",
            Self::Services => "services",
            Self::Journal => "journal",
            Self::FailedUnits => "failed_units",
            Self::Audio => "audio",
            Self::Network => "network",
            Self::Processes => "processes",
            Self::Packages => "packages",
            Self::ToolExists => "tool_exists",
            Self::BootTime => "boot_time",
            Self::System => "system",
            Self::LoadAverage => "load_average",
            Self::NetworkInfo => "network_info",
        };
        write!(f, "{}", s)
    }
}

/// Derive evidence kinds from route class name (legacy, use evidence_kinds_from_probes for truthfulness)
pub fn evidence_kinds_from_route(route_class: &str) -> Vec<EvidenceKind> {
    match route_class {
        "memory_usage" | "ram_info" => vec![EvidenceKind::Memory],
        "disk_usage" | "disk_space" => vec![EvidenceKind::Disk],
        "cpu_info" => vec![EvidenceKind::Cpu],
        "service_status" => vec![EvidenceKind::Services],
        "system_health_summary" => vec![
            EvidenceKind::Memory,
            EvidenceKind::Disk,
            EvidenceKind::BlockDevices,
            EvidenceKind::Cpu,
        ],
        "system_triage" => vec![EvidenceKind::Journal, EvidenceKind::FailedUnits],
        _ => vec![],
    }
}

/// Derive evidence kinds from ACTUAL probe results (v0.45.4: extended detection)
pub fn evidence_kinds_from_probes(probe_results: &[crate::rpc::ProbeResult]) -> Vec<EvidenceKind> {
    let mut kinds = Vec::new();
    for probe in probe_results {
        if probe.exit_code != 0 {
            continue;
        }
        let cmd = probe.command.to_lowercase();
        // Memory probes
        if cmd.contains("free") {
            kinds.push(EvidenceKind::Memory);
        }
        // Disk probes
        if cmd.contains("df ") || cmd.starts_with("df") {
            kinds.push(EvidenceKind::Disk);
        }
        if cmd.contains("lsblk") {
            kinds.push(EvidenceKind::BlockDevices);
        }
        // CPU probes
        if cmd.contains("lscpu") {
            kinds.push(EvidenceKind::Cpu);
        }
        if cmd.contains("sensors") {
            kinds.push(EvidenceKind::CpuTemperature);
        }
        // Service probes
        if cmd.contains("systemctl") && !cmd.contains("--failed") {
            kinds.push(EvidenceKind::Services);
        }
        if cmd.contains("--failed") {
            kinds.push(EvidenceKind::FailedUnits);
        }
        // Journal probes
        if cmd.contains("journalctl") {
            kinds.push(EvidenceKind::Journal);
        }
        // Audio probes (v0.45.4)
        if cmd.contains("lspci") && cmd.contains("audio") {
            kinds.push(EvidenceKind::Audio);
        }
        if cmd.contains("pactl") {
            kinds.push(EvidenceKind::Audio);
        }
        // Network probes (v0.45.4)
        if cmd.contains("ip addr") || cmd.contains("ip route") {
            kinds.push(EvidenceKind::Network);
        }
        // Process probes (v0.45.4)
        if cmd.contains("ps aux") {
            kinds.push(EvidenceKind::Processes);
        }
        // Package probes (v0.45.4)
        if cmd.contains("pacman") {
            kinds.push(EvidenceKind::Packages);
        }
        // Tool existence probes (v0.45.4)
        if cmd.contains("command -v") || cmd.contains("which ") {
            kinds.push(EvidenceKind::ToolExists);
        }
        // Boot time probes (v0.45.4)
        if cmd.contains("systemd-analyze") {
            kinds.push(EvidenceKind::BootTime);
        }
        // System info probes (v0.45.4)
        if cmd.contains("uname") {
            kinds.push(EvidenceKind::System);
        }
    }
    kinds.sort_by_key(|k| format!("{:?}", k));
    kinds.dedup();
    kinds
}
