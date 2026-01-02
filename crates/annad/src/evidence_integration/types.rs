//! Types for evidence integration (v0.0.410).

use anna_shared::evidence_engine::{EvidenceBundle, EvidenceDomain, EvidenceIntent};

/// Result of evidence integration
pub struct EvidenceIntegrationResult {
    /// The evidence bundle for specialist
    pub bundle: EvidenceBundle,
    /// If instant answer was found, bypass LLM
    pub instant_answer: Option<String>,
    /// Tags extracted from translator
    pub tags: Vec<String>,
    /// Evidence domain
    pub domain: EvidenceDomain,
    /// Evidence intent
    pub intent: EvidenceIntent,
}
