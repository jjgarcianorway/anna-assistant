//! Model Result Envelope (Part D) - v0.0.436.
//!
//! Strong typing for all model responses.
//! Every specialist must output exactly this envelope.

use serde::{Deserialize, Serialize};

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
            Self::Translator => super::DEFAULT_TRANSLATOR_TIMEOUT_MS,
            Self::Junior => super::DEFAULT_JUNIOR_TIMEOUT_MS,
            Self::Senior => super::DEFAULT_SENIOR_TIMEOUT_MS,
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

/// A proposed action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Type of action.
    #[serde(rename = "type")]
    pub action_type: ActionType,
    /// Action-specific payload.
    pub payload: ActionPayload,
    /// Risk level.
    #[serde(default)]
    pub risk: RiskLevel,
    /// Whether user confirmation is required.
    #[serde(default)]
    pub requires_confirmation: bool,
}

impl Action {
    /// Create a probe action.
    pub fn probe(probe_id: &str) -> Self {
        Self {
            action_type: ActionType::Probe,
            payload: ActionPayload::Probe {
                probe_id: probe_id.to_string(),
            },
            risk: RiskLevel::Safe,
            requires_confirmation: false,
        }
    }

    /// Create an ask_user action.
    pub fn ask_user(question: &str) -> Self {
        Self {
            action_type: ActionType::AskUser,
            payload: ActionPayload::AskUser {
                question: question.to_string(),
            },
            risk: RiskLevel::Safe,
            requires_confirmation: false,
        }
    }

    /// Create a propose_change action.
    pub fn propose_change(description: &str, command: &str) -> Self {
        Self {
            action_type: ActionType::ProposeChange,
            payload: ActionPayload::ProposeChange {
                description: description.to_string(),
                command: command.to_string(),
            },
            risk: RiskLevel::Risky,
            requires_confirmation: true,
        }
    }

    /// Create an install_helper action.
    pub fn install_helper(helper: &str) -> Self {
        Self {
            action_type: ActionType::InstallHelper,
            payload: ActionPayload::InstallHelper {
                helper_name: helper.to_string(),
            },
            risk: RiskLevel::Risky,
            requires_confirmation: true,
        }
    }
}

/// Type of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Run a probe.
    Probe,
    /// Ask the user a question.
    AskUser,
    /// Propose a system change.
    ProposeChange,
    /// Install a helper tool.
    InstallHelper,
}

/// Action-specific payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionPayload {
    /// Probe action payload.
    Probe { probe_id: String },
    /// Ask user payload.
    AskUser { question: String },
    /// Propose change payload.
    ProposeChange { description: String, command: String },
    /// Install helper payload.
    InstallHelper { helper_name: String },
    /// Generic/unknown payload.
    Other(serde_json::Value),
}

/// Risk level for an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Safe action (read-only, no side effects).
    #[default]
    Safe,
    /// Risky action (may modify system).
    Risky,
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

/// An error from the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelError {
    /// Error code.
    pub code: ErrorCode,
    /// Human-readable message.
    pub message: String,
}

impl ModelError {
    /// Create a new error.
    pub fn new(code: ErrorCode, message: &str) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }

    /// Create insufficient evidence error.
    pub fn insufficient_evidence(message: &str) -> Self {
        Self::new(ErrorCode::InsufficientEvidence, message)
    }

    /// Create need clarification error.
    pub fn need_clarification(message: &str) -> Self {
        Self::new(ErrorCode::NeedClarification, message)
    }
}

/// Error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// Not enough evidence to answer.
    InsufficientEvidence,
    /// Need clarification from user.
    NeedClarification,
    /// Model cannot handle this type of request.
    UnsupportedRequest,
    /// Internal model error.
    InternalError,
    /// Context too long.
    ContextTooLong,
    /// Unknown error.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_role_timeout() {
        assert!(ModelRole::Translator.default_timeout_ms() < ModelRole::Junior.default_timeout_ms());
        assert!(ModelRole::Junior.default_timeout_ms() < ModelRole::Senior.default_timeout_ms());
    }

    #[test]
    fn test_envelope_success() {
        let envelope = ModelResultEnvelope::success(
            ModelRole::Junior,
            "DSK-001",
            "Boot time is 15s",
            0.85,
        );

        assert!(envelope.ok);
        assert_eq!(envelope.role, ModelRole::Junior);
        assert!(envelope.validate().is_valid());
    }

    #[test]
    fn test_envelope_failure() {
        let envelope = ModelResultEnvelope::failure(
            ModelRole::Senior,
            "DSK-002",
            vec![ModelError::insufficient_evidence("No boot data")],
        );

        assert!(!envelope.ok);
        assert!(!envelope.errors.is_empty());
        assert!(envelope.validate().is_valid());
    }

    #[test]
    fn test_envelope_validation() {
        // Success with empty summary is invalid
        let mut envelope = ModelResultEnvelope::success(ModelRole::Junior, "DSK-001", "", 0.5);
        assert!(!envelope.validate().is_valid());

        // Failure with no errors is invalid
        envelope = ModelResultEnvelope::failure(ModelRole::Junior, "DSK-001", vec![]);
        assert!(!envelope.validate().is_valid());
    }

    #[test]
    fn test_claim_support() {
        let unsupported = Claim::new("Some claim");
        assert!(!unsupported.is_supported());

        let supported = Claim::with_support("Some claim", vec!["ev_1".to_string()]);
        assert!(supported.is_supported());
    }

    #[test]
    fn test_action_types() {
        let probe = Action::probe("sys.boot.analyze");
        assert_eq!(probe.action_type, ActionType::Probe);
        assert_eq!(probe.risk, RiskLevel::Safe);

        let change = Action::propose_change("Restart service", "systemctl restart nginx");
        assert_eq!(change.action_type, ActionType::ProposeChange);
        assert_eq!(change.risk, RiskLevel::Risky);
        assert!(change.requires_confirmation);
    }

    #[test]
    fn test_envelope_serialization() {
        let envelope = ModelResultEnvelope::success(ModelRole::Junior, "DSK-001", "Test", 0.9)
            .with_claim(Claim::with_support("Boot is slow", vec!["ev_boot".to_string()]))
            .with_evidence(EvidenceRef::probe("ev_boot", "Boot analysis"));

        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("junior"));
        assert!(json.contains("Boot is slow"));

        let parsed: ModelResultEnvelope = serde_json::from_str(&json).unwrap();
        assert!(parsed.ok);
        assert_eq!(parsed.claims.len(), 1);
    }
}
