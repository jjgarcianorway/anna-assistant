//! Outcome condition tests for reliability metrics.
//!
//! Tests canonical outcome determination based on various conditions
//! (timeout, parse errors, probe failures, etc.).

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
