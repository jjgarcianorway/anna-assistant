//! Golden tests for reliability scoring - METER phase budget tests.
//!
//! These tests lock exact behavior. Changes require explicit approval.

use anna_shared::reliability::{
    compute_reliability, ReliabilityExplanation, ReliabilityInput, ReliabilityReason,
    EXPLANATION_THRESHOLD,
};

// === METER PHASE: Budget exceeded golden tests ===

/// GOLDEN: Stage budget exceeded triggers BudgetExceeded outcome and penalty
/// Penalty: -15 (locked)
#[test]
fn golden_meter_budget_exceeded_penalty() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        budget_exceeded: true,
        exceeded_stage: Some("probes".to_string()),
        stage_budget_ms: 12_000,
        stage_elapsed_ms: 15_000,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Expected: 100 - 15 (budget_exceeded) = 85
    assert_eq!(output.score, 85, "Budget exceeded penalty should be -15");
    assert!(output.reasons.contains(&ReliabilityReason::BudgetExceeded));
}

/// GOLDEN: All stages within budget triggers no penalty
#[test]
fn golden_meter_within_budget_no_penalty() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        budget_exceeded: false,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Expected: 100 (no penalties)
    assert_eq!(
        output.score, 100,
        "No budget exceeded should mean no penalty"
    );
    assert!(!output.reasons.contains(&ReliabilityReason::BudgetExceeded));
}

/// GOLDEN: Probe timeout without stage budget exceeded triggers ProbeTimeout only
#[test]
fn golden_meter_timeout_without_budget_exceeded() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        timed_out_probes: 1,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        budget_exceeded: false, // Stage budget NOT exceeded
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Expected: 100 - 10 (coverage) - 10 (timeout) = 80
    assert_eq!(output.score, 80, "Timeout without budget exceeded");
    assert!(output.reasons.contains(&ReliabilityReason::ProbeTimeout));
    assert!(!output.reasons.contains(&ReliabilityReason::BudgetExceeded));
}

/// GOLDEN: Probe timeout WITH stage budget exceeded triggers BudgetExceeded ONLY (subsumption)
/// This is the NO DOUBLE PENALTY rule - BudgetExceeded subsumes ProbeTimeout
#[test]
fn golden_meter_budget_subsumes_timeout() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        timed_out_probes: 1,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        budget_exceeded: true, // Stage budget exceeded
        exceeded_stage: Some("probes".to_string()),
        stage_budget_ms: 12_000,
        stage_elapsed_ms: 18_000,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Expected: 100 - 15 (budget_exceeded) - 10 (coverage) = 75
    // ProbeTimeout penalty (-10) is NOT applied due to subsumption
    assert_eq!(
        output.score, 75,
        "Budget exceeded should subsume probe timeout"
    );
    assert!(
        output.reasons.contains(&ReliabilityReason::BudgetExceeded),
        "BudgetExceeded should be present"
    );
    assert!(
        !output.reasons.contains(&ReliabilityReason::ProbeTimeout),
        "ProbeTimeout should NOT be present (subsumed by BudgetExceeded)"
    );
}

/// GOLDEN: BudgetExceeded has higher priority than ProbeTimeout
#[test]
fn golden_meter_budget_priority() {
    // BudgetExceeded priority = 2
    // ProbeTimeout priority = 3
    assert!(
        ReliabilityReason::BudgetExceeded.priority() < ReliabilityReason::ProbeTimeout.priority(),
        "BudgetExceeded should have higher priority (lower number) than ProbeTimeout"
    );
}

/// GOLDEN: TRUST explanation includes BudgetExceeded templated details
#[test]
fn golden_meter_trust_explanation() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        answer_grounded: true,
        no_invention: true,
        budget_exceeded: true,
        exceeded_stage: Some("probes".to_string()),
        stage_budget_ms: 12_000,
        stage_elapsed_ms: 18_000,
        translator_used: true,
        translator_confidence: 0.6, // Additional penalty to ensure < 80
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert!(
        output.score < EXPLANATION_THRESHOLD,
        "Score should be below 80 for explanation"
    );

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some(), "Explanation should be generated");

    let exp = explanation.unwrap();

    // Find the budget exceeded reason
    let budget_reason = exp
        .reasons
        .iter()
        .find(|r| r.code == ReliabilityReason::BudgetExceeded);
    assert!(
        budget_reason.is_some(),
        "BudgetExceeded reason should be present"
    );

    let br = budget_reason.unwrap();
    // Template: "{stage} stage exceeded budget ({elapsed}ms > {budget}ms)"
    assert!(
        br.details.contains("probes"),
        "Details should include stage name: {}",
        br.details
    );
    assert!(
        br.details.contains("18000ms"),
        "Details should include elapsed time: {}",
        br.details
    );
    assert!(
        br.details.contains("12000ms"),
        "Details should include budget: {}",
        br.details
    );
}

/// GOLDEN: BudgetExceeded ordering is correct with existing priorities
#[test]
fn golden_meter_priority_ordering() {
    // Test the full priority chain
    let priorities = vec![
        (ReliabilityReason::InventionDetected, 0),
        (ReliabilityReason::EvidenceMissing, 1),
        (ReliabilityReason::BudgetExceeded, 2),
        (ReliabilityReason::ProbeTimeout, 3),
        (ReliabilityReason::ProbeFailed, 4),
        (ReliabilityReason::FallbackUsed, 5),
        (ReliabilityReason::PromptTruncated, 6),
        (ReliabilityReason::TranscriptCapped, 7),
        (ReliabilityReason::LowConfidence, 8),
        (ReliabilityReason::NotGrounded, 9),
    ];

    for (reason, expected_priority) in priorities {
        assert_eq!(
            reason.priority(),
            expected_priority,
            "{:?} should have priority {}",
            reason,
            expected_priority
        );
    }
}
