//! Golden tests for reliability scoring.
//!
//! These tests lock exact behavior. Changes require explicit approval.

use anna_shared::reliability::{
    compute_reliability, query_requires_evidence, ProbeHealth, ReliabilityExplanation,
    ReliabilityInput, ReliabilityOutput, ReliabilityReason, EXPLANATION_THRESHOLD,
};

// === GOLDEN TESTS: Scoring function ===

/// GOLDEN: probe_timeout_partial
/// planned=3, success=2, timeout=1, evidence_required=true
#[test]
fn golden_probe_timeout_partial() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        failed_probes: 0,
        timed_out_probes: 1,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // coverage_penalty = (1 - 2/3) * 30 = 10
    // timeout_penalty = 10
    // total = 100 - 10 - 10 = 80
    assert_eq!(output.score, 80, "Expected score 80");
    assert_eq!(output.probe_health, ProbeHealth::Partial);
    assert!(output.reasons.contains(&ReliabilityReason::ProbeTimeout));
    assert!(output.reasons.contains(&ReliabilityReason::ProbeFailed));
}

/// GOLDEN: probe_exitcode_partial
/// planned=3, success=2, fail=1 (no timeout)
#[test]
fn golden_probe_exitcode_partial() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        failed_probes: 1,
        timed_out_probes: 0,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // coverage_penalty = (1 - 2/3) * 30 = 10
    // no timeout penalty
    // total = 100 - 10 = 90
    assert_eq!(output.score, 90, "Expected score 90");
    assert_eq!(output.probe_health, ProbeHealth::Partial);
    assert!(output.reasons.contains(&ReliabilityReason::ProbeFailed));
    assert!(!output.reasons.contains(&ReliabilityReason::ProbeTimeout));
}

/// GOLDEN: no_probes_but_required
/// planned=0, evidence_required=true, grounded=false
#[test]
fn golden_no_probes_but_required() {
    let input = ReliabilityInput {
        planned_probes: 0,
        succeeded_probes: 0,
        failed_probes: 0,
        timed_out_probes: 0,
        answer_grounded: false,
        no_invention: true,
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // not_grounded (evidence_required): -30
    // evidence_missing (no probes): -25
    // total = 100 - 30 - 25 = 45
    assert_eq!(output.score, 45, "Expected score 45");
    assert_eq!(output.probe_health, ProbeHealth::NotNeeded);
    assert!(output.reasons.contains(&ReliabilityReason::EvidenceMissing));
    assert!(output.reasons.contains(&ReliabilityReason::NotGrounded));
}

/// GOLDEN: all_success_required (perfect score)
/// planned=3, success=3, grounded=true, no_invention=true
#[test]
fn golden_all_success_perfect() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        failed_probes: 0,
        timed_out_probes: 0,
        answer_grounded: true,
        no_invention: true,
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    assert_eq!(output.score, 100, "Expected perfect score 100");
    assert_eq!(output.probe_health, ProbeHealth::AllOk);
    assert!(output.reasons.is_empty(), "No degradation reasons");
}

/// GOLDEN: invention_ceiling
/// no_invention=false should cap at 40
#[test]
fn golden_invention_ceiling() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: false, // invention detected
        evidence_required: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    assert_eq!(output.score, 40, "Invention caps at 40");
    assert!(output.reasons.contains(&ReliabilityReason::InventionDetected));
}

/// GOLDEN: low_confidence_translator
#[test]
fn golden_low_confidence_translator() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        translator_used: true,
        translator_confidence: 0.5, // below 0.7
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // -20 for low confidence
    assert_eq!(output.score, 80, "Expected score 80");
    assert!(output.reasons.contains(&ReliabilityReason::LowConfidence));
}

/// GOLDEN: medium_confidence_translator
#[test]
fn golden_medium_confidence_translator() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        answer_grounded: true,
        no_invention: true,
        translator_used: true,
        translator_confidence: 0.75, // between 0.7 and 0.85
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // -10 for medium confidence
    assert_eq!(output.score, 90, "Expected score 90");
    assert!(output.reasons.contains(&ReliabilityReason::LowConfidence));
}

// === Non-golden tests ===

#[test]
fn test_probe_health_derivation() {
    let not_needed = compute_reliability(&ReliabilityInput {
        planned_probes: 0,
        ..Default::default()
    });
    assert_eq!(not_needed.probe_health, ProbeHealth::NotNeeded);

    let all_ok = compute_reliability(&ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 3,
        no_invention: true,
        answer_grounded: true,
        ..Default::default()
    });
    assert_eq!(all_ok.probe_health, ProbeHealth::AllOk);

    let partial = compute_reliability(&ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        no_invention: true,
        answer_grounded: true,
        ..Default::default()
    });
    assert_eq!(partial.probe_health, ProbeHealth::Partial);

    let none = compute_reliability(&ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 0,
        no_invention: true,
        answer_grounded: true,
        ..Default::default()
    });
    assert_eq!(none.probe_health, ProbeHealth::None);
}

#[test]
fn test_query_requires_evidence() {
    assert!(query_requires_evidence(
        "what processes are using the most memory?"
    ));
    assert!(query_requires_evidence("how much disk space is left?"));
    assert!(query_requires_evidence("what's my IP address?"));
    assert!(!query_requires_evidence("hello"));
    assert!(!query_requires_evidence("thanks for your help"));
}

#[test]
fn test_primary_reason_priority() {
    let output = compute_reliability(&ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 2,
        timed_out_probes: 1,
        answer_grounded: false,
        no_invention: false,
        evidence_required: true,
        ..Default::default()
    });

    // InventionDetected has highest priority (0)
    assert_eq!(
        output.primary_reason(),
        Some(&ReliabilityReason::InventionDetected)
    );
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

/// GOLDEN: resource caps accumulate
#[test]
fn golden_resource_caps() {
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

    // -10 for prompt_truncated, -5 for transcript_capped
    assert_eq!(output.score, 85, "Expected score 85");
    assert!(output.reasons.contains(&ReliabilityReason::PromptTruncated));
    assert!(output.reasons.contains(&ReliabilityReason::TranscriptCapped));
}

/// GOLDEN: all failures combined
#[test]
fn golden_worst_case() {
    let input = ReliabilityInput {
        planned_probes: 3,
        succeeded_probes: 0,
        failed_probes: 2,
        timed_out_probes: 1,
        answer_grounded: false,
        no_invention: false, // hard ceiling
        evidence_required: true,
        translator_used: true,
        translator_confidence: 0.5,
        prompt_truncated: true,
        transcript_capped: true,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Invention ceiling caps at 40, then clamp to 0
    // But we still track all reasons
    assert!(output.score <= 40, "Should be capped by invention ceiling");
    assert!(output.reasons.contains(&ReliabilityReason::InventionDetected));
}

/// GOLDEN: translator_probe_conflict
/// Translator says "no probes" but query clearly requires evidence.
/// This simulates the pipeline detecting the conflict via query_requires_evidence.
#[test]
fn golden_translator_probe_conflict() {
    // Query: "what process is using the most memory?"
    // Translator (mistakenly): needs_probes = []
    // Pipeline heuristic detects: evidence_required = true

    let query = "what process is using the most memory?";

    // Verify heuristic catches this
    assert!(
        query_requires_evidence(query),
        "Heuristic should detect evidence requirement"
    );

    // Simulate what happens when translator doesn't request probes
    // but the answer still tries to respond (not grounded)
    let input = ReliabilityInput {
        planned_probes: 0,         // translator said none
        succeeded_probes: 0,
        answer_grounded: false,    // can't be grounded without probes
        no_invention: true,        // didn't invent, just couldn't answer
        evidence_required: true,   // heuristic detected this
        translator_used: true,
        translator_confidence: 0.6, // mediocre confidence
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Expected penalties:
    // - not_grounded (evidence_required): -30
    // - evidence_missing (no probes, evidence_required): -25
    // - low_confidence (< 0.7): -20
    // total = 100 - 30 - 25 - 20 = 25
    assert_eq!(output.score, 25, "Translator/probe conflict should heavily degrade score");
    assert!(output.reasons.contains(&ReliabilityReason::NotGrounded));
    assert!(output.reasons.contains(&ReliabilityReason::EvidenceMissing));
    assert!(output.reasons.contains(&ReliabilityReason::LowConfidence));
}

/// GOLDEN: translator_probe_conflict_with_invention
/// Same as above but answer also contains invention language.
/// Invention ceiling caps at 40, then other penalties accumulate below that.
#[test]
fn golden_translator_probe_conflict_with_invention() {
    let input = ReliabilityInput {
        planned_probes: 0,
        succeeded_probes: 0,
        answer_grounded: false,
        no_invention: false,       // LLM invented an answer
        evidence_required: true,
        translator_used: true,
        translator_confidence: 0.6,
        ..Default::default()
    };

    let output = compute_reliability(&input);

    // Scoring trace:
    // 1. Start at 100
    // 2. Invention ceiling: caps to 40
    // 3. Not grounded (evidence_required): -30 → 10
    // 4. Evidence missing (no probes): -25 → -15
    // 5. Low confidence (< 0.7): -20 → -35
    // 6. Clamp to 0-100 → 0
    assert_eq!(output.score, 0, "Multiple failures drive score to floor");
    assert_eq!(
        output.primary_reason(),
        Some(&ReliabilityReason::InventionDetected),
        "Invention should be primary reason (highest priority)"
    );
    // All reasons should be tracked
    assert!(output.reasons.contains(&ReliabilityReason::InventionDetected));
    assert!(output.reasons.contains(&ReliabilityReason::NotGrounded));
    assert!(output.reasons.contains(&ReliabilityReason::EvidenceMissing));
    assert!(output.reasons.contains(&ReliabilityReason::LowConfidence));
}

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
    assert!(explanation.is_none(), "Score exactly at threshold should yield None");
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
        prompt_truncated: true,       // -10 → score 80, still at threshold
        transcript_capped: true,      // -5 → score 75
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
    use anna_shared::resource_limits::{ResourceDiagnostic, ResourceKind};

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
    let timeout_reason = exp.reasons.iter().find(|r| r.code == ReliabilityReason::ProbeTimeout);
    assert!(timeout_reason.is_some());
    let tr = timeout_reason.unwrap();

    // Details should follow template: "{timed_out} of {planned} probes timed out"
    assert!(tr.details.contains("2 of 5"), "Should have templated probe counts");
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
    assert_eq!(output.score, 100, "No budget exceeded should mean no penalty");
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
    assert_eq!(output.score, 75, "Budget exceeded should subsume probe timeout");
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
    assert!(output.score < EXPLANATION_THRESHOLD, "Score should be below 80 for explanation");

    let explanation = ReliabilityExplanation::build(&output, &input, vec![]);
    assert!(explanation.is_some(), "Explanation should be generated");

    let exp = explanation.unwrap();

    // Find the budget exceeded reason
    let budget_reason = exp.reasons.iter().find(|r| r.code == ReliabilityReason::BudgetExceeded);
    assert!(budget_reason.is_some(), "BudgetExceeded reason should be present");

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

// =============================================================================
// v0.0.24: FallbackUsed guardrail tests
// =============================================================================

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
    assert_eq!(output.score, 95, "Timeout with fallback should apply -5 penalty");
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
    assert_eq!(output.score, 100, "Deterministic routing should not be penalized");
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

    assert!(explanation.is_some(), "Should have explanation when score < 80");
    let explanation = explanation.unwrap();

    // Find FallbackUsed reason
    let fallback_reason = explanation.reasons.iter()
        .find(|r| r.code == ReliabilityReason::FallbackUsed);
    assert!(fallback_reason.is_some());

    let details = &fallback_reason.unwrap().details;
    assert!(details.contains("memory"), "Should mention evidence kind: {}", details);
    assert!(details.contains("memory_usage"), "Should mention route class: {}", details);
}
