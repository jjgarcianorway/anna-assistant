//! Review gate: deterministic-first hybrid review logic.
//!
//! Pure function that decides Accept/Revise/Escalate/Clarify based on
//! existing deterministic signals (reliability, grounding, guard).
//!
//! LLM review is only invoked when the gate returns "unclear".

use crate::review::{ReviewDecision, ReviewIssueKind};
use crate::trace::{FallbackUsed, SpecialistOutcome};
use serde::{Deserialize, Serialize};

/// Context for review gate decision (all deterministic signals)
#[derive(Debug, Clone, Default)]
pub struct ReviewContext {
    /// Reliability score from compute_reliability (0-100)
    pub reliability_score: u8,
    /// Grounding ratio from ANCHOR (0.0-1.0)
    pub grounding_ratio: f32,
    /// Total claims extracted
    pub total_claims: u32,
    /// Whether invention was detected (GUARD)
    pub invention_detected: bool,
    /// Number of contradictions found
    pub contradictions: u32,
    /// Number of unverifiable specifics
    pub unverifiable_specifics: u32,
    /// Whether evidence was required for this query type
    pub evidence_required: bool,
    /// Whether stage budget was exceeded
    pub budget_exceeded: bool,
    /// Specialist outcome from trace
    pub specialist_outcome: Option<SpecialistOutcome>,
    /// Fallback used (deterministic, timeout, etc.)
    pub fallback_used: Option<FallbackUsed>,
    /// Whether transcript was capped
    pub transcript_capped: bool,
    /// Whether prompt was truncated
    pub prompt_truncated: bool,
}

impl ReviewContext {
    /// Create new context with score
    pub fn new(reliability_score: u8) -> Self {
        Self { reliability_score, ..Default::default() }
    }

    /// Set grounding info
    pub fn with_grounding(mut self, ratio: f32, claims: u32) -> Self {
        self.grounding_ratio = ratio;
        self.total_claims = claims;
        self
    }

    /// Set guard info
    pub fn with_guard(mut self, invention: bool, contradictions: u32, unverifiable: u32) -> Self {
        self.invention_detected = invention;
        self.contradictions = contradictions;
        self.unverifiable_specifics = unverifiable;
        self
    }

    /// Set evidence_required
    pub fn with_evidence_required(mut self, required: bool) -> Self {
        self.evidence_required = required;
        self
    }

    /// Set fallback info
    pub fn with_fallback(mut self, fallback: FallbackUsed) -> Self {
        self.fallback_used = Some(fallback);
        self
    }

    /// Set budget exceeded
    pub fn with_budget_exceeded(mut self, exceeded: bool) -> Self {
        self.budget_exceeded = exceeded;
        self
    }
}

/// Outcome of deterministic gate (before any LLM review)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateOutcome {
    /// Decision from the gate
    pub decision: ReviewDecision,
    /// Reasons for the decision
    pub reasons: Vec<ReviewIssueKind>,
    /// Whether LLM review is required
    pub requires_llm_review: bool,
    /// Confidence in the decision (0.0-1.0)
    pub confidence: f32,
}

impl GateOutcome {
    /// Create an Accept outcome
    pub fn accept() -> Self {
        Self {
            decision: ReviewDecision::Accept,
            reasons: Vec::new(),
            requires_llm_review: false,
            confidence: 1.0,
        }
    }

    /// Create an Accept outcome with fallback tag
    pub fn accept_with_fallback() -> Self {
        Self {
            decision: ReviewDecision::Accept,
            reasons: Vec::new(),
            requires_llm_review: false,
            confidence: 0.85, // Lower confidence due to fallback
        }
    }

    /// Create a Revise outcome
    pub fn revise(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::Revise,
            reasons,
            requires_llm_review: false,
            confidence: 0.9,
        }
    }

    /// Create an Escalate outcome
    pub fn escalate(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::EscalateToSenior,
            reasons,
            requires_llm_review: false,
            confidence: 0.95,
        }
    }

    /// Create an Unclear outcome (requires LLM review)
    pub fn unclear() -> Self {
        Self {
            decision: ReviewDecision::Revise,
            reasons: Vec::new(),
            requires_llm_review: true,
            confidence: 0.5,
        }
    }

    /// Create a ClarifyUser outcome
    pub fn clarify(reasons: Vec<ReviewIssueKind>) -> Self {
        Self {
            decision: ReviewDecision::ClarifyUser,
            reasons,
            requires_llm_review: false,
            confidence: 0.9,
        }
    }
}

/// Thresholds for gate decisions (configurable)
#[derive(Debug, Clone)]
pub struct GateThresholds {
    /// Minimum score for accept (default: 80)
    pub accept_score: u8,
    /// Minimum grounding ratio (default: 0.5)
    pub min_grounding: f32,
    /// Score for accept with fallback (default: 70)
    pub fallback_accept_score: u8,
}

impl Default for GateThresholds {
    fn default() -> Self {
        Self {
            accept_score: 80,
            min_grounding: 0.5,
            fallback_accept_score: 70,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_accept_high_score_no_contradiction() {
        let ctx = ReviewContext::new(85)
            .with_grounding(0.9, 3)
            .with_guard(false, 0, 0);

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Accept);
        assert!(!outcome.requires_llm_review);
        assert_eq!(outcome.confidence, 1.0);
    }

    #[test]
    fn test_gate_escalate_on_invention() {
        let ctx = ReviewContext::new(90)
            .with_grounding(0.8, 2)
            .with_guard(true, 0, 0); // invention_detected

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::EscalateToSenior);
        assert!(outcome.reasons.contains(&ReviewIssueKind::Contradiction));
    }

    #[test]
    fn test_gate_escalate_on_contradiction() {
        let ctx = ReviewContext::new(85)
            .with_grounding(0.8, 2)
            .with_guard(false, 1, 0); // 1 contradiction

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::EscalateToSenior);
    }

    #[test]
    fn test_gate_revise_on_no_claims() {
        let ctx = ReviewContext::new(75)
            .with_grounding(0.0, 0) // no claims
            .with_evidence_required(true);

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Revise);
        assert!(outcome.reasons.contains(&ReviewIssueKind::TooVague));
    }

    #[test]
    fn test_gate_revise_on_low_grounding() {
        let ctx = ReviewContext::new(75)
            .with_grounding(0.3, 5) // low grounding
            .with_evidence_required(true);

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Revise);
        assert!(outcome.reasons.contains(&ReviewIssueKind::MissingEvidence));
    }

    #[test]
    fn test_gate_accept_deterministic_fallback() {
        let ctx = ReviewContext::new(75)
            .with_grounding(0.8, 2)
            .with_fallback(FallbackUsed::Deterministic {
                route_class: "MemoryUsage".to_string(),
            });

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Accept);
        assert_eq!(outcome.confidence, 0.85); // Lower confidence
    }

    #[test]
    fn test_gate_routes_to_llm_review_when_unclear() {
        let ctx = ReviewContext::new(65) // Medium score
            .with_grounding(0.6, 2);

        let outcome = deterministic_review_gate(&ctx);

        assert!(outcome.requires_llm_review);
    }

    #[test]
    fn test_deterministic_gate_is_stable_for_same_inputs() {
        let ctx = ReviewContext::new(85)
            .with_grounding(0.9, 3)
            .with_guard(false, 0, 0);

        let outcome1 = deterministic_review_gate(&ctx);
        let outcome2 = deterministic_review_gate(&ctx);

        assert_eq!(outcome1.decision, outcome2.decision);
        assert_eq!(outcome1.confidence, outcome2.confidence);
        assert_eq!(outcome1.requires_llm_review, outcome2.requires_llm_review);
    }

    #[test]
    fn test_gate_budget_exceeded_accepts_with_low_confidence() {
        let ctx = ReviewContext::new(65)
            .with_grounding(0.7, 2)
            .with_budget_exceeded(true);

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Accept);
        assert_eq!(outcome.confidence, 0.85);
    }

    #[test]
    fn test_gate_very_low_score_revises() {
        let ctx = ReviewContext::new(30)
            .with_grounding(0.2, 1)
            .with_guard(false, 0, 2); // 2 unverifiable

        let outcome = deterministic_review_gate(&ctx);

        assert_eq!(outcome.decision, ReviewDecision::Revise);
        assert!(!outcome.requires_llm_review);
    }
}
