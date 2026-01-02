//! Golden tests for reliability scoring - FallbackUsed guardrail tests.
//!
//! These tests lock exact behavior. Changes require explicit approval.
//! v0.0.24: FallbackUsed guardrail tests

use anna_shared::reliability::{
    compute_reliability, ReliabilityExplanation, ReliabilityInput, ReliabilityReason,
};
use anna_shared::trace::{FallbackUsed, SpecialistOutcome};

/// Timeout + fallback used → FallbackUsed penalty applies (-5)
#[test]
fn golden_fallback_timeout_with_fallback() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        answer_grounded: true,
        specialist_outcome: Some(SpecialistOutcome::Timeout),
        fallback_used: Some(FallbackUsed::Deterministic {
            route_class: "memory_usage".to_string(),
        }),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Should have FallbackUsed reason
    assert!(
        output.reasons.contains(&ReliabilityReason::FallbackUsed),
        "Timeout with fallback should include FallbackUsed reason"
    );

    // Score should be 100 - 5 = 95
    assert_eq!(
        output.score, 95,
        "Timeout with fallback should apply -5 penalty"
    );
}

/// Timeout + no fallback → No FallbackUsed penalty (only probe timeout)
#[test]
fn golden_fallback_timeout_no_fallback() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 0,
        timed_out_probes: 1,
        no_invention: true,
        specialist_outcome: Some(SpecialistOutcome::Timeout),
        fallback_used: Some(FallbackUsed::None),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Should NOT have FallbackUsed reason
    assert!(
        !output.reasons.contains(&ReliabilityReason::FallbackUsed),
        "Timeout without fallback should not include FallbackUsed reason"
    );

    // Should have ProbeTimeout reason instead
    assert!(output.reasons.contains(&ReliabilityReason::ProbeTimeout));
}

/// Specialist Ok → No FallbackUsed penalty (normal operation)
#[test]
fn golden_fallback_specialist_ok() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        answer_grounded: true,
        specialist_outcome: Some(SpecialistOutcome::Ok),
        fallback_used: Some(FallbackUsed::None),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Should NOT have FallbackUsed reason
    assert!(
        !output.reasons.contains(&ReliabilityReason::FallbackUsed),
        "Specialist Ok should not include FallbackUsed reason"
    );

    // Should be perfect score
    assert_eq!(output.score, 100);
}

/// Specialist Skipped (normal deterministic routing) → No FallbackUsed penalty
#[test]
fn golden_fallback_deterministic_route_not_penalized() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        answer_grounded: true,
        specialist_outcome: Some(SpecialistOutcome::Skipped),
        fallback_used: Some(FallbackUsed::Deterministic {
            route_class: "memory_usage".to_string(),
        }),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // GUARDRAIL: Skipped specialist should NOT trigger FallbackUsed penalty
    assert!(
        !output.reasons.contains(&ReliabilityReason::FallbackUsed),
        "Normal deterministic routing (Skipped) should NOT be penalized"
    );

    // Should be perfect score
    assert_eq!(
        output.score, 100,
        "Deterministic routing should not be penalized"
    );
}

/// Error + fallback → FallbackUsed penalty applies
#[test]
fn golden_fallback_error_with_fallback() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        answer_grounded: true,
        specialist_outcome: Some(SpecialistOutcome::Error),
        fallback_used: Some(FallbackUsed::Deterministic {
            route_class: "disk_usage".to_string(),
        }),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    assert!(
        output.reasons.contains(&ReliabilityReason::FallbackUsed),
        "Error with fallback should include FallbackUsed reason"
    );
    assert_eq!(output.score, 95);
}

/// BudgetExceeded + fallback → Both penalties apply
#[test]
fn golden_fallback_budget_exceeded_with_fallback() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        answer_grounded: true,
        budget_exceeded: true,
        exceeded_stage: Some("specialist".to_string()),
        specialist_outcome: Some(SpecialistOutcome::BudgetExceeded),
        fallback_used: Some(FallbackUsed::Deterministic {
            route_class: "cpu_info".to_string(),
        }),
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Should have both BudgetExceeded and FallbackUsed
    assert!(output.reasons.contains(&ReliabilityReason::BudgetExceeded));
    assert!(output.reasons.contains(&ReliabilityReason::FallbackUsed));

    // Score: 100 - 15 (budget) - 5 (fallback) = 80
    assert_eq!(output.score, 80, "Should apply both penalties");
}

/// FallbackUsed explanation includes evidence kinds when present
#[test]
fn golden_fallback_explanation_with_evidence() {
    let input = ReliabilityInput {
        planned_probes: 1,
        succeeded_probes: 1,
        no_invention: true,
        specialist_outcome: Some(SpecialistOutcome::Timeout),
        fallback_used: Some(FallbackUsed::Deterministic {
            route_class: "memory_usage".to_string(),
        }),
        used_deterministic_fallback: true,
        fallback_route_class: "memory_usage".to_string(),
        evidence_kinds: vec!["memory".to_string()],
        // Ensure score < 80 by adding multiple penalties
        prompt_truncated: true,
        transcript_capped: true,
        translator_used: true,
        translator_confidence: 0.6, // Low confidence: -20
        ..Default::default()
    };

    let output = compute_reliability(&input);
    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);

    assert!(
        explanation.is_some(),
        "Should have explanation when score < 80"
    );
    let explanation = explanation.unwrap();

    // Find FallbackUsed reason
    let fallback_reason = explanation
        .reasons
        .iter()
        .find(|r| r.code == ReliabilityReason::FallbackUsed);
    assert!(fallback_reason.is_some());

    let details = &fallback_reason.unwrap().details;
    assert!(
        details.contains("memory"),
        "Should mention evidence kind: {}",
        details
    );
    assert!(
        details.contains("memory_usage"),
        "Should mention route class: {}",
        details
    );
}
