//! Tests for ticket_packet.rs

use anna_shared::facts::FactKey;
use anna_shared::rpc::{ProbeResult, SpecialistDomain};
use anna_shared::teams::Team;
use anna_shared::ticket_packet::{
    policy_for_team, recommended_probes_for_domain, PacketPolicy,
    TicketPacket, TicketPacketBuilder,
};
use anna_shared::trace::EvidenceKind;

fn mock_probe(cmd: &str, stdout: &str, exit_code: i32) -> ProbeResult {
    ProbeResult {
        command: cmd.to_string(),
        exit_code,
        stdout: stdout.to_string(),
        stderr: String::new(),
        timing_ms: 50,
    }
}

#[test]
fn test_packet_creation() {
    let packet = TicketPacket::new("system_triage", SpecialistDomain::System, Team::General);
    assert_eq!(packet.route_class, "system_triage");
    assert_eq!(packet.domain, SpecialistDomain::System);
    assert!(!packet.has_evidence());
}

#[test]
fn test_packet_add_probe() {
    let mut packet = TicketPacket::new("test", SpecialistDomain::System, Team::General);
    packet.add_probe(mock_probe("free -h", "Mem: 16G", 0));

    assert_eq!(packet.budget.probes_executed, 1);
    assert_eq!(packet.budget.probes_succeeded, 1);
    assert!(packet.has_evidence());
}

#[test]
fn test_packet_builder() {
    let packet = TicketPacketBuilder::new("test", SpecialistDomain::System, Team::General)
        .fast_path()
        .plan_probes(3)
        .with_evidence(EvidenceKind::Memory)
        .add_probe(mock_probe("free -h", "Mem: 16G", 0))
        .build();

    assert_eq!(packet.budget.probes_planned, 3);
    assert_eq!(packet.budget.probes_executed, 1);
    assert!(packet.evidence_kinds.contains(&EvidenceKind::Memory));
}

#[test]
fn test_packet_budget_exceeded() {
    let packet = TicketPacketBuilder::new("test", SpecialistDomain::System, Team::General)
        .fast_path()
        .plan_probes(2)
        .add_probe(mock_probe("p1", &"x".repeat(50000), 0))
        .add_probe(mock_probe("p2", &"x".repeat(50000), 0))
        .build();

    // Fast path budget is 32KB, so 100KB should exceed it
    assert!(packet.budget.budget_exceeded);
}

#[test]
fn test_probe_success_rate() {
    let mut packet = TicketPacket::new("test", SpecialistDomain::System, Team::General);
    packet.add_probe(mock_probe("p1", "ok", 0));
    packet.add_probe(mock_probe("p2", "fail", 1));

    assert_eq!(packet.probe_success_rate(), 0.5);
}

#[test]
fn test_find_probe() {
    let mut packet = TicketPacket::new("test", SpecialistDomain::System, Team::General);
    packet.add_probe(mock_probe("free -h", "Mem: 16G", 0));
    packet.add_probe(mock_probe("df -h", "/dev/sda1 50G", 0));

    assert!(packet.find_probe("free").is_some());
    assert!(packet.find_probe("df").is_some());
    assert!(packet.find_probe("lsblk").is_none());
}

#[test]
fn test_recommended_probes() {
    let system_probes = recommended_probes_for_domain(SpecialistDomain::System);
    assert!(system_probes.contains(&"memory_info"));

    let storage_probes = recommended_probes_for_domain(SpecialistDomain::Storage);
    assert!(storage_probes.contains(&"disk_usage"));
}

// v0.0.36: PacketPolicy tests
#[test]
fn test_policy_for_desktop() {
    let policy = PacketPolicy::for_team(Team::Desktop);
    assert_eq!(policy.max_summary_lines, 10);
    assert!(policy.allowed_facts.contains(&FactKey::PreferredEditor));
}

#[test]
fn test_policy_for_storage() {
    let policy = PacketPolicy::for_team(Team::Storage);
    assert!(policy.required_probes.contains(&"disk_usage"));
    assert!(policy.required_probes.contains(&"block_devices"));
}

#[test]
fn test_policy_for_network() {
    let policy = PacketPolicy::for_team(Team::Network);
    assert!(policy.allowed_facts.contains(&FactKey::NetworkPrimaryInterface));
    assert!(policy.required_probes.contains(&"network_addrs"));
}

#[test]
fn test_policy_for_performance() {
    let policy = PacketPolicy::for_team(Team::Performance);
    assert_eq!(policy.max_probes, 5);
    assert!(policy.required_probes.contains(&"memory_info"));
}

#[test]
fn test_policy_truncate_summary() {
    let policy = PacketPolicy {
        max_summary_lines: 3,
        ..Default::default()
    };
    let long_summary = "line1\nline2\nline3\nline4\nline5";
    let truncated = policy.truncate_summary(long_summary);
    assert!(truncated.contains("omitted"));
    assert!(truncated.lines().count() <= 3);
}

#[test]
fn test_policy_no_truncate_short() {
    let policy = PacketPolicy::for_team(Team::General);
    let short_summary = "line1\nline2";
    let result = policy.truncate_summary(short_summary);
    assert_eq!(result, short_summary);
}

#[test]
fn test_policy_fact_allowed() {
    let policy = PacketPolicy::for_team(Team::Desktop);
    assert!(policy.is_fact_allowed(&FactKey::PreferredEditor));
    assert!(!policy.is_fact_allowed(&FactKey::NetworkPrimaryInterface));
}

#[test]
fn test_policy_truncation_deterministic() {
    let policy = PacketPolicy {
        max_summary_lines: 4,
        ..Default::default()
    };
    let summary = "a\nb\nc\nd\ne\nf";
    let result1 = policy.truncate_summary(summary);
    let result2 = policy.truncate_summary(summary);
    assert_eq!(result1, result2); // Must be deterministic
}

#[test]
fn test_all_teams_have_policy() {
    let teams = [
        Team::Desktop,
        Team::Storage,
        Team::Network,
        Team::Performance,
        Team::Services,
        Team::Security,
        Team::Hardware,
        Team::General,
    ];

    for team in teams {
        let policy = policy_for_team(team);
        assert!(policy.max_summary_lines >= 10);
        assert!(policy.max_probes >= 3);
    }
}
