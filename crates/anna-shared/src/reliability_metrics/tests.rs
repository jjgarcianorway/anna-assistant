//! Request metrics tests for reliability metrics (v0.0.444).
//!
//! Tests for RequestMetrics and RequestMetricsBuilder functionality.

use super::*;

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
