//! Citation Enforcement - No Citations, No Claims (v0.0.435).
//!
//! Every claim in Anna's response must be backed by evidence.
//! This module validates that claims have supporting citations.

use super::citations::{Citation, CitationStore, EvidenceId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A claim that needs to be validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim text.
    pub text: String,
    /// Type of claim.
    pub claim_type: ClaimType,
    /// Required evidence type.
    pub required_evidence: EvidenceType,
}

impl Claim {
    /// Create a new claim.
    pub fn new(text: &str, claim_type: ClaimType) -> Self {
        Self {
            text: text.to_string(),
            claim_type,
            required_evidence: claim_type.required_evidence(),
        }
    }

    /// Create a factual claim (needs probe evidence).
    pub fn factual(text: &str) -> Self {
        Self::new(text, ClaimType::Factual)
    }

    /// Create a documentation claim (needs doc evidence).
    pub fn documentation(text: &str) -> Self {
        Self::new(text, ClaimType::Documentation)
    }

    /// Create a recommendation claim (needs any evidence).
    pub fn recommendation(text: &str) -> Self {
        Self::new(text, ClaimType::Recommendation)
    }
}

/// Type of claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimType {
    /// Statement about current system state (needs probe evidence).
    Factual,
    /// Statement about how something works (needs doc evidence).
    Documentation,
    /// Suggestion or recommendation (needs some evidence).
    Recommendation,
    /// Acknowledgment of uncertainty (no evidence needed).
    Uncertainty,
}

impl ClaimType {
    /// Get required evidence type.
    pub fn required_evidence(&self) -> EvidenceType {
        match self {
            Self::Factual => EvidenceType::Probe,
            Self::Documentation => EvidenceType::Documentation,
            Self::Recommendation => EvidenceType::Any,
            Self::Uncertainty => EvidenceType::None,
        }
    }
}

/// Type of evidence required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceType {
    /// Must have probe output.
    Probe,
    /// Must have documentation (man, help, wiki).
    Documentation,
    /// Any evidence type.
    Any,
    /// No evidence required.
    None,
}

/// A claim that has been validated with evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedClaim {
    /// The original claim.
    pub claim: Claim,
    /// Supporting citations.
    pub citations: Vec<Citation>,
    /// Validation result.
    pub validation: ValidationResult,
}

impl SupportedClaim {
    /// Create a supported claim.
    pub fn new(claim: Claim, citations: Vec<Citation>) -> Self {
        let validation = validate_claim_support(&claim, &citations);
        Self {
            claim,
            citations,
            validation,
        }
    }

    /// Check if claim is valid.
    pub fn is_valid(&self) -> bool {
        self.validation.is_valid()
    }

    /// Format for display.
    pub fn format(&self) -> String {
        let status = if self.is_valid() { "✓" } else { "✗" };
        let citations_str = if self.citations.is_empty() {
            "no citations".to_string()
        } else {
            self.citations
                .iter()
                .map(|c| c.short_ref())
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!("{} {} ({})", status, self.claim.text, citations_str)
    }
}

/// Result of claim validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the claim is valid.
    pub valid: bool,
    /// Reason for validation result.
    pub reason: String,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a valid result.
    pub fn valid() -> Self {
        Self {
            valid: true,
            reason: "Claim supported by evidence".to_string(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid result.
    pub fn invalid(reason: &str) -> Self {
        Self {
            valid: false,
            reason: reason.to_string(),
            warnings: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn with_warning(mut self, warning: &str) -> Self {
        self.warnings.push(warning.to_string());
        self
    }

    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// Validate that a claim has appropriate support.
fn validate_claim_support(claim: &Claim, citations: &[Citation]) -> ValidationResult {
    match claim.required_evidence {
        EvidenceType::None => ValidationResult::valid(),

        EvidenceType::Probe => {
            let has_probe = citations
                .iter()
                .any(|c| c.evidence_id.0.starts_with("probe:"));

            if has_probe {
                ValidationResult::valid()
            } else if !citations.is_empty() {
                ValidationResult::valid()
                    .with_warning("Factual claim supported by docs, not probe")
            } else {
                ValidationResult::invalid("Factual claim requires probe evidence")
            }
        }

        EvidenceType::Documentation => {
            let has_doc = citations.iter().any(|c| {
                c.evidence_id.0.starts_with("man:")
                    || c.evidence_id.0.starts_with("help:")
                    || c.evidence_id.0.starts_with("wiki:")
                    || c.evidence_id.0.starts_with("doc:")
            });

            if has_doc {
                ValidationResult::valid()
            } else if !citations.is_empty() {
                ValidationResult::valid()
                    .with_warning("Documentation claim supported by probe, not docs")
            } else {
                ValidationResult::invalid("Documentation claim requires doc evidence")
            }
        }

        EvidenceType::Any => {
            if citations.is_empty() {
                ValidationResult::invalid("Recommendation requires some evidence")
            } else {
                ValidationResult::valid()
            }
        }
    }
}

/// Validates claims against a citation store.
pub struct ClaimValidator {
    /// Strictness level.
    strictness: Strictness,
}

impl ClaimValidator {
    /// Create a new validator.
    pub fn new(strictness: Strictness) -> Self {
        Self { strictness }
    }

    /// Create with default strictness.
    pub fn default_strictness() -> Self {
        Self::new(Strictness::Standard)
    }

    /// Validate a single claim.
    pub fn validate_claim(
        &self,
        claim: &Claim,
        store: &CitationStore,
    ) -> SupportedClaim {
        // Find citations relevant to this claim
        let relevant_citations = self.find_relevant_citations(claim, store);
        SupportedClaim::new(claim.clone(), relevant_citations)
    }

    /// Validate multiple claims.
    pub fn validate_claims(
        &self,
        claims: &[Claim],
        store: &CitationStore,
    ) -> ValidationReport {
        let mut supported_claims = Vec::new();
        let mut unsupported_claims = Vec::new();

        for claim in claims {
            let supported = self.validate_claim(claim, store);
            if supported.is_valid() {
                supported_claims.push(supported);
            } else {
                unsupported_claims.push(supported);
            }
        }

        ValidationReport {
            supported_claims,
            unsupported_claims,
            total_claims: claims.len(),
            strictness: self.strictness,
        }
    }

    /// Find citations relevant to a claim.
    fn find_relevant_citations(&self, claim: &Claim, store: &CitationStore) -> Vec<Citation> {
        let mut relevant = Vec::new();
        let claim_words: HashSet<String> = claim
            .text
            .to_lowercase()
            .split_whitespace()
            .filter(|w| w.len() > 3) // Skip short words
            .map(|s| s.to_string())
            .collect();

        for citation in store.all_citations() {
            // Check if citation excerpt overlaps with claim
            let excerpt_words: HashSet<String> = citation
                .excerpt
                .to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() > 3)
                .map(|s| s.to_string())
                .collect();

            let overlap: usize = claim_words.intersection(&excerpt_words).count();

            // Require at least some overlap or matching evidence type
            if overlap >= 1 || self.matches_evidence_type(claim, &citation.evidence_id) {
                relevant.push(citation.clone());
            }
        }

        relevant
    }

    /// Check if evidence type matches claim requirement.
    fn matches_evidence_type(&self, claim: &Claim, evidence_id: &EvidenceId) -> bool {
        match claim.required_evidence {
            EvidenceType::None => true,
            EvidenceType::Probe => evidence_id.0.starts_with("probe:"),
            EvidenceType::Documentation => {
                evidence_id.0.starts_with("man:")
                    || evidence_id.0.starts_with("help:")
                    || evidence_id.0.starts_with("wiki:")
                    || evidence_id.0.starts_with("doc:")
            }
            EvidenceType::Any => true,
        }
    }
}

impl Default for ClaimValidator {
    fn default() -> Self {
        Self::default_strictness()
    }
}

/// Strictness level for validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strictness {
    /// Allow claims with weak evidence.
    Lenient,
    /// Standard enforcement.
    Standard,
    /// Require exact evidence match.
    Strict,
}

/// Report from validating multiple claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Claims that passed validation.
    pub supported_claims: Vec<SupportedClaim>,
    /// Claims that failed validation.
    pub unsupported_claims: Vec<SupportedClaim>,
    /// Total claims checked.
    pub total_claims: usize,
    /// Strictness used.
    pub strictness: Strictness,
}

impl ValidationReport {
    /// Check if all claims are valid.
    pub fn all_valid(&self) -> bool {
        self.unsupported_claims.is_empty()
    }

    /// Get percentage of valid claims.
    pub fn validity_rate(&self) -> f64 {
        if self.total_claims == 0 {
            100.0
        } else {
            (self.supported_claims.len() as f64 / self.total_claims as f64) * 100.0
        }
    }

    /// Format report for display.
    pub fn format(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Validation Report ({:.0}% valid)",
            self.validity_rate()
        ));
        lines.push(format!(
            "Checked {} claims ({} supported, {} unsupported)",
            self.total_claims,
            self.supported_claims.len(),
            self.unsupported_claims.len()
        ));

        if !self.unsupported_claims.is_empty() {
            lines.push("\nUnsupported claims:".to_string());
            for claim in &self.unsupported_claims {
                lines.push(format!("  - {}: {}", claim.claim.text, claim.validation.reason));
            }
        }

        lines.join("\n")
    }

    /// Get warnings from all claims.
    pub fn all_warnings(&self) -> Vec<String> {
        self.supported_claims
            .iter()
            .chain(self.unsupported_claims.iter())
            .flat_map(|c| c.validation.warnings.clone())
            .collect()
    }
}

/// Extract claims from response text (simple heuristic).
pub fn extract_claims(text: &str) -> Vec<Claim> {
    let mut claims = Vec::new();

    // Patterns that indicate factual claims
    let factual_patterns = [
        "is running",
        "is not running",
        "is installed",
        "is not installed",
        "takes",
        "uses",
        "shows",
        "indicates",
        "currently",
        "GB",
        "MB",
        "seconds",
        "ms",
        "%",
    ];

    // Patterns that indicate documentation claims
    let doc_patterns = [
        "according to",
        "the manual",
        "documentation",
        "man page",
        "wiki",
        "means",
        "is used to",
        "allows",
    ];

    // Patterns that indicate uncertainty
    let uncertainty_patterns = [
        "might",
        "may",
        "could",
        "possibly",
        "I'm not sure",
        "unclear",
        "unknown",
    ];

    for sentence in text.split('.').filter(|s| !s.trim().is_empty()) {
        let sentence_lower = sentence.to_lowercase();

        // Check for uncertainty first
        if uncertainty_patterns
            .iter()
            .any(|p| sentence_lower.contains(p))
        {
            claims.push(Claim::new(sentence.trim(), ClaimType::Uncertainty));
            continue;
        }

        // Check for factual claims
        if factual_patterns.iter().any(|p| sentence_lower.contains(p)) {
            claims.push(Claim::factual(sentence.trim()));
            continue;
        }

        // Check for documentation claims
        if doc_patterns.iter().any(|p| sentence_lower.contains(p)) {
            claims.push(Claim::documentation(sentence.trim()));
            continue;
        }

        // Default to recommendation if contains action words
        if sentence_lower.contains("should")
            || sentence_lower.contains("recommend")
            || sentence_lower.contains("try")
        {
            claims.push(Claim::recommendation(sentence.trim()));
        }
    }

    claims
}

#[cfg(test)]
mod tests {
    use super::*;

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
            super::super::sources::KnowledgeSource::ProbeOutput("sys.boot.analyze".to_string()),
            "Boot time: 15 seconds",
        );
        store.add_citation(super::super::citations::Citation::new(
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
        assert!(claims.iter().any(|c| matches!(c.claim_type, ClaimType::Factual)));

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
}
