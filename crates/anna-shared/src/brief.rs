//! Ticket Brief with team-relevance filtering (v0.0.32).
//!
//! Specialists only see evidence relevant to their domain.
//! Storage team sees disk/lsblk, not memory/systemd noise.

use crate::rpc::ProbeResult;
use crate::teams::Team;
use crate::trace::EvidenceKind;
use serde::{Deserialize, Serialize};

/// Probe command patterns mapped to evidence kinds
const PROBE_PATTERNS: &[(&str, EvidenceKind)] = &[
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
        Team::Logs => vec![], // Logs team reviews log output (v0.0.42)
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

/// Filtered view of a ticket for team review (v0.0.32)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketBrief {
    /// Original user request
    pub user_request: String,
    /// Domain classification
    pub domain: String,
    /// Intent classification
    pub intent: String,
    /// Route class
    pub route_class: String,
    /// Filtered probe results (only team-relevant)
    pub relevant_probes: Vec<ProbeResult>,
    /// Evidence kinds present
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Count of probes filtered out
    pub filtered_count: usize,
    /// Any facts learned during ticket processing
    pub facts_learned: Vec<String>,
}

impl TicketBrief {
    /// Build a brief from ticket data and probe results
    pub fn build(
        user_request: &str,
        domain: &str,
        intent: &str,
        route_class: &str,
        team: Team,
        probe_results: &[ProbeResult],
        facts_learned: &[String],
    ) -> Self {
        let relevant: Vec<ProbeResult> = probe_results
            .iter()
            .filter(|p| is_probe_relevant(&p.command, team))
            .cloned()
            .collect();

        let filtered_count = probe_results.len() - relevant.len();

        // Collect unique evidence kinds from relevant probes
        let mut evidence_kinds: Vec<EvidenceKind> = relevant
            .iter()
            .filter_map(|p| evidence_kind_for_probe(&p.command))
            .collect();
        evidence_kinds.sort_by_key(|k| k.to_string());
        evidence_kinds.dedup();

        Self {
            user_request: user_request.to_string(),
            domain: domain.to_string(),
            intent: intent.to_string(),
            route_class: route_class.to_string(),
            relevant_probes: relevant,
            evidence_kinds,
            filtered_count,
            facts_learned: facts_learned.to_vec(),
        }
    }

    /// Check if brief has any evidence
    pub fn has_evidence(&self) -> bool {
        !self.relevant_probes.is_empty()
    }

    /// Get summary line for debug output
    pub fn summary(&self) -> String {
        let kinds: Vec<_> = self.evidence_kinds.iter().map(|k| k.to_string()).collect();
        if kinds.is_empty() {
            format!(
                "{} probes (none classified)",
                self.relevant_probes.len()
            )
        } else {
            format!(
                "{} probes ({}), {} filtered",
                self.relevant_probes.len(),
                kinds.join(", "),
                self.filtered_count
            )
        }
    }
}

/// Build brief from a ticket (convenience wrapper)
pub fn build_brief_from_ticket(
    ticket: &crate::ticket::Ticket,
    probe_results: &[ProbeResult],
) -> TicketBrief {
    TicketBrief::build(
        &ticket.user_request,
        &ticket.domain,
        &ticket.intent,
        &ticket.route_class,
        ticket.team,
        probe_results,
        &ticket.facts_learned,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_probe(command: &str) -> ProbeResult {
        ProbeResult {
            command: command.to_string(),
            exit_code: 0,
            stdout: "output".to_string(),
            stderr: String::new(),
            timing_ms: 100,
        }
    }

    #[test]
    fn test_evidence_kind_for_probe() {
        assert_eq!(evidence_kind_for_probe("df -h"), Some(EvidenceKind::Disk));
        assert_eq!(
            evidence_kind_for_probe("free -m"),
            Some(EvidenceKind::Memory)
        );
        assert_eq!(
            evidence_kind_for_probe("lsblk"),
            Some(EvidenceKind::BlockDevices)
        );
        assert_eq!(
            evidence_kind_for_probe("systemctl status nginx"),
            Some(EvidenceKind::Services)
        );
        assert_eq!(evidence_kind_for_probe("unknown_cmd"), None);
    }

    #[test]
    fn test_relevant_evidence_for_team() {
        let storage = relevant_evidence_for_team(Team::Storage);
        assert!(storage.contains(&EvidenceKind::Disk));
        assert!(storage.contains(&EvidenceKind::BlockDevices));
        assert!(!storage.contains(&EvidenceKind::Memory));

        let perf = relevant_evidence_for_team(Team::Performance);
        assert!(perf.contains(&EvidenceKind::Memory));
        assert!(perf.contains(&EvidenceKind::Cpu));
        assert!(!perf.contains(&EvidenceKind::Disk));
    }

    #[test]
    fn test_is_probe_relevant() {
        // Storage team sees disk probes
        assert!(is_probe_relevant("df -h", Team::Storage));
        assert!(is_probe_relevant("lsblk", Team::Storage));
        // Storage team doesn't see memory probes
        assert!(!is_probe_relevant("free -m", Team::Storage));

        // Performance team sees memory/cpu probes
        assert!(is_probe_relevant("free -m", Team::Performance));
        assert!(is_probe_relevant("lscpu", Team::Performance));
        // Performance team doesn't see disk probes
        assert!(!is_probe_relevant("df -h", Team::Performance));

        // General team sees everything
        assert!(is_probe_relevant("df -h", Team::General));
        assert!(is_probe_relevant("free -m", Team::General));
    }

    #[test]
    fn test_brief_build_filters_probes() {
        let probes = vec![
            mock_probe("df -h"),
            mock_probe("lsblk"),
            mock_probe("free -m"),
            mock_probe("systemctl status nginx"),
        ];

        let brief = TicketBrief::build(
            "how much disk space?",
            "storage",
            "question",
            "disk_usage",
            Team::Storage,
            &probes,
            &[],
        );

        // Storage team should only see 2 disk-related probes
        assert_eq!(brief.relevant_probes.len(), 2);
        assert_eq!(brief.filtered_count, 2);
        assert!(brief.evidence_kinds.contains(&EvidenceKind::Disk));
        assert!(brief.evidence_kinds.contains(&EvidenceKind::BlockDevices));
    }

    #[test]
    fn test_brief_general_sees_all() {
        let probes = vec![
            mock_probe("df -h"),
            mock_probe("free -m"),
            mock_probe("systemctl status nginx"),
        ];

        let brief = TicketBrief::build(
            "system health?",
            "system",
            "question",
            "system_health",
            Team::General,
            &probes,
            &[],
        );

        // General team sees all probes
        assert_eq!(brief.relevant_probes.len(), 3);
        assert_eq!(brief.filtered_count, 0);
    }

    #[test]
    fn test_brief_summary() {
        let probes = vec![mock_probe("df -h"), mock_probe("lsblk")];

        let brief = TicketBrief::build(
            "disk info",
            "storage",
            "question",
            "disk_usage",
            Team::Storage,
            &probes,
            &[],
        );

        let summary = brief.summary();
        assert!(summary.contains("2 probes"));
        assert!(summary.contains("disk"));
    }

    #[test]
    fn test_brief_with_facts_learned() {
        let probes = vec![mock_probe("df -h")];
        let facts = vec!["preferred_editor".to_string()];

        let brief = TicketBrief::build(
            "disk info",
            "storage",
            "question",
            "disk_usage",
            Team::Storage,
            &probes,
            &facts,
        );

        assert_eq!(brief.facts_learned.len(), 1);
        assert_eq!(brief.facts_learned[0], "preferred_editor");
    }

    // Golden tests for deterministic output
    #[test]
    fn golden_storage_brief_filters_memory() {
        let probes = vec![
            mock_probe("df -h /"),
            mock_probe("free -m"),
            mock_probe("lsblk --json"),
        ];

        let brief = TicketBrief::build(
            "is my disk full?",
            "storage",
            "question",
            "disk_usage",
            Team::Storage,
            &probes,
            &[],
        );

        // Exactly 2 probes should be relevant
        assert_eq!(brief.relevant_probes.len(), 2);
        // free -m should be filtered
        assert!(!brief
            .relevant_probes
            .iter()
            .any(|p| p.command.contains("free")));
    }

    #[test]
    fn golden_performance_brief_filters_disk() {
        let probes = vec![
            mock_probe("df -h /"),
            mock_probe("free -m"),
            mock_probe("lscpu"),
        ];

        let brief = TicketBrief::build(
            "why is system slow?",
            "performance",
            "investigate",
            "system_slow",
            Team::Performance,
            &probes,
            &[],
        );

        // Exactly 2 probes should be relevant (memory + cpu)
        assert_eq!(brief.relevant_probes.len(), 2);
        // df should be filtered
        assert!(!brief
            .relevant_probes
            .iter()
            .any(|p| p.command.contains("df")));
    }
}
