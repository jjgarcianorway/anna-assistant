//! Validation reporting and strictness levels.

use super::validation::SupportedClaim;
use serde::{Deserialize, Serialize};

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
                lines.push(format!(
                    "  - {}: {}",
                    claim.claim.text, claim.validation.reason
                ));
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
