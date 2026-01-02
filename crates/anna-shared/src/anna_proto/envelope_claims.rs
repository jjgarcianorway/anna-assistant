//! Claim and evidence types - Claim, EvidenceRef, EvidenceKind.

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

/// Reference to evidence used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// Unique evidence ID.
    pub id: String,
    /// Kind of evidence.
    pub kind: EvidenceKind,
    /// Human-readable title.
    pub title: String,
}

impl EvidenceRef {
    /// Create a new evidence reference.
    pub fn new(id: &str, kind: EvidenceKind, title: &str) -> Self {
        Self {
            id: id.to_string(),
            kind,
            title: title.to_string(),
        }
    }

    /// Create a probe evidence reference.
    pub fn probe(id: &str, title: &str) -> Self {
        Self::new(id, EvidenceKind::Probe, title)
    }

    /// Create a man page evidence reference.
    pub fn man(id: &str, title: &str) -> Self {
        Self::new(id, EvidenceKind::Man, title)
    }
}

/// Kind of evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceKind {
    /// Probe output.
    Probe,
    /// Man page.
    Man,
    /// --help output.
    Help,
    /// Wiki page.
    Wiki,
}
