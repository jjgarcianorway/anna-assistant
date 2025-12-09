//! Brief tests (v0.0.229).

#[cfg(test)]
mod tests {
    use crate::brief::{
        evidence_kind_for_probe, is_probe_relevant, relevant_evidence_for_team, TicketBrief,
    };
    use crate::rpc::ProbeResult;
    use crate::teams::Team;
    use crate::trace::EvidenceKind;

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
