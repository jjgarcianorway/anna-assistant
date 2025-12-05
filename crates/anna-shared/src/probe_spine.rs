//! Probe spine: deterministic tool selection and evidence requirements.
//!
//! This module enforces minimum probes when evidence is required,
//! preventing "no probes, no evidence, but claims anyway" scenarios.
//!
//! v0.45.x stabilization: LLM-first, probe spine, no fake answers.

use serde::{Deserialize, Serialize};

/// Evidence kinds that can be gathered from the system.
/// Each kind maps to one or more probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// CPU information (model, cores, threads)
    Cpu,
    /// CPU temperature from sensors
    CpuTemperature,
    /// Memory usage (total, used, available)
    Memory,
    /// Disk/filesystem usage
    Disk,
    /// Block devices (lsblk)
    BlockDevices,
    /// GPU information
    Gpu,
    /// Audio/sound hardware
    Audio,
    /// Network interfaces and addresses
    Network,
    /// Running processes (top)
    Processes,
    /// Systemd services and units
    Services,
    /// System journal (errors, warnings)
    Journal,
    /// Installed packages (count, list)
    Packages,
    /// Specific tool/command availability
    ToolExists,
    /// Boot time and uptime
    BootTime,
    /// Kernel and OS info
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
    /// lscpu - CPU info
    Lscpu,
    /// sensors - temperature readings
    Sensors,
    /// free - memory usage
    Free,
    /// df - filesystem usage
    Df,
    /// lsblk - block devices
    Lsblk,
    /// lspci | grep -i audio - audio hardware
    LspciAudio,
    /// pactl list cards - PulseAudio cards
    PactlCards,
    /// ip addr - network addresses
    IpAddr,
    /// ps aux --sort=-%mem - top memory processes
    TopMemory,
    /// ps aux --sort=-%cpu - top CPU processes
    TopCpu,
    /// systemctl --failed - failed units
    FailedUnits,
    /// systemctl is-active <service> - service status
    IsActive(String),
    /// journalctl -p err - errors
    JournalErrors,
    /// journalctl -p warning - warnings
    JournalWarnings,
    /// pacman -Q <pkg> - package installed check
    PacmanQ(String),
    /// pacman -Qe | wc -l - package count
    PacmanCount,
    /// command -v <cmd> - tool exists
    CommandV(String),
    /// systemd-analyze - boot time
    SystemdAnalyze,
    /// uname -a - system info
    Uname,
    /// Custom probe command
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

/// Route capability - what the deterministic path can and cannot do.
#[derive(Debug, Clone)]
pub struct RouteCapability {
    /// Can produce a deterministic answer from probe data alone?
    /// TRUE only for narrow typed queries with extractable claims:
    /// - MemoryUsage, DiskUsage, ServiceStatus, TopProcesses
    /// FALSE for queries needing LLM interpretation:
    /// - "do I have nano", "what is my sound card", temperature
    pub can_answer_deterministically: bool,

    /// Evidence kinds required to answer this query.
    /// If not empty, at least one probe must succeed for each kind.
    pub required_evidence: Vec<EvidenceKind>,

    /// Minimum probes that MUST run (spine enforcement).
    /// Even if translator says probes=[], these will be added.
    pub spine_probes: Vec<ProbeId>,

    /// Whether this query inherently requires evidence to answer.
    /// "what cpu" = true, "help" = false.
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

/// Get probes for an evidence kind.
pub fn probes_for_evidence(kind: EvidenceKind) -> Vec<ProbeId> {
    match kind {
        EvidenceKind::Cpu => vec![ProbeId::Lscpu],
        EvidenceKind::CpuTemperature => vec![ProbeId::Sensors],
        EvidenceKind::Memory => vec![ProbeId::Free],
        EvidenceKind::Disk => vec![ProbeId::Df],
        EvidenceKind::BlockDevices => vec![ProbeId::Lsblk],
        EvidenceKind::Gpu => vec![], // Rely on hardware snapshot
        EvidenceKind::Audio => vec![ProbeId::LspciAudio, ProbeId::PactlCards],
        EvidenceKind::Network => vec![ProbeId::IpAddr],
        EvidenceKind::Processes => vec![ProbeId::TopCpu, ProbeId::TopMemory],
        EvidenceKind::Services => vec![ProbeId::FailedUnits],
        EvidenceKind::Journal => vec![ProbeId::JournalErrors, ProbeId::JournalWarnings],
        EvidenceKind::Packages => vec![ProbeId::PacmanCount],
        EvidenceKind::ToolExists => vec![], // Needs specific tool name
        EvidenceKind::BootTime => vec![ProbeId::SystemdAnalyze],
        EvidenceKind::System => vec![ProbeId::Uname],
    }
}

/// Convert ProbeId to command string.
pub fn probe_to_command(probe: &ProbeId) -> String {
    match probe {
        ProbeId::Lscpu => "lscpu".to_string(),
        ProbeId::Sensors => "sensors".to_string(),
        ProbeId::Free => "free -b".to_string(),
        ProbeId::Df => "df -h".to_string(),
        ProbeId::Lsblk => "lsblk -b -J".to_string(),
        ProbeId::LspciAudio => "lspci | grep -i audio".to_string(),
        ProbeId::PactlCards => "pactl list cards 2>/dev/null || true".to_string(),
        ProbeId::IpAddr => "ip addr".to_string(),
        ProbeId::TopMemory => "ps aux --sort=-%mem | head -6".to_string(),
        ProbeId::TopCpu => "ps aux --sort=-%cpu | head -6".to_string(),
        ProbeId::FailedUnits => "systemctl --failed --no-pager".to_string(),
        ProbeId::IsActive(s) => format!("systemctl is-active {}", s),
        ProbeId::JournalErrors => "journalctl -p err -b --no-pager | head -20".to_string(),
        ProbeId::JournalWarnings => "journalctl -p warning -b --no-pager | head -20".to_string(),
        ProbeId::PacmanQ(p) => format!("pacman -Q {} 2>/dev/null", p),
        ProbeId::PacmanCount => "pacman -Qe | wc -l".to_string(),
        ProbeId::CommandV(c) => format!("command -v {}", c),
        ProbeId::SystemdAnalyze => "systemd-analyze".to_string(),
        ProbeId::Uname => "uname -a".to_string(),
        ProbeId::Custom(c) => c.clone(),
    }
}

/// Enforce spine probes: if translator proposed empty probes but query requires evidence,
/// return the minimum required probes.
pub fn enforce_spine_probes(
    translator_probes: &[String],
    capability: &RouteCapability,
) -> (Vec<String>, Option<String>) {
    if !translator_probes.is_empty() {
        return (translator_probes.to_vec(), None);
    }

    if !capability.evidence_required {
        return (vec![], None);
    }

    if capability.spine_probes.is_empty() && capability.required_evidence.is_empty() {
        return (vec![], None);
    }

    // Build probe list from spine_probes and required_evidence
    let mut probes: Vec<String> = capability.spine_probes
        .iter()
        .map(|p| probe_to_command(p))
        .collect();

    for kind in &capability.required_evidence {
        for probe in probes_for_evidence(*kind) {
            let cmd = probe_to_command(&probe);
            if !probes.contains(&cmd) {
                probes.push(cmd);
            }
        }
    }

    let reason = if probes.is_empty() {
        None
    } else {
        Some(format!(
            "query requires {:?} evidence, enforcing {} probe(s)",
            capability.required_evidence,
            probes.len()
        ))
    };

    (probes, reason)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforce_spine_when_empty() {
        let cap = RouteCapability {
            evidence_required: true,
            required_evidence: vec![EvidenceKind::Cpu],
            spine_probes: vec![ProbeId::Lscpu],
            can_answer_deterministically: false,
        };

        let (probes, reason) = enforce_spine_probes(&[], &cap);
        assert!(!probes.is_empty(), "Should enforce probes when empty");
        assert!(reason.is_some(), "Should provide reason");
    }

    #[test]
    fn test_no_enforce_when_not_required() {
        let cap = RouteCapability {
            evidence_required: false,
            ..Default::default()
        };

        let (probes, reason) = enforce_spine_probes(&[], &cap);
        assert!(probes.is_empty(), "Should not enforce when not required");
        assert!(reason.is_none());
    }

    #[test]
    fn test_preserve_translator_probes() {
        let cap = RouteCapability {
            evidence_required: true,
            spine_probes: vec![ProbeId::Lscpu],
            ..Default::default()
        };

        let translator = vec!["custom_probe".to_string()];
        let (probes, reason) = enforce_spine_probes(&translator, &cap);
        assert_eq!(probes, translator, "Should preserve translator probes");
        assert!(reason.is_none());
    }
}
