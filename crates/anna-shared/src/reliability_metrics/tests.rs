//! Acceptance tests for reliability metrics (v0.0.444).
//!
//! Tests:
//! - Specialist timeout → FailedTimeout
//! - Specialist invalid JSON → FailedParse
//! - Missing required probe → FailedProbes
//! - Probe failure → FailedProbes
//! - Stats reflect reality

use super::*;

/// Test: Specialist timeout results in FailedTimeout outcome.
#[test]
fn test_specialist_timeout_outcome() {
    let cond = OutcomeConditions {
        timeout_occurred: true,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::FailedTimeout);
    assert!(outcome.is_failure());
    assert!(!outcome.is_resolved());

    // Stats must reflect this
    let mut stats = ReliabilityStats::new();
    stats.record_outcome(CanonicalOutcome::FailedTimeout, Some("network"));
    assert_eq!(stats.failed_timeout, 1);
    assert_eq!(stats.answered_verified, 0);
    assert!((stats.verified_rate() - 0.0).abs() < 0.001);
    assert!((stats.failure_rate() - 1.0).abs() < 0.001);
}

/// Test: Specialist invalid JSON results in FailedParse outcome.
#[test]
fn test_specialist_invalid_json_outcome() {
    // Specialist responded but JSON was invalid
    let cond = OutcomeConditions {
        specialist_responded: true,
        json_valid: false,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::FailedParse);
    assert!(outcome.is_failure());
    assert!(!outcome.is_resolved());

    // Stats must reflect this
    let mut stats = ReliabilityStats::new();
    stats.record_outcome(CanonicalOutcome::FailedParse, Some("storage"));
    assert_eq!(stats.failed_parse, 1);
    assert_eq!(stats.answered_verified, 0);
}

/// Test: Specialist didn't respond at all → FailedParse.
#[test]
fn test_specialist_no_response_outcome() {
    let cond = OutcomeConditions {
        specialist_responded: false,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::FailedParse);
}

/// Test: Schema validation failed → FailedParse.
#[test]
fn test_schema_validation_failed_outcome() {
    let cond = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: false,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::FailedParse);
}

/// Test: Required probes failed → FailedProbes outcome.
#[test]
fn test_probe_failure_outcome() {
    let cond = OutcomeConditions {
        probes_failed: true,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::FailedProbes);
    assert!(outcome.is_failure());
    assert!(!outcome.is_resolved());

    // Stats must reflect this
    let mut stats = ReliabilityStats::new();
    stats.record_outcome(CanonicalOutcome::FailedProbes, Some("hardware"));
    assert_eq!(stats.failed_probes, 1);
    assert_eq!(stats.answered_verified, 0);
}

/// Test: User cancelled → AbortedByUser (not a failure).
#[test]
fn test_user_cancelled_outcome() {
    let cond = OutcomeConditions {
        user_cancelled: true,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::AbortedByUser);
    assert!(!outcome.is_failure()); // Cancellation is not a failure
    assert!(!outcome.is_resolved());
}

/// Test: Clarification asked → ClarificationNeeded.
#[test]
fn test_clarification_outcome() {
    let cond = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        clarification_asked: true,
        answer_rendered: false,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::ClarificationNeeded);
    assert!(!outcome.is_terminal()); // Pending user action
}

/// Test: Verified answer with high evidence coverage.
#[test]
fn test_verified_answer_outcome() {
    let cond = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.9,
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::AnsweredVerified);
    assert!(outcome.is_resolved());
    assert!(outcome.is_useful());
}

/// Test: Partial answer with medium evidence coverage.
#[test]
fn test_partial_answer_outcome() {
    let cond = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.5, // 50% coverage
        ..Default::default()
    };

    let outcome = cond.determine();
    assert_eq!(outcome, CanonicalOutcome::AnsweredPartial);
    assert!(!outcome.is_resolved());
    assert!(outcome.is_partial());
    assert!(outcome.is_useful());
}

/// Test: Stats never show 100% if failures exist.
#[test]
fn test_stats_no_false_100_percent() {
    let mut stats = ReliabilityStats::new();

    // 9 verified, 1 timeout
    for _ in 0..9 {
        stats.record_outcome(CanonicalOutcome::AnsweredVerified, None);
    }
    stats.record_outcome(CanonicalOutcome::FailedTimeout, None);

    assert_eq!(stats.total_requests, 10);
    assert_eq!(stats.answered_verified, 9);
    assert_eq!(stats.failed_timeout, 1);

    // Verified rate should be 90%, NOT 100%
    assert!((stats.verified_rate() - 0.9).abs() < 0.001);

    // Failure rate should be 10%
    assert!((stats.failure_rate() - 0.1).abs() < 0.001);
}

/// Test: "Failed to parse specialist response" error must result in FailedParse.
#[test]
fn test_parse_error_string_detection() {
    // This simulates the observed bug:
    // "Failed to parse specialist response. Parse error: Timeout"
    // yet ticket marked resolved and success rate 100%

    // With our system, timeout should be FailedTimeout
    let cond_timeout = OutcomeConditions {
        timeout_occurred: true,
        ..Default::default()
    };
    assert_eq!(cond_timeout.determine(), CanonicalOutcome::FailedTimeout);

    // Parse error without timeout
    let cond_parse = OutcomeConditions {
        specialist_responded: true,
        json_valid: false,
        ..Default::default()
    };
    assert_eq!(cond_parse.determine(), CanonicalOutcome::FailedParse);
}

/// Test: Request metrics with full flow.
#[test]
fn test_request_metrics_flow() {
    let m = RequestMetricsBuilder::new("REQ-001", "What is my disk usage?")
        .topic("storage")
        .intent("diagnose")
        .probes(vec!["df".into()], vec!["df".into()], 1)
        .claims(3, 3, vec!["ev1".into(), "ev2".into()])
        .timing(50, 500, 100)
        .validation(true, None)
        .models(Some("qwen".into()), Some("qwen".into()), None)
        .finish(CanonicalOutcome::AnsweredVerified, None);

    assert_eq!(m.evidence_coverage(), 1.0);
    assert_eq!(m.probe_coverage(), 1.0);
    assert!(m.outcome.is_resolved());

    // Stats should reflect this
    let mut stats = ReliabilityStats::new();
    stats.record(&m);
    assert_eq!(stats.answered_verified, 1);
    assert!((stats.verified_rate() - 1.0).abs() < 0.001);
}

/// Test: Request metrics with failure.
#[test]
fn test_request_metrics_failure() {
    let m = RequestMetricsBuilder::new("REQ-002", "Test query")
        .topic("network")
        .timing(50, 15000, 0) // Long LLM time - timeout
        .finish(
            CanonicalOutcome::FailedTimeout,
            Some("LLM timed out".into()),
        );

    assert!(m.outcome.is_failure());

    let mut stats = ReliabilityStats::new();
    stats.record(&m);
    assert_eq!(stats.failed_timeout, 1);
    assert!((stats.verified_rate() - 0.0).abs() < 0.001);
}

/// Test: Model inventory deduplication.
#[test]
fn test_model_inventory_no_duplicates() {
    let mut inv = ModelInventory::new();

    // Add same model with different cases
    inv.add_discovered("Qwen2.5:7b", ModelOwner::User);
    inv.add_discovered("qwen2.5:7b", ModelOwner::User);
    inv.add_discovered("QWEN2.5:7B", ModelOwner::User);

    // Should only have 1 model
    assert_eq!(inv.discovered_count(), 1);
}

/// Test: Model inventory ownership tracking.
#[test]
fn test_model_inventory_ownership() {
    let mut inv = ModelInventory::new();

    inv.add_discovered("user_model:1b", ModelOwner::User);
    inv.add_anna_installed("anna_model:1b");

    assert_eq!(inv.user_installed_count(), 1);
    assert_eq!(inv.anna_installed_count(), 1);
    assert_eq!(inv.discovered_count(), 2);
}

/// Test: Probe inventory.
#[test]
fn test_probe_inventory() {
    let inv = default_probe_inventory();

    // Should have common probes
    assert!(inv.get("df").is_some());
    assert!(inv.get("free").is_some());
    assert!(inv.get("systemctl_failed").is_some());

    // Display should work
    let display = inv.display();
    assert!(display.contains("df"));
    assert!(display.contains("probes available"));
}

// === End-to-end scenario tests ===

/// Scenario: "is this a laptop or a desktop?"
/// Without proper hardware probes, should be CLARIFICATION_NEEDED or FAILED_PROBES.
#[test]
fn test_scenario_laptop_or_desktop() {
    // If probes fail (e.g., can't determine chassis type)
    let cond_probe_fail = OutcomeConditions {
        probes_failed: true,
        ..Default::default()
    };
    assert_eq!(cond_probe_fail.determine(), CanonicalOutcome::FailedProbes);

    // If we can't answer with evidence, should ask clarification
    let cond_clarify = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        clarification_asked: true,
        answer_rendered: false,
        ..Default::default()
    };
    assert_eq!(
        cond_clarify.determine(),
        CanonicalOutcome::ClarificationNeeded
    );

    // Should NEVER be "verified 90%" without evidence
    let cond_no_evidence = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.0, // No evidence
        ..Default::default()
    };
    // Low evidence = partial, not verified
    assert_eq!(
        cond_no_evidence.determine(),
        CanonicalOutcome::AnsweredPartial
    );
    assert!(!cond_no_evidence.determine().is_resolved());
}

/// Scenario: "what is my vim setup?"
/// Needs .vimrc probe; if fails, should not claim success.
#[test]
fn test_scenario_vim_setup() {
    // Probe success with evidence
    let cond_success = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 0.9, // Has .vimrc content
        ..Default::default()
    };
    assert_eq!(cond_success.determine(), CanonicalOutcome::AnsweredVerified);

    // Probe failed (no .vimrc)
    let cond_probe_fail = OutcomeConditions {
        probes_failed: true,
        ..Default::default()
    };
    assert_eq!(cond_probe_fail.determine(), CanonicalOutcome::FailedProbes);
}

/// Scenario: "is nano installed?"
/// Simple yes/no with evidence from `which nano`.
#[test]
fn test_scenario_nano_installed() {
    // Probe worked, evidence supports answer
    let cond_yes = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 1.0, // which nano returned path
        ..Default::default()
    };
    assert_eq!(cond_yes.determine(), CanonicalOutcome::AnsweredVerified);

    // Probe worked, no nano found - still verified (evidence supports "no")
    let cond_no = OutcomeConditions {
        specialist_responded: true,
        json_valid: true,
        schema_valid: true,
        answer_rendered: true,
        evidence_coverage: 1.0, // which nano returned empty
        ..Default::default()
    };
    assert_eq!(cond_no.determine(), CanonicalOutcome::AnsweredVerified);
}

/// Test: Stats display format.
#[test]
fn test_stats_display_format() {
    let mut stats = ReliabilityStats::new();
    stats.record_outcome(CanonicalOutcome::AnsweredVerified, Some("storage"));
    stats.record_outcome(CanonicalOutcome::FailedTimeout, Some("network"));

    let display = stats.display();

    // Must contain section headers
    assert!(display.contains("[requests]"));
    assert!(display.contains("[latency]"));
    assert!(display.contains("[reliability]"));

    // Must contain actual counts
    assert!(display.contains("total_requests"));
    assert!(display.contains("answered_verified"));
    assert!(display.contains("failed_timeout"));

    // Must contain rates
    assert!(display.contains("verified_rate"));
    assert!(display.contains("failure_rate"));
}

/// Test: Conversion from old ticket_integrity outcome.
#[test]
fn test_from_ticket_integrity_outcome() {
    use crate::ticket_integrity::outcome::TicketOutcome as TIO;

    assert_eq!(
        from_ticket_integrity_outcome(TIO::Answered),
        CanonicalOutcome::AnsweredVerified
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ParseError),
        CanonicalOutcome::FailedParse
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ProbeError),
        CanonicalOutcome::FailedProbes
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::ClarificationPending),
        CanonicalOutcome::ClarificationNeeded
    );
    assert_eq!(
        from_ticket_integrity_outcome(TIO::Cancelled),
        CanonicalOutcome::AbortedByUser
    );
}

/// Test: Conversion from old ticket_state outcome.
#[test]
fn test_from_ticket_state_outcome() {
    use crate::ticket_state::TicketOutcome as TSO;

    assert_eq!(
        from_ticket_state_outcome(TSO::Success),
        CanonicalOutcome::AnsweredVerified
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::Partial),
        CanonicalOutcome::AnsweredPartial
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorParse),
        CanonicalOutcome::FailedParse
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorTimeout),
        CanonicalOutcome::FailedTimeout
    );
    assert_eq!(
        from_ticket_state_outcome(TSO::ErrorTool),
        CanonicalOutcome::FailedProbes
    );
}
