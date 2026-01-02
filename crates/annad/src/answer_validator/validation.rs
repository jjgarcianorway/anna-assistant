//! Core validation logic for checking answers against evidence.

use anna_shared::claims::extract_claims;
use anna_shared::grounding::{compute_grounding, ParsedEvidence};
use anna_shared::guard::{run_guard, VerifyResult};
use anna_shared::reliability::{compute_reliability, ReliabilityInput};
use tracing::debug;

use super::types::ValidationIssue;

/// Validate an answer and return score + issues
pub fn validate_answer(
    answer: &str,
    evidence: &ParsedEvidence,
    reliability_input: &ReliabilityInput,
) -> (u8, Vec<ValidationIssue>) {
    let mut issues = Vec::new();

    // Extract claims from answer
    let claims = extract_claims(answer);
    debug!("Extracted {} claims from answer", claims.len());

    // Compute grounding against evidence
    let grounding = compute_grounding(&claims, evidence);
    if grounding.verified_claims < grounding.total_claims {
        let ungrounded = (grounding.total_claims - grounding.verified_claims) as usize;
        if ungrounded > 0 {
            issues.push(ValidationIssue::UngroundedClaims { count: ungrounded });
        }
    }

    // Run invention guard
    let guard = run_guard(&claims, evidence, reliability_input.evidence_required);
    if guard.invention_detected {
        // Extract unverifiable claims from details
        for item in &guard.details {
            if matches!(
                item.result,
                VerifyResult::Unverifiable | VerifyResult::Contradiction { .. }
            ) {
                issues.push(ValidationIssue::InventionDetected {
                    claim: format!("{:?}", item.claim),
                });
            }
        }
    }

    // Check for missing evidence
    // Note: evidence_kinds are EvidenceKind, not String
    // Skip this check if no evidence kinds specified
    // TODO: Convert reliability_input.evidence_kinds to EvidenceKind

    // Check confidence
    if reliability_input.translator_confidence < 0.7 {
        issues.push(ValidationIssue::LowConfidence {
            confidence: reliability_input.translator_confidence,
        });
    }

    // Compute final score
    let output = compute_reliability(reliability_input);

    (output.score, issues)
}
