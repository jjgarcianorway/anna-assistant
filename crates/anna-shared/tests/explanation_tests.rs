//! Golden tests for reliability scoring - TRUST Explanation tests.
//!
//! These tests lock exact behavior. Changes require explicit approval.

use anna_shared::reliability::{
    compute_reliability, ProbeHealth, ReliabilityExplanation, ReliabilityInput,
    ReliabilityOutput, ReliabilityReason, EXPLANATION_THRESHOLD,
};
use anna_shared::resource_limits::{ResourceDiagnostic, ResourceKind};

// === GOLDEN TESTS: TRUST Explanations ===

/// GOLDEN: score >= 80 yields None explanation
#[test]
fn golden_trust_high_score_no_explanation() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert_eq!(output.score, 100);

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_none(), "Score >= 80 should yield None");
}

/// GOLDEN: score at exactly threshold yields None
#[test]
fn golden_trust_exact_threshold_no_explanation() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        translator_used: true,
        translator_confidence: 0.5, // -20
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert_eq!(output.score, EXPLANATION_THRESHOLD); // 80

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(
        explanation.is_none(),
        "Score exactly at threshold should yield None"
    );
}

/// GOLDEN: single reason explanation
#[test]
fn golden_trust_single_reason() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        translator_used: true,
        translator_confidence: 0.75, // -10 → score 90, then another -10 needed
        prompt_truncated: true,      // -10 → score 80, still at threshold
        transcript_capped: true,     // -5 → score 75
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert_eq!(output.score, 75);

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some());

    let exp = explanation.unwrap();
    assert_eq!(exp.score, 75);
    assert!(!exp.reasons.is_empty());
    assert!(exp.summary.contains("75"));
}

/// GOLDEN: multi-reason ordering by priority
#[test]
fn golden_trust_multi_reason_ordering() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 1,
        timed_out_probes: 1,
        failed_probes: 1,
        answer_grounded: false,
        no_invention: true,
        evidence_required: true,
        translator_used: true,
        translator_confidence: 0.5,
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert!(output.score < 80);

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some());

    let exp = explanation.unwrap();
    // Reasons should be sorted by priority: EvidenceMissing, ProbeTimeout, ProbeFailed, etc.
    assert!(exp.reasons.len() >= 2);

    // First reason should be highest priority (lower priority number)
    if exp.reasons.len() >= 2 {
        assert!(
            exp.reasons[0].code.priority() <= exp.reasons[1].code.priority(),
            "Reasons should be sorted by priority"
        );
    }
}

/// GOLDEN: invention ceiling explicitly reported
#[test]
fn golden_trust_invention_ceiling() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: false, // invention detected
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert_eq!(output.score, 40);

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some());

    let exp = explanation.unwrap();
    assert_eq!(exp.score, 40);

    // First reason should be InventionDetected (priority 0)
    assert_eq!(exp.reasons[0].code, ReliabilityReason::InventionDetected);
    // Should have no penalty (it's a ceiling, not a deduction)
    assert!(exp.reasons[0].penalty.is_none());
    // Summary should mention ceiling
    assert!(exp.summary.contains("capped at 40"));
}

/// GOLDEN: COST caps appear in explanation
#[test]
fn golden_trust_cost_caps_integration() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        prompt_truncated: true,
        transcript_capped: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);
    assert_eq!(output.score, 85);

    // Simulate diagnostics from COST phase
    let diagnostics = vec![ResourceDiagnostic::transcript_capped(5)];

    let explanation = ReliabilityExplanation::build(&output, &input, diagnostics);
    // Score 85 >= 80, so no explanation
    assert!(explanation.is_none());

    // Now test with lower score
    let input_low = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        prompt_truncated: true,
        transcript_capped: true,
        translator_used: true,
        translator_confidence: 0.5, // -20 → 65
        ..Default::default()
    };

    let output_low = compute_reliability(&input_low);
    assert!(output_low.score < 80);

    let explanation = ReliabilityExplanation::build(
        &output_low,
        &input_low,
        vec![ResourceDiagnostic::transcript_capped(5)],
    );
    assert!(explanation.is_some());

    let exp = explanation.unwrap();
    assert_eq!(exp.diagnostics.len(), 1);
    assert_eq!(exp.diagnostics[0].kind, ResourceKind::TranscriptEvents);
}

/// GOLDEN: templated details are deterministic
#[test]
fn golden_trust_templated_details() {
    let input = ReliabilityInput {
        planned_probes: 5,
        succeeded_probes: 3,
        timed_out_probes: 2,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);
    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some());

    let exp = explanation.unwrap();

    // Find the timeout reason
    let timeout_reason = exp
        .reasons
        .iter()
        .find(|r| r.code == ReliabilityReason::ProbeTimeout);
    assert!(timeout_reason.is_some());
    let tr = timeout_reason.unwrap();

    // Details should follow template: "{timed_out} of {planned} probes timed out"
    assert!(
        tr.details.contains("2 of 5"),
        "Should have templated probe counts"
    );
}

/// GOLDEN: deduplication of reasons
#[test]
fn golden_trust_reason_deduplication() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 1,
        timed_out_probes: 1,
        failed_probes: 1,
        answer_grounded: true,
        no_invention: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);
    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);

    if let Some(exp) = explanation {
        // Each reason code should appear at most once
        let mut seen = std::collections::HashSet::new();
        for reason in &exp.reasons {
            assert!(
                seen.insert(reason.code),
                "Reason {:?} should not be duplicated",
                reason.code
            );
        }
    }
}

#[test]
fn test_explanation_threshold() {
    let high_score = ReliabilityOutput {
        score: 85,
        reasons: vec![ReliabilityReason::LowConfidence],
        breakdown: vec![],
        probe_health: ProbeHealth::AllOk,
        probe_coverage_ratio: 1.0,
    };
    assert!(high_score.explanation(80).is_none());

    let low_score = ReliabilityOutput {
        score: 75,
        reasons: vec![ReliabilityReason::ProbeFailed],
        breakdown: vec![],
        probe_health: ProbeHealth::Partial,
        probe_coverage_ratio: 0.67,
    };
    assert_eq!(low_score.explanation(80), Some("probe failed".to_string()));
}
