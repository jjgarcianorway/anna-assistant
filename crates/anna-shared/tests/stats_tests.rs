//! Tests for per-team and per-person statistics (v0.0.32).

use anna_shared::roster::Tier;
use anna_shared::stats::{GlobalStats, PersonStats, PersonStatsTracker, TeamStats};
use anna_shared::teams::Team;

#[test]
fn test_team_stats_new() {
    let stats = TeamStats::new(Team::Storage);
    assert_eq!(stats.team, Team::Storage);
    assert_eq!(stats.tickets_total, 0);
}

#[test]
fn test_team_stats_record_verified() {
    let mut stats = TeamStats::new(Team::Network);
    stats.record_verified(2, 85, false);

    assert_eq!(stats.tickets_total, 1);
    assert_eq!(stats.tickets_verified, 1);
    assert_eq!(stats.avg_rounds, 2.0);
    assert_eq!(stats.avg_reliability_score, 85.0);
    assert_eq!(stats.escalation_rate, 0.0);
}

#[test]
fn test_team_stats_success_rate() {
    let mut stats = TeamStats::new(Team::Performance);
    stats.record_verified(1, 90, false);
    stats.record_verified(2, 85, false);
    stats.record_failed(3, 50, true);

    assert_eq!(stats.tickets_total, 3);
    assert!((stats.success_rate() - 0.666).abs() < 0.01);
}

#[test]
fn test_global_stats_new() {
    let stats = GlobalStats::new();
    assert_eq!(stats.by_team.len(), 8);
    assert!(stats.most_consulted_team.is_none());
}

#[test]
fn test_global_stats_record_ticket() {
    let mut stats = GlobalStats::new();
    stats.record_ticket(Team::Storage, true, 1, 90, false);
    stats.record_ticket(Team::Storage, true, 2, 85, false);
    stats.record_ticket(Team::Network, false, 3, 50, true);

    assert_eq!(stats.total_requests, 3);
    assert_eq!(stats.get_team(Team::Storage).unwrap().tickets_total, 2);
    assert_eq!(stats.get_team(Team::Network).unwrap().tickets_total, 1);
    assert_eq!(stats.most_consulted_team, Some(Team::Storage));
}

#[test]
fn test_global_stats_overall_success_rate() {
    let mut stats = GlobalStats::new();
    stats.record_ticket(Team::Storage, true, 1, 90, false);
    stats.record_ticket(Team::Network, true, 1, 85, false);
    stats.record_ticket(Team::Hardware, false, 3, 50, true);

    assert!((stats.overall_success_rate() - 0.666).abs() < 0.01);
}

#[test]
fn test_global_stats_serialization() {
    let mut stats = GlobalStats::new();
    stats.record_ticket(Team::Security, true, 1, 88, false);

    let json = serde_json::to_string(&stats).unwrap();
    let parsed: GlobalStats = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.total_requests, 1);
    assert_eq!(parsed.get_team(Team::Security).unwrap().tickets_verified, 1);
}

// v0.0.32 PersonStats tests

#[test]
fn test_person_stats_from_roster() {
    let stats = PersonStats::from_roster(Team::Network, Tier::Junior);
    assert_eq!(stats.person_id, "network_jr");
    assert_eq!(stats.display_name, "Riley");
    assert_eq!(stats.team, Team::Network);
    assert_eq!(stats.tier, "junior");
}

#[test]
fn test_person_stats_record_closure() {
    let mut stats = PersonStats::from_roster(Team::Storage, Tier::Junior);
    stats.record_closure(2, 85);
    stats.record_closure(3, 90);

    assert_eq!(stats.tickets_closed, 2);
    assert!((stats.avg_loops - 2.5).abs() < 0.01);
    assert!((stats.avg_score - 87.5).abs() < 0.01);
}

#[test]
fn test_person_stats_escalations() {
    let mut jr_stats = PersonStats::from_roster(Team::Storage, Tier::Junior);
    let mut sr_stats = PersonStats::from_roster(Team::Storage, Tier::Senior);

    jr_stats.record_escalation_sent();
    jr_stats.record_escalation_sent();
    sr_stats.record_escalation_received();
    sr_stats.record_escalation_received();

    assert_eq!(jr_stats.escalations_sent, 2);
    assert_eq!(sr_stats.escalations_received, 2);
}

#[test]
fn test_person_stats_tracker_new() {
    let tracker = PersonStatsTracker::new();
    // 8 teams * 2 tiers = 16 persons
    assert_eq!(tracker.by_person.len(), 16);
    assert!(tracker.get_person("network_jr").is_some());
    assert!(tracker.get_person("storage_sr").is_some());
}

#[test]
fn test_person_stats_tracker_record_closure() {
    let mut tracker = PersonStatsTracker::new();
    tracker.record_closure("storage_jr", 2, 85);
    tracker.record_closure("storage_jr", 1, 90);

    let stats = tracker.get_person("storage_jr").unwrap();
    assert_eq!(stats.tickets_closed, 2);
    assert!((stats.avg_score - 87.5).abs() < 0.01);
}

#[test]
fn test_person_stats_tracker_record_escalation() {
    let mut tracker = PersonStatsTracker::new();
    tracker.record_escalation("network_jr", "network_sr");

    assert_eq!(
        tracker.get_person("network_jr").unwrap().escalations_sent,
        1
    );
    assert_eq!(
        tracker.get_person("network_sr").unwrap().escalations_received,
        1
    );
}

#[test]
fn test_person_stats_tracker_top_closers() {
    let mut tracker = PersonStatsTracker::new();
    tracker.record_closure("storage_jr", 1, 85);
    tracker.record_closure("storage_jr", 2, 90);
    tracker.record_closure("network_jr", 1, 88);

    let top = tracker.top_closers(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].person_id, "storage_jr");
    assert_eq!(top[0].tickets_closed, 2);
}

#[test]
fn test_person_stats_serialization() {
    let mut tracker = PersonStatsTracker::new();
    tracker.record_closure("perf_jr", 2, 92);

    let json = serde_json::to_string(&tracker).unwrap();
    let parsed: PersonStatsTracker = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.get_person("perf_jr").unwrap().tickets_closed, 1);
    assert_eq!(parsed.get_person("perf_jr").unwrap().avg_score, 92.0);
}

// Golden tests
#[test]
fn golden_riley_network_junior() {
    let stats = PersonStats::from_roster(Team::Network, Tier::Junior);
    assert_eq!(stats.person_id, "network_jr");
    assert_eq!(stats.display_name, "Riley");
}

#[test]
fn golden_taylor_storage_senior() {
    let stats = PersonStats::from_roster(Team::Storage, Tier::Senior);
    assert_eq!(stats.person_id, "storage_sr");
    assert_eq!(stats.display_name, "Taylor");
}
