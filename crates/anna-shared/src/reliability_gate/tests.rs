//! Acceptance tests for Hard Reliability Gate (v0.0.445).

use super::answer_contract::*;
use super::claim_evidence::*;
use super::deterministic::*;
use super::gate::*;

// ============================================================================
// PART 7 - Acceptance Tests
// ============================================================================

/// Test: "do I have swap" returns only yes/no (+ evidence).
#[test]
fn test_swap_question_returns_boolean_only() {
    let contract = AnswerContract::from_question("do I have swap", "system");
    assert_eq!(contract.shape, AnswerShape::BooleanWithReason);

    let mut binding = EvidenceBinding::new("REQ-001");

    // Valid: single boolean claim
    let claim = StrictClaim::new("C1", "Yes, swap is enabled", ClaimType::Boolean, "system");
    binding.add_claim(claim);

    let violation = contract.validate_answer(&binding);
    assert!(violation.is_none());

    // Invalid: metric claim for boolean question
    let mut binding2 = EvidenceBinding::new("REQ-002");
    let wrong_claim = StrictClaim::new("C1", "Swap is 8 GiB", ClaimType::Metric, "system");
    binding2.add_claim(wrong_claim);

    let violation = contract.validate_answer(&binding2);
    assert!(matches!(
        violation,
        Some(ContractViolation::WrongClaimType { .. })
    ));
}

/// Test: "is nano installed" never returns package lists.
#[test]
fn test_installed_check_returns_boolean_not_list() {
    let contract = AnswerContract::from_question("is nano installed", "packages");
    assert_eq!(contract.shape, AnswerShape::BooleanWithReason);

    let mut binding = EvidenceBinding::new("REQ-001");

    // Invalid: list claim for boolean question
    let list_claim = StrictClaim::new(
        "C1",
        "Installed packages: vim, nano, emacs",
        ClaimType::List,
        "packages",
    );
    binding.add_claim(list_claim);

    let violation = contract.validate_answer(&binding);
    assert!(matches!(
        violation,
        Some(ContractViolation::WrongClaimType { .. })
    ));
}

/// Test: Any timeout results in FAILED_TIMEOUT, not partial info.
#[test]
fn test_timeout_results_in_failed_timeout() {
    let gate = ReliabilityGate::default();

    // Even with valid claims/evidence, timeout should fail
    let mut binding = EvidenceBinding::new("REQ-001");
    let claim = StrictClaim::new("C1", "RAM is 16 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);

    let evidence = StrictEvidence::new(
        "E1",
        "probe:meminfo",
        "cat /proc/meminfo",
        "MemTotal: 16777216 kB",
        EvidenceType::Numeric,
        "REQ-001",
    );
    binding.add_evidence(evidence);
    binding.bind("C1", "E1");

    let input = GateInput::new("REQ-001")
        .with_binding(binding)
        .with_timeout();

    let result = gate.evaluate(&input);
    assert_eq!(result.outcome, GateOutcome::FailedTimeout);
    assert!(!result.passed());
}

/// Test: Stats after failures show 0 resolved.
#[test]
fn test_failure_outcomes_not_counted_as_success() {
    // All failure outcomes should NOT be considered success
    let failures = vec![
        GateOutcome::FailedNoEvidence,
        GateOutcome::FailedTimeout,
        GateOutcome::FailedParse,
        GateOutcome::FailedLowConfidence,
        GateOutcome::FailedAmbiguousQuery,
        GateOutcome::FailedContractViolation,
        GateOutcome::FailedNoClaims,
        GateOutcome::FailedGenericAnswer,
    ];

    for outcome in failures {
        assert!(!outcome.is_success(), "{:?} should not be success", outcome);
    }

    // Only Pass is success
    assert!(GateOutcome::Pass.is_success());
}

/// Test: No claims results in failure.
#[test]
fn test_no_claims_fails_gate() {
    let gate = ReliabilityGate::default();
    let binding = EvidenceBinding::new("REQ-001");
    // No claims added

    let input = GateInput::new("REQ-001").with_binding(binding);
    let result = gate.evaluate(&input);

    assert_eq!(result.outcome, GateOutcome::FailedNoClaims);
}

/// Test: Claims without evidence fail gate.
#[test]
fn test_claims_without_evidence_fails_gate() {
    let gate = ReliabilityGate::default();

    let mut binding = EvidenceBinding::new("REQ-001");
    let claim = StrictClaim::new("C1", "RAM is 16 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);
    // No evidence added

    let input = GateInput::new("REQ-001").with_binding(binding);
    let result = gate.evaluate(&input);

    assert_eq!(result.outcome, GateOutcome::FailedNoEvidence);
}

/// Test: Generic fallback answer is rejected.
#[test]
fn test_generic_answer_rejected() {
    let gate = ReliabilityGate::default();

    let input = GateInput::new("REQ-001").with_generic_answer();
    let result = gate.evaluate(&input);

    assert_eq!(result.outcome, GateOutcome::FailedGenericAnswer);
}

/// Test: Parse error fails gate.
#[test]
fn test_parse_error_fails_gate() {
    let gate = ReliabilityGate::default();

    let input = GateInput::new("REQ-001").with_parse_error();
    let result = gate.evaluate(&input);

    assert_eq!(result.outcome, GateOutcome::FailedParse);
}

/// Test: Low confidence fails gate.
#[test]
fn test_low_confidence_fails_gate() {
    let gate = ReliabilityGate::with_min_confidence(0.7);

    let mut binding = EvidenceBinding::new("REQ-001");
    let claim = StrictClaim::new("C1", "RAM is 16 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);
    let evidence = StrictEvidence::new(
        "E1",
        "probe:meminfo",
        "cat /proc/meminfo",
        "MemTotal: 16777216 kB",
        EvidenceType::Numeric,
        "REQ-001",
    );
    binding.add_evidence(evidence);
    binding.bind("C1", "E1");

    let input = GateInput::new("REQ-001")
        .with_binding(binding)
        .with_confidence(0.5); // Below 0.7 threshold

    let result = gate.evaluate(&input);
    assert_eq!(result.outcome, GateOutcome::FailedLowConfidence);
}

/// Test: Deterministic routing for simple RAM query.
#[test]
fn test_deterministic_ram_query() {
    let policy = DeterministicPolicy::new();

    assert!(policy.can_skip_llm("how much ram do I have"));
    assert!(policy.can_skip_llm("how much free memory"));
    assert!(policy.can_skip_llm("available memory"));

    let probes = policy.get_probes("how much ram");
    assert!(probes.is_some());
}

/// Test: Deterministic routing for swap query.
#[test]
fn test_deterministic_swap_query() {
    let policy = DeterministicPolicy::new();

    assert!(policy.can_skip_llm("do I have swap"));
    assert!(policy.can_skip_llm("is swap enabled"));

    let probes = policy.get_probes("do I have swap");
    assert!(probes.is_some());
}

/// Test: LLM required for diagnosis/explanation.
#[test]
fn test_llm_required_for_diagnosis() {
    let policy = DeterministicPolicy::new();

    assert!(!policy.can_skip_llm("why is my system slow"));
    assert!(!policy.can_skip_llm("explain how systemd works"));
    assert!(!policy.can_skip_llm("help me fix nginx"));
}

/// Test: Evidence freshness validation.
#[test]
fn test_evidence_freshness() {
    let mut binding = EvidenceBinding::new("REQ-002");

    let claim = StrictClaim::new("C1", "RAM is 16 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);

    // Evidence from old request
    let stale_evidence = StrictEvidence::new(
        "E1",
        "probe:meminfo",
        "cat /proc/meminfo",
        "MemTotal: 16777216 kB",
        EvidenceType::Numeric,
        "REQ-001", // Different request ID
    );
    binding.add_evidence(stale_evidence);

    // Binding should fail due to stale evidence
    assert!(!binding.bind("C1", "E1"));
    assert!(!binding.all_claims_bound());
}

/// Test: Diagnosis claims require multiple evidence sources.
#[test]
fn test_diagnosis_requires_multiple_sources() {
    let mut binding = EvidenceBinding::new("REQ-001");

    let claim = StrictClaim::new(
        "C1",
        "System is slow due to memory pressure",
        ClaimType::Diagnosis,
        "system",
    );
    binding.add_claim(claim);

    // First evidence source
    let e1 = StrictEvidence::new(
        "E1",
        "probe:memory",
        "free -h",
        "available: 500M",
        EvidenceType::MultiSource,
        "REQ-001",
    );
    binding.add_evidence(e1);
    binding.bind("C1", "E1");

    // Not enough - diagnosis needs 2+ sources
    assert!(!binding.all_claims_bound());

    // Second evidence source
    let e2 = StrictEvidence::new(
        "E2",
        "probe:vmstat",
        "vmstat 1 1",
        "swpd: 1000000",
        EvidenceType::MultiSource,
        "REQ-001",
    );
    binding.add_evidence(e2);
    binding.bind("C1", "E2");

    // Now satisfied
    assert!(binding.all_claims_bound());
}

/// Test: Answer contract for metric questions.
#[test]
fn test_metric_answer_contract() {
    let contract = AnswerContract::from_question("how much free RAM do I have", "system");
    assert_eq!(contract.shape, AnswerShape::SingleMetric);

    let mut binding = EvidenceBinding::new("REQ-001");

    // Valid: single metric claim
    let claim = StrictClaim::new("C1", "17.0 GiB available", ClaimType::Metric, "system");
    binding.add_claim(claim);

    let violation = contract.validate_answer(&binding);
    assert!(violation.is_none());

    // Invalid: too many claims
    binding.add_claim(StrictClaim::new("C2", "8 GiB used", ClaimType::Metric, "system"));
    let violation = contract.validate_answer(&binding);
    assert!(matches!(
        violation,
        Some(ContractViolation::TooManyClaims { .. })
    ));
}

/// Test: Generic content detection.
#[test]
fn test_generic_content_detection() {
    // Generic system info when asking about services
    let result = detect_generic_content(
        "System Information:\nCPU: Intel\nMemory: 16GB\nDisk: 500GB",
        "services",
    );
    assert!(result.is_some());

    // Relevant content for system domain
    let result = detect_generic_content("Memory Info: 16GB total, 8GB available", "system");
    assert!(result.is_none());
}

/// Test: Full gate flow with valid input.
#[test]
fn test_full_gate_flow_pass() {
    let gate = ReliabilityGate::default();

    let mut binding = EvidenceBinding::new("REQ-001");

    // Add claim
    let claim = StrictClaim::new("C1", "17.0 GiB available", ClaimType::Metric, "system");
    binding.add_claim(claim);

    // Add matching evidence
    let evidence = StrictEvidence::new(
        "E1",
        "probe:meminfo",
        "cat /proc/meminfo",
        "MemAvailable: 17848320 kB",
        EvidenceType::Numeric,
        "REQ-001",
    );
    binding.add_evidence(evidence);
    binding.bind("C1", "E1");

    // Create contract
    let contract = AnswerContract::from_question("how much free RAM", "system");

    let input = GateInput::new("REQ-001")
        .with_binding(binding)
        .with_contract(contract)
        .with_confidence(0.9);

    let result = gate.evaluate(&input);

    assert!(result.passed());
    assert_eq!(result.outcome, GateOutcome::Pass);
    assert!(result.evidence_coverage > 0.99);
}

/// Test: Gate failure message content.
#[test]
fn test_failure_messages_are_user_friendly() {
    // All failure messages should be user-friendly (no technical jargon)
    let outcomes = vec![
        GateOutcome::FailedNoEvidence,
        GateOutcome::FailedTimeout,
        GateOutcome::FailedParse,
        GateOutcome::FailedLowConfidence,
        GateOutcome::FailedAmbiguousQuery,
        GateOutcome::FailedNoClaims,
        GateOutcome::FailedGenericAnswer,
    ];

    for outcome in outcomes {
        let msg = outcome.failure_message();
        assert!(!msg.is_empty());
        assert!(!msg.contains("null"));
        assert!(!msg.contains("error:"));
        assert!(!msg.contains("panic"));
    }
}
