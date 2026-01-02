//! Golden tests for reliability scoring - Probe-related tests.
//!
//! These tests lock exact behavior. Changes require explicit approval.

use anna_shared::reliability::{
    compute_reliability, query_requires_evidence, ProbeHealth, ReliabilityInput,
    ReliabilityReason,
};

// === GOLDEN TESTS: Scoring function - Probe scenarios ===

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
    assert!(output
        .reasons
        .contains(&ReliabilityReason::InventionDetected));
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
    assert!(output
        .reasons
        .contains(&ReliabilityReason::TranscriptCapped));
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
    assert!(output
        .reasons
        .contains(&ReliabilityReason::InventionDetected));
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
        planned_probes: 0, // translator said none
        succeeded_probes: 0,
        answer_grounded: false,  // can't be grounded without probes
        no_invention: true,      // didn't invent, just couldn't answer
        evidence_required: true, // heuristic detected this
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
    assert_eq!(
        output.score, 25,
        "Translator/probe conflict should heavily degrade score"
    );
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
        no_invention: false, // LLM invented an answer
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
    assert!(output
        .reasons
        .contains(&ReliabilityReason::InventionDetected));
    assert!(output.reasons.contains(&ReliabilityReason::NotGrounded));
    assert!(output.reasons.contains(&ReliabilityReason::EvidenceMissing));
    assert!(output.reasons.contains(&ReliabilityReason::LowConfidence));
}
