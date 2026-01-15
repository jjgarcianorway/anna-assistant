//! Exposure Control - Trust Boundaries and User Mental Model
//!
//! # User Mental Model Contract
//!
//! ## What Anna IS:
//! - A **software tool** that executes commands and processes text
//! - A **local assistant** running on the user's machine
//! - A **deterministic system** that follows programmed rules
//! - A **diagnostic tool** that gathers and reports system information
//!
//! ## What Anna is NOT:
//! - NOT conscious, aware, or sentient
//! - NOT an entity with desires, feelings, or intentions
//! - NOT an authority figure or decision-maker
//! - NOT capable of independent thought or judgment
//!
//! ## What "Internal Dialogue" Represents:
//! - **Processing stages** shown in human-readable format
//! - **Routing decisions** displayed as conversation for clarity
//! - **Debug information** formatted for readability
//! - It is NOT actual communication between conscious entities
//!
//! ## Language Guidelines:
//! - Use passive voice for system actions ("request was processed")
//! - Avoid urgency language ("critical", "urgent", "immediately")
//! - Avoid authority language ("must", "required", "mandatory")
//! - Avoid consciousness attribution ("thinks", "decides", "wants")
//! - Keep tone calm, professional, predictable
//!
//! # Exposure Levels
//!
//! The system enforces strict information boundaries through exposure levels:
//!
//! | Level    | Dialogue | Metadata | Timing | Debug |
//! |----------|----------|----------|--------|-------|
//! | Silent   | No       | No       | No     | No    |
//! | Summary  | No       | Summary  | No     | No    |
//! | Dialogue | Yes      | Summary  | Yes    | No    |
//! | Debug    | Yes      | Full     | Yes    | Yes   |
//!
//! Levels are strictly ordered: Silent < Summary < Dialogue < Debug
//! No implicit escalation. No partial overlap.

pub mod consent;
pub mod gate;
pub mod levels;
pub mod sanitize;

pub use consent::{ConsentState, check_consent, record_consent, CONSENT_ACKNOWLEDGEMENT};
pub use gate::{ExposureGate, GateResult, BlockReason, filter_dialogue, is_dialogue_enabled};
pub use levels::{ExposureLevel, ExposureFilter, should_show, DialogueClassification};
pub use sanitize::{sanitize_dialogue, validate_wording, ForbiddenPattern, SanitizationResult};

/// Mental model assertion - used in tests to verify contract adherence.
/// Anna is software, not an entity.
pub const MENTAL_MODEL_ASSERTION: &str =
    "Anna is a software tool that processes requests. \
     Internal dialogue shows processing stages, not consciousness.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mental_model_no_consciousness_language() {
        // The mental model assertion must not contain forbidden patterns
        let result = sanitize_dialogue(MENTAL_MODEL_ASSERTION);
        assert!(result.is_clean, "Mental model assertion contains forbidden patterns: {:?}", result.violations);
    }

    #[test]
    fn test_exposure_level_ordering() {
        assert!(ExposureLevel::Silent < ExposureLevel::Summary);
        assert!(ExposureLevel::Summary < ExposureLevel::Dialogue);
        assert!(ExposureLevel::Dialogue < ExposureLevel::Debug);
    }
}
