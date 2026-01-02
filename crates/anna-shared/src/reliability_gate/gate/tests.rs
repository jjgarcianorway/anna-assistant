//! Tests for the reliability gate.

use super::*;
use crate::reliability_gate::claim_evidence::{
    ClaimType, EvidenceBinding, EvidenceType, StrictClaim, StrictEvidence,
};

#[test]
fn test_gate_pass_with_evidence() {
    let gate = ReliabilityGate::default();

    let mut binding = EvidenceBinding::new("REQ-001");
    let claim = StrictClaim::new("C1", "Free RAM is 17 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);

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

    let input = GateInput::new("REQ-001").with_binding(binding);
    let result = gate.evaluate(&input);

    assert!(result.passed());
    assert_eq!(result.outcome, GateOutcome::Pass);
}

#[test]
fn test_gate_fail_no_evidence() {
    let gate = ReliabilityGate::default();

    let mut binding = EvidenceBinding::new("REQ-001");
    let claim = StrictClaim::new("C1", "Free RAM is 17 GiB", ClaimType::Metric, "system");
    binding.add_claim(claim);
    // No evidence added

    let input = GateInput::new("REQ-001").with_binding(binding);
    let result = gate.evaluate(&input);

    assert!(!result.passed());
    assert_eq!(result.outcome, GateOutcome::FailedNoEvidence);
}

#[test]
fn test_gate_fail_timeout() {
    let gate = ReliabilityGate::default();
    let input = GateInput::new("REQ-001").with_timeout();
    let result = gate.evaluate(&input);

    assert!(!result.passed());
    assert_eq!(result.outcome, GateOutcome::FailedTimeout);
}

#[test]
fn test_gate_fail_parse_error() {
    let gate = ReliabilityGate::default();
    let input = GateInput::new("REQ-001").with_parse_error();
    let result = gate.evaluate(&input);

    assert!(!result.passed());
    assert_eq!(result.outcome, GateOutcome::FailedParse);
}

#[test]
fn test_gate_fail_generic_answer() {
    let gate = ReliabilityGate::default();
    let input = GateInput::new("REQ-001").with_generic_answer();
    let result = gate.evaluate(&input);

    assert!(!result.passed());
    assert_eq!(result.outcome, GateOutcome::FailedGenericAnswer);
}
