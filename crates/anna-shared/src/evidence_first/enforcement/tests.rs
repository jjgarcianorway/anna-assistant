//! Tests for enforcement module.

use super::*;
use crate::evidence_first::citations::{Citation, CitationStore, EvidenceId};
use crate::evidence_first::sources::KnowledgeSource;

#[test]
fn test_claim_types() {
    let factual = Claim::factual("Boot time is 15 seconds");
    assert!(matches!(factual.claim_type, ClaimType::Factual));
    assert!(matches!(factual.required_evidence, EvidenceType::Probe));

    let doc = Claim::documentation("systemctl is used to manage services");
    assert!(matches!(doc.claim_type, ClaimType::Documentation));
}

#[test]
fn test_validation_result() {
    let valid = ValidationResult::valid();
    assert!(valid.is_valid());

    let invalid = ValidationResult::invalid("no evidence");
    assert!(!invalid.is_valid());

    let warning = ValidationResult::valid().with_warning("weak evidence");
    assert!(warning.is_valid());
    assert!(!warning.warnings.is_empty());
}

#[test]
fn test_supported_claim() {
    let claim = Claim::new("test", ClaimType::Uncertainty);
    let supported = SupportedClaim::new(claim, vec![]);

    // Uncertainty claims don't need evidence
    assert!(supported.is_valid());
}

#[test]
fn test_claim_validator() {
    let validator = ClaimValidator::default_strictness();
    let mut store = CitationStore::new();

    // Add some evidence
    let id = EvidenceId::probe("sys.boot.analyze");
    store.add_evidence(
        id.clone(),
        KnowledgeSource::ProbeOutput("sys.boot.analyze".to_string()),
        "Boot time: 15 seconds",
    );
    store.add_citation(Citation::new(
        id,
        "probe:sys.boot.analyze",
        "Boot time: 15 seconds",
    ));

    // Test validation
    let claim = Claim::factual("Boot time is 15 seconds");
    let result = validator.validate_claim(&claim, &store);
    assert!(result.is_valid());
}

#[test]
fn test_validation_report() {
    let report = ValidationReport {
        supported_claims: vec![],
        unsupported_claims: vec![],
        total_claims: 0,
        strictness: Strictness::Standard,
    };

    assert!(report.all_valid());
    assert!((report.validity_rate() - 100.0).abs() < 0.01);
}

#[test]
fn test_extract_claims() {
    let text = "Boot time is 15 seconds. According to the manual, systemctl manages services. You should restart the service.";
    let claims = extract_claims(text);

    assert!(claims.len() >= 2);

    // Check for factual claim
    assert!(claims
        .iter()
        .any(|c| matches!(c.claim_type, ClaimType::Factual)));

    // Check for doc claim
    assert!(claims
        .iter()
        .any(|c| matches!(c.claim_type, ClaimType::Documentation)));
}

#[test]
fn test_strictness_levels() {
    assert!(matches!(Strictness::Lenient, Strictness::Lenient));
    assert!(matches!(Strictness::Standard, Strictness::Standard));
    assert!(matches!(Strictness::Strict, Strictness::Strict));
}
