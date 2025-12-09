//! Probe command generation (v0.0.193).

use super::types::{EvidenceKind, ProbeId};

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
        // v0.45.4: Use JSON output for proper SYSLOG_IDENTIFIER attribution
        ProbeId::JournalErrors => "journalctl -p err -b --no-pager -o json | head -50".to_string(),
        ProbeId::JournalWarnings => {
            "journalctl -p warning -b --no-pager -o json | head -50".to_string()
        }
        ProbeId::PacmanQ(p) => format!("pacman -Q {} 2>/dev/null", p),
        ProbeId::PacmanCount => "pacman -Qe | wc -l".to_string(),
        // v0.45.4: Use login shell to get full PATH (e.g., ~/.bashrc exports)
        ProbeId::CommandV(c) => format!("sh -lc 'command -v {}'", c),
        ProbeId::SystemdAnalyze => "systemd-analyze".to_string(),
        ProbeId::Uname => "uname -a".to_string(),
        ProbeId::Custom(c) => c.clone(),
    }
}
