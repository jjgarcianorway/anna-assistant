//! Core envelope types - ModelRole, ModelResultEnvelope, EnvelopeValidation.

use serde::{Deserialize, Serialize};

use super::{Action, Claim, EvidenceRef, ModelError};

/// Role of the model that produced this response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelRole {
    /// Fast classifier/translator.
    Translator,
    /// Responsive general assistant.
    Junior,
    /// Deep reasoning specialist.
    Senior,
}

impl ModelRole {
    /// Get default timeout for this role in milliseconds.
    pub fn default_timeout_ms(&self) -> u64 {
        match self {
            Self::Translator => crate::anna_proto::DEFAULT_TRANSLATOR_TIMEOUT_MS,
            Self::Junior => crate::anna_proto::DEFAULT_JUNIOR_TIMEOUT_MS,
            Self::Senior => crate::anna_proto::DEFAULT_SENIOR_TIMEOUT_MS,
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Translator => "translator",
            Self::Junior => "junior",
            Self::Senior => "senior",
        }
    }
}

impl std::fmt::Display for ModelRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// The main response envelope from any model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelResultEnvelope {
    /// Whether the operation succeeded.
    pub ok: bool,
    /// Role of the model that produced this.
    pub role: ModelRole,
    /// Associated ticket ID.
    pub ticket_id: String,
    /// Confidence level (0.0 to 1.0).
    pub confidence: f64,
    /// Human-readable summary.
    pub summary: String,
    /// Claims made (each with evidence support).
    #[serde(default)]
    pub claims: Vec<Claim>,
    /// Proposed next actions.
    #[serde(default)]
    pub next_actions: Vec<Action>,
    /// Evidence used in this response.
    #[serde(default)]
    pub evidence_used: Vec<EvidenceRef>,
    /// Errors (only when ok=false).
    #[serde(default)]
    pub errors: Vec<ModelError>,
}

impl ModelResultEnvelope {
    /// Create a successful envelope.
    pub fn success(role: ModelRole, ticket_id: &str, summary: &str, confidence: f64) -> Self {
        Self {
            ok: true,
            role,
            ticket_id: ticket_id.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            summary: summary.to_string(),
            claims: Vec::new(),
            next_actions: Vec::new(),
            evidence_used: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Create a failed envelope.
    pub fn failure(role: ModelRole, ticket_id: &str, errors: Vec<ModelError>) -> Self {
        Self {
            ok: false,
            role,
            ticket_id: ticket_id.to_string(),
            confidence: 0.0,
            summary: String::new(),
            claims: Vec::new(),
            next_actions: Vec::new(),
            evidence_used: Vec::new(),
            errors,
        }
    }

    /// Add a claim.
    pub fn with_claim(mut self, claim: Claim) -> Self {
        self.claims.push(claim);
        self
    }

    /// Add an action.
    pub fn with_action(mut self, action: Action) -> Self {
        self.next_actions.push(action);
        self
    }

    /// Add evidence reference.
    pub fn with_evidence(mut self, evidence: EvidenceRef) -> Self {
        self.evidence_used.push(evidence);
        self
    }

    /// Check if response has claims.
    pub fn has_claims(&self) -> bool {
        !self.claims.is_empty()
    }

    /// Check if all claims have evidence support.
    pub fn all_claims_supported(&self) -> bool {
        self.claims.iter().all(|c| !c.supports.is_empty())
    }

    /// Validate envelope integrity.
    pub fn validate(&self) -> EnvelopeValidation {
        let mut issues = Vec::new();

        if self.ok && self.summary.is_empty() {
            issues.push("Success envelope has empty summary".to_string());
        }

        if !self.ok && self.errors.is_empty() {
            issues.push("Failure envelope has no errors".to_string());
        }

        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!("Confidence {} out of range [0,1]", self.confidence));
        }

        if issues.is_empty() {
            EnvelopeValidation::Valid
        } else {
            EnvelopeValidation::Invalid { issues }
        }
    }
}

/// Envelope validation result.
#[derive(Debug, Clone)]
pub enum EnvelopeValidation {
    /// Envelope is valid.
    Valid,
    /// Envelope has issues.
    Invalid { issues: Vec<String> },
}

impl EnvelopeValidation {
    /// Check if valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}
