//! Claim extraction and validation tests.
//!
//! Tests for claim extraction from text and validation reporting.

#[cfg(test)]
mod tests {
    use crate::evidence_first::{
        citations::{Citation, CitationStore, EvidenceId},
        enforcement::{extract_claims, Claim, ClaimValidator},
        sources::KnowledgeSource,
    };

    /// Test 7: Claim extraction from text.
    #[test]
    fn test_claim_extraction() {
        let response_text = "Boot time is 16 seconds. According to the manual, systemd-analyze shows boot timing. You should disable NetworkManager-wait-online. I'm not sure about the exact cause.";

        let claims = extract_claims(response_text);

        // Should extract multiple claims
        assert!(!claims.is_empty(), "Should extract claims");

        // Should have different types
        use crate::evidence_first::enforcement::ClaimType;
        let has_factual = claims
            .iter()
            .any(|c| matches!(c.claim_type, ClaimType::Factual));
        let has_doc = claims
            .iter()
            .any(|c| matches!(c.claim_type, ClaimType::Documentation));
        let has_uncertainty = claims
            .iter()
            .any(|c| matches!(c.claim_type, ClaimType::Uncertainty));

        assert!(has_factual || has_doc, "Should have factual or doc claims");
    }

    /// Test 13: Validation report formatting.
    #[test]
    fn test_validation_report() {
        let mut store = CitationStore::new();
        let id = EvidenceId::probe("test");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ProbeOutput("test".to_string()),
            "test output",
        );
        store.add_citation(Citation::new(id, "probe:test", "test output"));

        let validator = ClaimValidator::default();
        let claims = vec![Claim::factual("test output shows results")];

        let report = validator.validate_claims(&claims, &store);

        // Format should include key info
        let formatted = report.format();
        assert!(
            formatted.contains("Validation Report"),
            "Should have header"
        );
    }
}
