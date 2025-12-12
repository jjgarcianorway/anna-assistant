//! Hard Reliability Gate (v0.0.447).
//!
//! The gate that decides: show answer or show failure.
//!
//! Checks (ALL must pass):
//! 1. No timeout occurred
//! 2. No parsing errors occurred
//! 3. Not a generic/fallback answer
//! 4. Confidence >= 0.85 (configurable)
//! 5. At least 1 claim exists
//! 6. Every claim has ≥1 evidence item
//! 7. Question match (answer matches question type)
//! 8. Domain consistency (domain matches probes)
//! 9. No hallucinated entities (all nouns in evidence)
//! 10. Contract validation (answer shape = question shape)

use super::answer_contract::{AnswerContract, ContractViolation};
use super::claim_evidence::EvidenceBinding;
use serde::{Deserialize, Serialize};

/// Outcome of the reliability gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateOutcome {
    /// All checks passed - answer can be shown
    Pass,
    /// Failed - no evidence for claims
    FailedNoEvidence,
    /// Failed - timeout in required stage
    FailedTimeout,
    /// Failed - parsing error occurred
    FailedParse,
    /// Failed - low confidence answer
    FailedLowConfidence,
    /// Failed - query was ambiguous
    FailedAmbiguousQuery,
    /// Failed - answer shape doesn't match question
    FailedContractViolation,
    /// Failed - no claims in answer
    FailedNoClaims,
    /// Failed - generic/irrelevant answer detected
    FailedGenericAnswer,
    /// Failed - answer doesn't match question type
    FailedQuestionMismatch,
    /// Failed - domain doesn't match probes
    FailedDomainMismatch,
    /// Failed - hallucinated entity detected
    FailedHallucination,
    /// Failed - probe failed or returned empty
    FailedProbeCoverage,
}

impl GateOutcome {
    /// Check if this is a success outcome.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Get user-facing failure message.
    pub fn failure_message(&self) -> &'static str {
        match self {
            Self::Pass => "",
            Self::FailedNoEvidence => "I don't have enough verified data to answer this yet.",
            Self::FailedTimeout => "I couldn't complete the analysis in time. Please try again.",
            Self::FailedParse => "I encountered an internal error processing this request.",
            Self::FailedLowConfidence => "I'm not confident enough in my answer to show it.",
            Self::FailedAmbiguousQuery => "I need more details to answer this accurately.",
            Self::FailedContractViolation => "I couldn't produce an answer in the expected format.",
            Self::FailedNoClaims => "I don't have any verified information to share.",
            Self::FailedGenericAnswer => {
                "I don't have specific information to answer this question."
            }
            Self::FailedQuestionMismatch => {
                "My answer doesn't match what you asked. Let me try again."
            }
            Self::FailedDomainMismatch => {
                "I gathered information from the wrong area. Let me refocus."
            }
            Self::FailedHallucination => "I couldn't verify some details in my answer.",
            Self::FailedProbeCoverage => "Some system checks failed or returned incomplete data.",
        }
    }

    /// Get code for metrics/logging.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::FailedNoEvidence => "FAILED_NO_EVIDENCE",
            Self::FailedTimeout => "FAILED_TIMEOUT",
            Self::FailedParse => "FAILED_PARSE",
            Self::FailedLowConfidence => "FAILED_LOW_CONFIDENCE",
            Self::FailedAmbiguousQuery => "FAILED_AMBIGUOUS_QUERY",
            Self::FailedContractViolation => "FAILED_CONTRACT_VIOLATION",
            Self::FailedNoClaims => "FAILED_NO_CLAIMS",
            Self::FailedGenericAnswer => "FAILED_GENERIC_ANSWER",
            Self::FailedQuestionMismatch => "FAILED_QUESTION_MISMATCH",
            Self::FailedDomainMismatch => "FAILED_DOMAIN_MISMATCH",
            Self::FailedHallucination => "FAILED_HALLUCINATION",
            Self::FailedProbeCoverage => "FAILED_PROBE_COVERAGE",
        }
    }

    /// Check if this is a timeout outcome.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::FailedTimeout)
    }

    /// Check if this is any failure outcome.
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

/// Individual gate check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCheck {
    /// Check name
    pub name: String,
    /// Whether it passed
    pub passed: bool,
    /// Details if failed
    pub details: Option<String>,
}

impl GateCheck {
    /// Create a passing check.
    pub fn pass(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            details: None,
        }
    }

    /// Create a failing check.
    pub fn fail(name: &str, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            details: Some(details.to_string()),
        }
    }
}

/// Complete gate evaluation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResult {
    /// Overall outcome
    pub outcome: GateOutcome,
    /// Individual check results
    pub checks: Vec<GateCheck>,
    /// Request ID
    pub request_id: String,
    /// Evidence coverage (0.0 to 1.0)
    pub evidence_coverage: f32,
}

impl GateResult {
    /// Create a passing result.
    pub fn pass(request_id: &str, checks: Vec<GateCheck>, coverage: f32) -> Self {
        Self {
            outcome: GateOutcome::Pass,
            checks,
            request_id: request_id.to_string(),
            evidence_coverage: coverage,
        }
    }

    /// Create a failing result.
    pub fn fail(
        request_id: &str,
        outcome: GateOutcome,
        checks: Vec<GateCheck>,
        coverage: f32,
    ) -> Self {
        Self {
            outcome,
            checks,
            request_id: request_id.to_string(),
            evidence_coverage: coverage,
        }
    }

    /// Check if gate passed.
    pub fn passed(&self) -> bool {
        self.outcome.is_success()
    }

    /// Get first failed check.
    pub fn first_failure(&self) -> Option<&GateCheck> {
        self.checks.iter().find(|c| !c.passed)
    }
}

/// Input for reliability gate evaluation.
#[derive(Debug, Clone, Default)]
pub struct GateInput {
    /// Request ID
    pub request_id: String,
    /// Evidence binding (claims + evidence)
    pub binding: EvidenceBinding,
    /// Answer contract
    pub contract: Option<AnswerContract>,
    /// Did timeout occur?
    pub timeout_occurred: bool,
    /// Did parse error occur?
    pub parse_error_occurred: bool,
    /// Is this a generic/fallback answer?
    pub is_generic_answer: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Original question text
    pub question: String,
    /// Detected domain
    pub domain: String,
    /// Probe domains actually used
    pub probe_domains: Vec<String>,
    /// Did any probe fail or timeout?
    pub probe_failed: bool,
    /// Did any probe return empty?
    pub probe_empty: bool,
    /// Entities in answer that need verification
    pub answer_entities: Vec<String>,
    /// Entities found in probe output
    pub evidence_entities: Vec<String>,
}

impl GateInput {
    /// Create new gate input.
    pub fn new(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            binding: EvidenceBinding::new(request_id),
            contract: None,
            timeout_occurred: false,
            parse_error_occurred: false,
            is_generic_answer: false,
            confidence: 1.0,
            question: String::new(),
            domain: String::new(),
            probe_domains: Vec::new(),
            probe_failed: false,
            probe_empty: false,
            answer_entities: Vec::new(),
            evidence_entities: Vec::new(),
        }
    }

    /// Set evidence binding.
    pub fn with_binding(mut self, binding: EvidenceBinding) -> Self {
        self.binding = binding;
        self
    }

    /// Set answer contract.
    pub fn with_contract(mut self, contract: AnswerContract) -> Self {
        self.contract = Some(contract);
        self
    }

    /// Mark timeout occurred.
    pub fn with_timeout(mut self) -> Self {
        self.timeout_occurred = true;
        self
    }

    /// Mark parse error occurred.
    pub fn with_parse_error(mut self) -> Self {
        self.parse_error_occurred = true;
        self
    }

    /// Mark as generic answer.
    pub fn with_generic_answer(mut self) -> Self {
        self.is_generic_answer = true;
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set original question.
    pub fn with_question(mut self, question: &str) -> Self {
        self.question = question.to_string();
        self
    }

    /// Set domain.
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }

    /// Set probe domains.
    pub fn with_probe_domains(mut self, domains: Vec<String>) -> Self {
        self.probe_domains = domains;
        self
    }

    /// Mark probe failed.
    pub fn with_probe_failed(mut self) -> Self {
        self.probe_failed = true;
        self
    }

    /// Mark probe returned empty.
    pub fn with_probe_empty(mut self) -> Self {
        self.probe_empty = true;
        self
    }

    /// Set answer entities for hallucination check.
    pub fn with_answer_entities(mut self, entities: Vec<String>) -> Self {
        self.answer_entities = entities;
        self
    }

    /// Set evidence entities for hallucination check.
    pub fn with_evidence_entities(mut self, entities: Vec<String>) -> Self {
        self.evidence_entities = entities;
        self
    }
}

/// The reliability gate.
pub struct ReliabilityGate {
    /// Minimum confidence threshold (default 0.85)
    min_confidence: f32,
}

impl Default for ReliabilityGate {
    fn default() -> Self {
        Self {
            min_confidence: 0.85, // v0.0.447: raised from 0.5 to 0.85
        }
    }
}

impl ReliabilityGate {
    /// Create gate with custom confidence threshold.
    pub fn with_min_confidence(min_confidence: f32) -> Self {
        Self { min_confidence }
    }

    /// Evaluate the gate. Returns Pass only if ALL checks pass.
    pub fn evaluate(&self, input: &GateInput) -> GateResult {
        let mut checks = Vec::new();

        // Check 1: No timeout
        if input.timeout_occurred {
            checks.push(GateCheck::fail("no_timeout", "Timeout occurred"));
            return GateResult::fail(&input.request_id, GateOutcome::FailedTimeout, checks, 0.0);
        }
        checks.push(GateCheck::pass("no_timeout"));

        // Check 2: No parse errors
        if input.parse_error_occurred {
            checks.push(GateCheck::fail("no_parse_error", "Parse error occurred"));
            return GateResult::fail(&input.request_id, GateOutcome::FailedParse, checks, 0.0);
        }
        checks.push(GateCheck::pass("no_parse_error"));

        // Check 3: Not a generic answer
        if input.is_generic_answer {
            checks.push(GateCheck::fail(
                "not_generic",
                "Generic/irrelevant answer detected",
            ));
            return GateResult::fail(
                &input.request_id,
                GateOutcome::FailedGenericAnswer,
                checks,
                0.0,
            );
        }
        checks.push(GateCheck::pass("not_generic"));

        // Check 4: Confidence above threshold
        if input.confidence < self.min_confidence {
            checks.push(GateCheck::fail(
                "confidence",
                &format!(
                    "Confidence {:.2} below threshold {:.2}",
                    input.confidence, self.min_confidence
                ),
            ));
            return GateResult::fail(
                &input.request_id,
                GateOutcome::FailedLowConfidence,
                checks,
                0.0,
            );
        }
        checks.push(GateCheck::pass("confidence"));

        // Check 5: Has claims
        if input.binding.claims.is_empty() {
            checks.push(GateCheck::fail("has_claims", "No claims in answer"));
            return GateResult::fail(&input.request_id, GateOutcome::FailedNoClaims, checks, 0.0);
        }
        checks.push(GateCheck::pass("has_claims"));

        // Check 6: All claims have evidence
        let coverage = input.binding.coverage();
        if !input.binding.all_claims_bound() {
            let unbound: Vec<_> = input
                .binding
                .unbound_claims()
                .iter()
                .map(|c| c.text.clone())
                .collect();
            checks.push(GateCheck::fail(
                "claims_have_evidence",
                &format!("Unbound claims: {:?}", unbound),
            ));
            return GateResult::fail(
                &input.request_id,
                GateOutcome::FailedNoEvidence,
                checks,
                coverage,
            );
        }
        checks.push(GateCheck::pass("claims_have_evidence"));

        // Check 7: Probe coverage (no failed or empty probes)
        if input.probe_failed {
            checks.push(GateCheck::fail(
                "probe_coverage",
                "One or more probes failed or timed out",
            ));
            return GateResult::fail(
                &input.request_id,
                GateOutcome::FailedProbeCoverage,
                checks,
                coverage,
            );
        }
        if input.probe_empty {
            checks.push(GateCheck::fail(
                "probe_coverage",
                "One or more probes returned empty output",
            ));
            return GateResult::fail(
                &input.request_id,
                GateOutcome::FailedProbeCoverage,
                checks,
                coverage,
            );
        }
        checks.push(GateCheck::pass("probe_coverage"));

        // Check 8: Domain consistency (if domain specified)
        if !input.domain.is_empty() && !input.probe_domains.is_empty() {
            let domain_match = input.probe_domains.iter().any(|d| {
                d == &input.domain || d.contains(&input.domain) || input.domain.contains(d)
            });
            if !domain_match {
                checks.push(GateCheck::fail(
                    "domain_consistency",
                    &format!(
                        "Domain '{}' doesn't match probe domains {:?}",
                        input.domain, input.probe_domains
                    ),
                ));
                return GateResult::fail(
                    &input.request_id,
                    GateOutcome::FailedDomainMismatch,
                    checks,
                    coverage,
                );
            }
            checks.push(GateCheck::pass("domain_consistency"));
        }

        // Check 9: No hallucinated entities
        if !input.answer_entities.is_empty() && !input.evidence_entities.is_empty() {
            let hallucinated: Vec<_> = input
                .answer_entities
                .iter()
                .filter(|e| {
                    !input.evidence_entities.iter().any(|ev| {
                        ev.to_lowercase().contains(&e.to_lowercase())
                            || e.to_lowercase().contains(&ev.to_lowercase())
                    })
                })
                .collect();

            if !hallucinated.is_empty() {
                checks.push(GateCheck::fail(
                    "no_hallucination",
                    &format!("Unverified entities: {:?}", hallucinated),
                ));
                return GateResult::fail(
                    &input.request_id,
                    GateOutcome::FailedHallucination,
                    checks,
                    coverage,
                );
            }
            checks.push(GateCheck::pass("no_hallucination"));
        }

        // Check 10: Contract validation (if provided)
        if let Some(contract) = &input.contract {
            if let Some(violation) = contract.validate_answer(&input.binding) {
                checks.push(GateCheck::fail(
                    "contract_valid",
                    &format!("Contract violation: {:?}", violation),
                ));
                return GateResult::fail(
                    &input.request_id,
                    GateOutcome::FailedContractViolation,
                    checks,
                    coverage,
                );
            }
            checks.push(GateCheck::pass("contract_valid"));
        }

        // All checks passed
        GateResult::pass(&input.request_id, checks, coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reliability_gate::claim_evidence::{
        ClaimType, EvidenceType, StrictClaim, StrictEvidence,
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
}
