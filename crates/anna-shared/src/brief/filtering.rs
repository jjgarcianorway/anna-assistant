//! Brief filtering logic (v0.0.229).

use crate::teams::Team;
use crate::trace::EvidenceKind;

/// Probe command patterns mapped to evidence kinds
pub const PROBE_PATTERNS: &[(&str, EvidenceKind)] = &[
    // Memory probes
    ("free", EvidenceKind::Memory),
    ("vmstat", EvidenceKind::Memory),
    ("top -b", EvidenceKind::Memory),
    // Disk probes
    ("df", EvidenceKind::Disk),
    ("du", EvidenceKind::Disk),
    ("lsblk", EvidenceKind::BlockDevices),
    ("blkid", EvidenceKind::BlockDevices),
    ("fdisk", EvidenceKind::BlockDevices),
    ("btrfs", EvidenceKind::BlockDevices),
    ("mount", EvidenceKind::Disk),
    // CPU probes
    ("lscpu", EvidenceKind::Cpu),
    ("cat /proc/cpuinfo", EvidenceKind::Cpu),
    ("nproc", EvidenceKind::Cpu),
    ("uptime", EvidenceKind::Cpu),
    // Service probes
    ("systemctl", EvidenceKind::Services),
    ("journalctl", EvidenceKind::Services),
    ("service ", EvidenceKind::Services),
];

/// Get evidence kind for a probe command
pub fn evidence_kind_for_probe(command: &str) -> Option<EvidenceKind> {
    let cmd_lower = command.to_lowercase();
    for (pattern, kind) in PROBE_PATTERNS {
        if cmd_lower.contains(pattern) {
            return Some(kind.clone());
        }
    }
    None
}

/// Get which evidence kinds are relevant for a team
pub fn relevant_evidence_for_team(team: Team) -> Vec<EvidenceKind> {
    match team {
        Team::Desktop => vec![], // Desktop sees all (user environment)
        Team::Storage => vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        Team::Network => vec![], // Network evidence not yet defined
        Team::Performance => vec![EvidenceKind::Memory, EvidenceKind::Cpu],
        Team::Services => vec![EvidenceKind::Services],
        Team::Security => vec![EvidenceKind::Services], // Security reviews services too
        Team::Hardware => vec![
            EvidenceKind::Cpu,
            EvidenceKind::Memory,
            EvidenceKind::BlockDevices,
        ],
        Team::Logs => vec![],    // Logs team reviews log output (v0.0.42)
        Team::General => vec![], // General sees all
    }
}

/// Check if a probe is relevant for a team
pub fn is_probe_relevant(command: &str, team: Team) -> bool {
    let relevant = relevant_evidence_for_team(team);

    // Empty relevance list means "all are relevant"
    if relevant.is_empty() {
        return true;
    }

    // Check if probe's evidence kind matches team's interests
    match evidence_kind_for_probe(command) {
        Some(kind) => relevant.contains(&kind),
        None => true, // Unknown probes included by default
    }
}
