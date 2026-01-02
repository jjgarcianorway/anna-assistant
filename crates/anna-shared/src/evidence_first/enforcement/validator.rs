//! Claim validator implementation.

use super::claim::{Claim, EvidenceType};
use super::report::{Strictness, ValidationReport};
use super::validation::SupportedClaim;
use crate::evidence_first::citations::{CitationStore, EvidenceId};
use crate::evidence_first::Citation;
use std::collections::HashSet;

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
    pub fn validate_claim(&self, claim: &Claim, store: &CitationStore) -> SupportedClaim {
        // Find citations relevant to this claim
        let relevant_citations = self.find_relevant_citations(claim, store);
        SupportedClaim::new(claim.clone(), relevant_citations)
    }

    /// Validate multiple claims.
    pub fn validate_claims(&self, claims: &[Claim], store: &CitationStore) -> ValidationReport {
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
