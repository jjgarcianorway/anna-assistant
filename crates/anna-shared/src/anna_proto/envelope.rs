//! Model Result Envelope (Part D) - v0.0.436.
//!
//! Strong typing for all model responses.
//! Every specialist must output exactly this envelope.

// Re-export all types from sibling modules to maintain public API
pub use super::envelope_actions::{Action, ActionPayload, ActionType, RiskLevel};
pub use super::envelope_claims::{Claim, EvidenceKind, EvidenceRef};
pub use super::envelope_errors::{ErrorCode, ModelError};
pub use super::envelope_types::{EnvelopeValidation, ModelResultEnvelope, ModelRole};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_role_timeout() {
        assert!(
            ModelRole::Translator.default_timeout_ms() < ModelRole::Junior.default_timeout_ms()
        );
        assert!(ModelRole::Junior.default_timeout_ms() < ModelRole::Senior.default_timeout_ms());
    }

    #[test]
    fn test_envelope_success() {
        let envelope =
            ModelResultEnvelope::success(ModelRole::Junior, "DSK-001", "Boot time is 15s", 0.85);

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
            .with_claim(Claim::with_support(
                "Boot is slow",
                vec!["ev_boot".to_string()],
            ))
            .with_evidence(EvidenceRef::probe("ev_boot", "Boot analysis"));

        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains("junior"));
        assert!(json.contains("Boot is slow"));

        let parsed: ModelResultEnvelope = serde_json::from_str(&json).unwrap();
        assert!(parsed.ok);
        assert_eq!(parsed.claims.len(), 1);
    }
}
