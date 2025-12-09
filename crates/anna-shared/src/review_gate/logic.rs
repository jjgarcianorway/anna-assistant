//! Review gate logic (v0.0.228).

use crate::review::ReviewIssueKind;
use crate::trace::FallbackUsed;

use super::types::{GateOutcome, GateThresholds, ReviewContext};

/// Pure deterministic review gate.
/// NO I/O - just logic on signals.
pub fn deterministic_review_gate(ctx: &ReviewContext) -> GateOutcome {
    deterministic_review_gate_with_thresholds(ctx, &GateThresholds::default())
}

/// Deterministic review gate with custom thresholds
pub fn deterministic_review_gate_with_thresholds(
    ctx: &ReviewContext,
    thresholds: &GateThresholds,
) -> GateOutcome {
    // Rule 1: Invention detected → hard fail → EscalateToSenior
    // Contradictions and unverifiable specifics in evidence_required context
    if ctx.invention_detected {
        return GateOutcome::escalate(vec![ReviewIssueKind::Contradiction]);
    }

    // Rule 2: Explicit contradictions → Escalate
    if ctx.contradictions > 0 {
        return GateOutcome::escalate(vec![ReviewIssueKind::Contradiction]);
    }

    // Rule 3: No claims → Revise (check first since it's more specific)
    if ctx.total_claims == 0 && ctx.evidence_required {
        return GateOutcome::revise(vec![ReviewIssueKind::TooVague]);
    }

    // Rule 4: Low grounding → Revise
    if ctx.grounding_ratio < thresholds.min_grounding && ctx.evidence_required {
        return GateOutcome::revise(vec![ReviewIssueKind::MissingEvidence]);
    }

    // Rule 4: High score, no contradictions → Accept
    if ctx.reliability_score >= thresholds.accept_score && ctx.contradictions == 0 {
        return GateOutcome::accept();
    }

    // Rule 5: Deterministic fallback with decent score → Accept with tag
    if let Some(FallbackUsed::Deterministic { route_class: _ }) = &ctx.fallback_used {
        if ctx.reliability_score >= thresholds.fallback_accept_score {
            return GateOutcome::accept_with_fallback();
        }
    }

    // Rule 6: Timeout fallback with decent score → Accept with tag
    if let Some(FallbackUsed::Timeout { .. }) = &ctx.fallback_used {
        if ctx.reliability_score >= thresholds.fallback_accept_score {
            return GateOutcome::accept_with_fallback();
        }
    }

    // Rule 7: Budget exceeded but has result → Accept with lower confidence
    if ctx.budget_exceeded && ctx.reliability_score >= 60 {
        return GateOutcome::accept_with_fallback();
    }

    // Rule 8: Medium score range → unclear, needs LLM review
    if ctx.reliability_score >= 50 && ctx.reliability_score < thresholds.accept_score {
        return GateOutcome::unclear();
    }

    // Rule 9: Very low score → Revise (deterministic fix attempt first)
    if ctx.reliability_score < 50 {
        let mut reasons = Vec::new();
        if ctx.grounding_ratio < thresholds.min_grounding {
            reasons.push(ReviewIssueKind::MissingEvidence);
        }
        if ctx.unverifiable_specifics > 0 {
            reasons.push(ReviewIssueKind::UnverifiableSpecifics);
        }
        if reasons.is_empty() {
            reasons.push(ReviewIssueKind::TooVague);
        }
        return GateOutcome::revise(reasons);
    }

    // Default: unclear
    GateOutcome::unclear()
}
