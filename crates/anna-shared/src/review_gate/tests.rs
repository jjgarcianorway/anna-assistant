//! Review gate tests (v0.0.228).

#[cfg(test)]
mod tests {
    use crate::review::{ReviewDecision, ReviewIssueKind};
    use crate::review_gate::{deterministic_review_gate, GateOutcome, ReviewContext};
    use crate::trace::FallbackUsed;

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
        let ctx = ReviewContext::new(75).with_grounding(0.8, 2).with_fallback(
            FallbackUsed::Deterministic {
                route_class: "MemoryUsage".to_string(),
            },
        );

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
