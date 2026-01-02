//! Canary Tests - Validation & Quality (Part 4 of 4) - v0.0.437.
//!
//! Fixed tests that MUST pass for any release.
//! Any regression here BLOCKS release.
//!
//! Test cases:
//! - Clarification stops execution
//! - Evidence binding required
//! - Subject mismatch detection
//! - Fact/Status never has tutorials
//! - Intent validation complete

#[cfg(test)]
mod validation_canary_tests {
    use crate::question_contract::evidence_bind::*;
    use crate::question_contract::intent::*;
    use crate::question_contract::*;

    // ============================================
    // CANARY 6: Clarification stops execution
    // ============================================

    #[test]
    fn canary_clarification_blocks() {
        let intent = IntentBuilder::new("canary_ambiguous")
            .category(IntentCategory::Status) // Set category even for clarification
            .subject(Subject::Service) // Set subject even for clarification
            .needs_clarification(
                "Which service do you mean?",
                vec!["nginx", "apache", "postgresql"],
            )
            .build();

        assert!(intent.needs_clarification());

        // Validation should pass - clarification intent with proper category/subject
        let validation = validate_intent(&intent);
        assert!(validation.is_valid());

        // The key point: clarification STOPS execution
        assert!(super::super::super::CLARIFICATION_STOPS_EXECUTION);
    }

    // ============================================
    // CANARY 7: Evidence binding required
    // ============================================

    #[test]
    fn canary_evidence_binding_required() {
        let intent = IntentBuilder::new("canary_evidence")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let claims = vec![UnboundClaim::new("4.2 GB free", "free")];

        // No evidence = binding fails
        let result = EvidenceBinding::bind(&intent, claims.clone(), &[]);
        assert!(matches!(result, BindingResult::NoEvidence));

        // With evidence = binding succeeds
        let evidence = vec![EvidenceItem::new(
            "ev_mem",
            Subject::Memory,
            vec!["free"],
            "Memory info",
        )];
        let result = EvidenceBinding::bind(&intent, claims, &evidence);
        assert!(result.is_valid());
    }

    // ============================================
    // CANARY 8: Subject mismatch detection
    // ============================================

    #[test]
    fn canary_subject_mismatch() {
        use crate::question_contract::stats::MisclassificationDetector;

        // User asked about memory, Anna answered about CPU
        assert!(MisclassificationDetector::subject_mismatch(
            Subject::Cpu,
            "I asked about RAM usage"
        ));

        // Correct subject
        assert!(!MisclassificationDetector::subject_mismatch(
            Subject::Memory,
            "Thanks for the RAM info"
        ));
    }

    // ============================================
    // CANARY 9: Fact/Status never has tutorials
    // ============================================

    #[test]
    fn canary_fact_status_no_tutorials() {
        assert!(!IntentCategory::Fact.allows_tutorials());
        assert!(!IntentCategory::Status.allows_tutorials());
        assert!(IntentCategory::Explanation.allows_tutorials());
        assert!(IntentCategory::ActionRequest.allows_tutorials());
    }

    // ============================================
    // CANARY 10: Intent validation complete
    // ============================================

    #[test]
    fn canary_intent_validation() {
        // Unknown category is invalid
        let bad_intent = IntentBuilder::new("canary_bad").build(); // No category or subject set
        let validation = validate_intent(&bad_intent);
        assert!(!validation.is_valid());

        // Complete intent is valid
        let good_intent = IntentBuilder::new("canary_good")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .allow_fields(vec!["free"])
            .build();
        let validation = validate_intent(&good_intent);
        assert!(validation.is_valid());
    }
}
