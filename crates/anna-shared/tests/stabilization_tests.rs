//! Stabilization tests for v0.0.45.
//!
//! These tests lock invariants that ensure correctness:
//! - No probe, no claim: if probes_succeeded == 0, numeric claims must be rejected
//! - Evidence gating: deterministic answers must have ParsedProbeData from same request
//! - Reliability truthfulness: evidence_required + no evidence = reliability < 100

use anna_shared::reliability::{compute_reliability, ReliabilityInput};

/// Golden test: No probe, no claim invariant.
/// If probes_succeeded == 0 AND evidence_required == true,
/// any answer with claims should have invention_detected or reliability < 100.
#[test]
fn golden_no_probe_no_claim_invariant() {
    // Scenario: Query requires evidence, but no probes ran
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(0)
        .with_succeeded_probes(0)
        .with_total_claims(3) // Answer has 3 claims
        .with_verified_claims(0) // None verified (no evidence)
        .with_answer_grounded(false)
        .with_no_invention(true) // Guard didn't flag (but should have)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // INVARIANT: Reliability MUST be < 100 when claims exist but no evidence
    assert!(
        output.score < 100,
        "With evidence_required=true, 0 probes succeeded, and 3 unverified claims, \
        reliability must be < 100, got {}",
        output.score
    );

    // Should have evidence missing reason
    assert!(
        output.reasons.iter().any(|r| format!("{:?}", r).contains("Evidence")),
        "Should have evidence-related degradation reason"
    );
}

/// Golden test: Evidence missing with claims should cap reliability.
#[test]
fn golden_evidence_missing_caps_reliability() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(2)
        .with_succeeded_probes(0) // All probes failed
        .with_total_claims(5)
        .with_verified_claims(0)
        .with_answer_grounded(false)
        .with_no_invention(true)
        .with_translator_confidence(95);

    let output = compute_reliability(&input);

    // Should be significantly penalized
    assert!(
        output.score <= 70,
        "With all probes failed and unverified claims, reliability should be <= 70, got {}",
        output.score
    );
}

/// Golden test: No claims, no evidence required = high reliability.
#[test]
fn golden_no_claims_no_evidence_ok() {
    let input = ReliabilityInput::default()
        .with_evidence_required(false)
        .with_planned_probes(0)
        .with_succeeded_probes(0)
        .with_total_claims(0) // No claims
        .with_verified_claims(0)
        .with_answer_grounded(true) // Generic answer, no claims
        .with_no_invention(true)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // Should be reasonably high - no evidence needed, no claims
    assert!(
        output.score >= 80,
        "With no evidence required and no claims, reliability should be >= 80, got {}",
        output.score
    );
}

/// Golden test: Invention detected must cap at 40.
#[test]
fn golden_invention_detected_ceiling() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(1)
        .with_succeeded_probes(1)
        .with_total_claims(2)
        .with_verified_claims(0) // Claims not verified
        .with_answer_grounded(false)
        .with_no_invention(false) // INVENTION DETECTED
        .with_translator_confidence(100);

    let output = compute_reliability(&input);

    // INVARIANT: Invention ceiling is 40
    assert!(
        output.score <= 40,
        "With invention_detected=true, reliability must be <= 40, got {}",
        output.score
    );
}

/// Golden test: All probes succeeded, claims verified = high reliability.
#[test]
fn golden_fully_verified_high_reliability() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(3)
        .with_succeeded_probes(3)
        .with_total_claims(3)
        .with_verified_claims(3) // All claims verified
        .with_answer_grounded(true)
        .with_no_invention(true)
        .with_translator_confidence(95);

    let output = compute_reliability(&input);

    // Should be high reliability
    assert!(
        output.score >= 90,
        "With all probes succeeded and claims verified, reliability should be >= 90, got {}",
        output.score
    );
}

/// Test that partial probe success still provides reasonable reliability.
#[test]
fn test_partial_probe_success() {
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(4)
        .with_succeeded_probes(2) // 50% success
        .with_total_claims(2)
        .with_verified_claims(2)
        .with_answer_grounded(true)
        .with_no_invention(true)
        .with_translator_confidence(85);

    let output = compute_reliability(&input);

    // Should be degraded but not catastrophically
    assert!(
        output.score >= 60 && output.score <= 85,
        "With 50% probe success and verified claims, reliability should be 60-85, got {}",
        output.score
    );
}
