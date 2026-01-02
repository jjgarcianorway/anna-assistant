//! Claim validation logic and results.

use super::claim::{Claim, EvidenceType};
use crate::evidence_first::citations::Citation;
use serde::{Deserialize, Serialize};

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
pub(super) fn validate_claim_support(claim: &Claim, citations: &[Citation]) -> ValidationResult {
    match claim.required_evidence {
        EvidenceType::None => ValidationResult::valid(),

        EvidenceType::Probe => {
            let has_probe = citations
                .iter()
                .any(|c| c.evidence_id.0.starts_with("probe:"));

            if has_probe {
                ValidationResult::valid()
            } else if !citations.is_empty() {
                ValidationResult::valid().with_warning("Factual claim supported by docs, not probe")
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
