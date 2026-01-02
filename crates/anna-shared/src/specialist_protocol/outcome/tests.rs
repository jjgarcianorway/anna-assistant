//! Tests for ticket outcome determination and statistics.

use super::{determine_outcome, HonestTicketStats, TicketOutcome};
use crate::specialist_protocol::{ProbeEvidence, ResponseMeta, StrictResponse};

fn make_meta() -> ResponseMeta {
    ResponseMeta {
        handled_by: "Test".to_string(),
        ticket_id: "TEST-001".to_string(),
        version: 1,
    }
}

fn make_success() -> StrictResponse {
    StrictResponse::success(
        "services.systemd",
        "check_failed_services",
        "No failed systemd services.",
        vec!["0 failed units".to_string()],
        vec![ProbeEvidence {
            id: "systemctl_failed".to_string(),
            summary: "0 failed units".to_string(),
            raw_reference: None,
        }],
        make_meta(),
    )
}

#[test]
fn test_success_outcome() {
    let response = make_success();
    let validation = crate::specialist_protocol::validate_response(&response);
    let outcome = determine_outcome(&response, &validation);

    assert_eq!(outcome, TicketOutcome::Success);
    assert!(outcome.is_resolved());
    assert_eq!(outcome.xp_value(), 10);
}

#[test]
fn test_useful_partial_outcome() {
    let response = StrictResponse::partial(
        "storage.disk",
        "check_disk_usage",
        "Root filesystem is at 97% used.",
        vec!["30 GiB free".to_string()],
        "Could not identify largest directories.",
        vec![ProbeEvidence {
            id: "df".to_string(),
            summary: "97% used".to_string(),
            raw_reference: None,
        }],
        make_meta(),
    )
    .with_confidence(0.6);

    let validation = crate::specialist_protocol::validate_response(&response);
    let outcome = determine_outcome(&response, &validation);

    assert_eq!(outcome, TicketOutcome::UsefulPartial);
    assert!(outcome.is_resolved());
}

#[test]
fn test_failure_outcome() {
    let response = StrictResponse::failure(
        "system",
        "unknown",
        "Complete system failure occurred.",
        make_meta(),
    );

    let validation = crate::specialist_protocol::validate_response(&response);
    let outcome = determine_outcome(&response, &validation);

    assert_eq!(outcome, TicketOutcome::Failed);
    assert!(outcome.is_failed());
}

#[test]
fn test_honest_unknown_outcome() {
    let response = StrictResponse::failure(
        "network",
        "check_vpn",
        "I don't have the capability to check VPN configuration.",
        make_meta(),
    );

    let validation = crate::specialist_protocol::validate_response(&response);
    let outcome = determine_outcome(&response, &validation);

    assert_eq!(outcome, TicketOutcome::HonestUnknown);
    assert!(outcome.is_resolved()); // Honest unknown is still "resolved"
}

#[test]
fn test_stats_recording() {
    let mut stats = HonestTicketStats::default();

    stats.record(TicketOutcome::Success, 500);
    stats.record(TicketOutcome::Success, 600);
    stats.record(TicketOutcome::UsefulPartial, 800);
    stats.record(TicketOutcome::Failed, 100);

    assert_eq!(stats.total, 4);
    assert_eq!(stats.success, 2);
    assert_eq!(stats.useful_partial, 1);
    assert_eq!(stats.failed, 1);
    assert_eq!(stats.success_rate(), 50.0);
    assert_eq!(stats.resolved(), 3);
}

#[test]
fn test_stats_validation() {
    let mut stats = HonestTicketStats::default();
    stats.total = 10;
    stats.success = 10;
    stats.failed = 1; // This is inconsistent!

    let result = stats.validate();
    assert!(result.is_err());
}

#[test]
fn test_stats_display() {
    let mut stats = HonestTicketStats::default();
    stats.record(TicketOutcome::Success, 500);
    stats.record_parse_error();

    let display = format!("{}", stats);
    assert!(display.contains("total_tickets"));
    assert!(display.contains("parse"));
}
