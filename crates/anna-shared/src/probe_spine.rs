//! Probe spine: deterministic tool selection and evidence requirements.
//! Prevents "no probes, no evidence, but claims anyway" scenarios.

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

/// Decision from probe spine enforcement.
#[derive(Debug, Clone)]
pub struct ProbeSpineDecision {
    pub enforced: bool,
    pub reason: String,
    pub probes: Vec<ProbeId>,
    pub evidence_kinds: Vec<EvidenceKind>,
}

/// Extract package name from "do I have X" style queries.
fn extract_package_name(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    // Patterns: "do I have nano", "is nano installed", "have I got vim"
    let patterns = [
        ("do i have ", true),
        ("do you have ", true),
        ("is ", false),  // "is nano installed"
        ("have i got ", true),
        ("got ", true),
    ];

    for (pattern, after) in patterns {
        if let Some(idx) = lower.find(pattern) {
            let start = if after { idx + pattern.len() } else { idx + pattern.len() };
            let rest = &text[start..];
            // Extract first word as package name
            let pkg: String = rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !pkg.is_empty() && pkg.len() > 1 {
                // Skip if followed by "installed" (for "is X installed" pattern)
                let pkg_lower = pkg.to_lowercase();
                if pkg_lower != "it" && pkg_lower != "there" && pkg_lower != "this" {
                    return Some(pkg.to_lowercase());
                }
            }
        }
    }
    None
}

/// Enforce minimum probes based on USER TEXT keywords (last line of defense).
pub fn enforce_minimum_probes(
    user_text: &str,
    translator_probes: &[String],
) -> ProbeSpineDecision {
    let lower = user_text.to_lowercase();
    let mut probes: Vec<ProbeId> = Vec::new();
    let mut evidence_kinds: Vec<EvidenceKind> = Vec::new();
    let mut reasons: Vec<&str> = Vec::new();

    // Rule 1: Package/tool check
    if lower.contains("do i have") || lower.contains("is installed")
        || lower.contains("have i got") || lower.contains("installed?")
    {
        if let Some(pkg) = extract_package_name(user_text) {
            probes.push(ProbeId::PacmanQ(pkg.clone()));
            probes.push(ProbeId::CommandV(pkg));
            evidence_kinds.push(EvidenceKind::Packages);
            evidence_kinds.push(EvidenceKind::ToolExists);
            reasons.push("package/tool check");
        }
    }

    // Rule 2: Sound/audio
    if lower.contains("sound card") || lower.contains("audio device")
        || lower.contains("audio card") || lower.contains("sound device")
        || (lower.contains("sound") && lower.contains("hardware"))
        || (lower.contains("audio") && lower.contains("hardware"))
    {
        if !probes.iter().any(|p| matches!(p, ProbeId::LspciAudio)) {
            probes.push(ProbeId::LspciAudio);
            probes.push(ProbeId::PactlCards);
            evidence_kinds.push(EvidenceKind::Audio);
            reasons.push("audio hardware query");
        }
    }

    // Rule 3: Temperature
    if lower.contains("temperature") || lower.contains(" temp ")
        || lower.contains("thermal") || lower.contains("temps?")
        || lower.contains("how hot")
    {
        if !probes.iter().any(|p| matches!(p, ProbeId::Sensors)) {
            probes.push(ProbeId::Sensors);
            evidence_kinds.push(EvidenceKind::CpuTemperature);
            reasons.push("temperature query");
        }
    }

    // Rule 4: CPU cores/model/architecture
    if lower.contains("cores") || lower.contains("cpu model")
        || lower.contains("architecture") || lower.contains("processor")
        || lower.contains("how many cpu")
    {
        if !probes.iter().any(|p| matches!(p, ProbeId::Lscpu)) {
            probes.push(ProbeId::Lscpu);
            evidence_kinds.push(EvidenceKind::Cpu);
            reasons.push("CPU info query");
        }
    }

    // Rule 5: System health / errors / problems
    if lower.contains("how is my computer") || lower.contains("errors")
        || lower.contains("problems") || lower.contains("system health")
        || lower.contains("what's wrong") || lower.contains("issues")
    {
        if !probes.iter().any(|p| matches!(p, ProbeId::JournalErrors)) {
            probes.push(ProbeId::JournalErrors);
            probes.push(ProbeId::FailedUnits);
            probes.push(ProbeId::SystemdAnalyze);
            evidence_kinds.push(EvidenceKind::Journal);
            evidence_kinds.push(EvidenceKind::Services);
            evidence_kinds.push(EvidenceKind::BootTime);
            reasons.push("system health query");
        }
    }

    // Merge with translator probes (translator probes come first)
    let mut final_probes = probes.clone();
    for tp in translator_probes {
        // Parse translator probe string to ProbeId if possible, or use Custom
        let probe_id = ProbeId::Custom(tp.clone());
        if !final_probes.iter().any(|p| probe_to_command(p) == *tp) {
            final_probes.insert(0, probe_id);
        }
    }

    let enforced = !probes.is_empty();
    let reason = if reasons.is_empty() {
        "no keyword matches".to_string()
    } else {
        format!("enforced for: {}", reasons.join(", "))
    };

    ProbeSpineDecision {
        enforced,
        reason,
        probes: final_probes,
        evidence_kinds,
    }
}

// Tests are in tests/probe_spine_tests.rs
