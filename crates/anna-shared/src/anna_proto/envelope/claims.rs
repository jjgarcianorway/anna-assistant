//! Claim types for model responses.

use serde::{Deserialize, Serialize};

/// A claim made by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    /// The claim text.
    pub text: String,
    /// Evidence IDs that support this claim.
    #[serde(default)]
    pub supports: Vec<String>,
}

impl Claim {
    /// Create a new claim.
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            supports: Vec::new(),
        }
    }

    /// Create a claim with evidence support.
    pub fn with_support(text: &str, evidence_ids: Vec<String>) -> Self {
        Self {
            text: text.to_string(),
            supports: evidence_ids,
        }
    }

    /// Check if claim has evidence support.
    pub fn is_supported(&self) -> bool {
        !self.supports.is_empty()
    }
}
