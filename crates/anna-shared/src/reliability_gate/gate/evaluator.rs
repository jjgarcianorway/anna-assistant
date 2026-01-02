//! The reliability gate evaluator.

use super::check::GateCheck;
use super::input::GateInput;
use super::outcome::GateOutcome;
use super::result::GateResult;

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
