//! Evidence Binding (Part C) - v0.0.437.
//!
//! Every claim in the final answer must map to:
//! - One or more EvidenceRef IDs
//! - And must satisfy the intent's subject and allowed_fields
//!
//! If evidence exists but does not map cleanly:
//! - Anna must say "I collected data but cannot safely answer exactly what you asked."
//! - Then propose a clarification or alternative phrasing
//!
//! This prevents hallucinated glue text.

// Re-export all public types from sibling modules
pub use super::evidence_bind_engine::EvidenceBinding;
pub use super::evidence_bind_types::{
    BindingResult, BindingViolation, BoundClaim, EvidenceItem, UnboundClaim,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::question_contract::intent::{IntentBuilder, IntentCategory, Subject};

    #[test]
    fn test_valid_binding() {
        let intent = IntentBuilder::new("int_001")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .allow_fields(vec!["free"])
            .build();

        let claims = vec![UnboundClaim::new("4.2 GB free", "free")];

        let evidence = vec![EvidenceItem::new(
            "ev_mem",
            Subject::Memory,
            vec!["free", "total"],
            "Memory usage",
        )];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        assert!(result.is_valid());
        assert_eq!(result.valid_claims().len(), 1);
    }

    #[test]
    fn test_mismatched_evidence() {
        let intent = IntentBuilder::new("int_002")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let claims = vec![UnboundClaim::new("Intel Core i7", "cpu_model")];

        // Evidence is about CPU, not memory
        let evidence = vec![EvidenceItem::new(
            "ev_cpu",
            Subject::Cpu,
            vec!["model"],
            "CPU info",
        )];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        match result {
            BindingResult::MismatchedEvidence { message, .. } => {
                assert!(message.contains("cannot safely answer"));
            }
            _ => panic!("Expected MismatchedEvidence"),
        }
    }

    #[test]
    fn test_no_evidence() {
        let intent = IntentBuilder::new("int_003")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .build();

        let claims = vec![UnboundClaim::new("4.2 GB free", "free")];

        let result = EvidenceBinding::bind(&intent, claims, &[]);

        assert!(matches!(result, BindingResult::NoEvidence));
    }

    #[test]
    fn test_partial_binding() {
        let intent = IntentBuilder::new("int_004")
            .category(IntentCategory::Fact)
            .subject(Subject::Memory)
            .allow_fields(vec!["free", "total"])
            .build();

        let claims = vec![
            UnboundClaim::new("4.2 GB free", "free"),
            UnboundClaim::new("Some unverified claim", "unknown"),
        ];

        let evidence = vec![EvidenceItem::new(
            "ev_mem",
            Subject::Memory,
            vec!["free"],
            "Memory free",
        )];

        let result = EvidenceBinding::bind(&intent, claims, &evidence);

        match result {
            BindingResult::PartialBind {
                valid_claims,
                unbound_claims,
            } => {
                assert_eq!(valid_claims.len(), 1);
                assert_eq!(unbound_claims.len(), 1);
            }
            _ => panic!("Expected PartialBind"),
        }
    }

    #[test]
    fn test_binding_validation() {
        let claims = vec![
            BoundClaim::new("Valid claim", "field", vec!["ev_1".to_string()]),
            BoundClaim::new("No evidence claim", "field", vec![]),
        ];

        let violations = EvidenceBinding::validate_binding(&claims);

        assert_eq!(violations.len(), 1);
        assert!(matches!(violations[0], BindingViolation::NoEvidence { .. }));
    }

    #[test]
    fn test_fallback_message() {
        let result = BindingResult::MismatchedEvidence {
            message: "Cannot answer".to_string(),
            suggestion: "Try rephrasing".to_string(),
        };

        let msg = result.fallback_message().unwrap();
        assert!(msg.contains("Cannot answer"));
        assert!(msg.contains("Try rephrasing"));
    }
}
