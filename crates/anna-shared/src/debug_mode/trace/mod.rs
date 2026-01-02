//! Canonical Trace Block (v0.0.446).
//!
//! One structured block for all debug output. No scattered "internal comms".
//! Predictable layout, filterable, useful for forensics.

pub mod block;
pub mod llm;
pub mod probe;
pub mod types;

// Re-export all public items from submodules
pub use block::TraceBlock;
pub use llm::{LlmTrace, ParseErrorInfo, PromptDigest};
pub use probe::ProbeTrace;
pub use types::{
    FailureDetail, GateCheck, GateResult, RouteType, TimeoutInfo, TimingTrace, TraceOutcome,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_mode::config::DebugLevel;

    #[test]
    fn test_parse_error_info() {
        let err = ParseErrorInfo::new("Expected '}' at end of object")
            .with_location(1234, "evidence")
            .with_context("{\"answer\": \"test\", \"evidence\":", 1234);

        assert_eq!(err.byte_offset, Some(1234));
        assert_eq!(err.field_name, Some("evidence".to_string()));
    }

    #[test]
    fn test_prompt_digest() {
        let digest = PromptDigest::new("You are a helpful assistant.", "What is my disk usage?");

        assert!(!digest.system_hash.is_empty());
        assert!(!digest.user_hash.is_empty());
        assert!(digest.total_chars > 0);
    }

    #[test]
    fn test_route_type_display() {
        assert_eq!(RouteType::Deterministic.to_string(), "deterministic");
        assert_eq!(RouteType::LlmSpecialist.to_string(), "llm_specialist");
        assert_eq!(RouteType::LlmFallback.to_string(), "llm_fallback");
        assert_eq!(RouteType::Clarification.to_string(), "clarification");
    }

    #[test]
    fn test_trace_outcome_display() {
        assert_eq!(TraceOutcome::Success.to_string(), "SUCCESS");
        assert_eq!(
            TraceOutcome::FailedNoEvidence.to_string(),
            "FAILED_NO_EVIDENCE"
        );
        assert_eq!(TraceOutcome::FailedTimeout.to_string(), "FAILED_TIMEOUT");
    }
}
