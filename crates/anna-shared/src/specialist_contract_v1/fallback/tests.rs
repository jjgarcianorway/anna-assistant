//! Fallback tests - v0.0.440.
//!
//! Tests for fallback functionality.

#[cfg(test)]
mod tests {
    use crate::specialist_contract_v1::fallback::{
        FallbackReason, FallbackResponse, FallbackSummarizer, ProbeEvidence,
    };

    #[test]
    fn test_fallback_response_new() {
        let response = FallbackResponse::new("DSK-0101", "Boot time is 7.5 seconds.", 0.85);
        assert_eq!(response.case_id, "DSK-0101");
        assert!(!response.is_insufficient());
    }

    #[test]
    fn test_fallback_response_insufficient() {
        let response =
            FallbackResponse::insufficient_evidence("DSK-0101", vec!["systemd_analyze"]);
        assert!(response.is_insufficient());
        assert_eq!(response.confidence, 0.0);
    }

    #[test]
    fn test_summarizer_memory() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::success(
            "free_h",
            "              total        used        free      shared  buff/cache   available\nMem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi",
        )];

        let response = summarizer.summarize("DSK-0101", &evidence, &["free_h"]);
        assert!(!response.is_insufficient());
        assert!(response.answer.contains("31Gi"));
    }

    #[test]
    fn test_summarizer_boot() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::success(
            "systemd_analyze",
            "Startup finished in 2.5s (kernel) + 5.2s (userspace) = 7.7s",
        )];

        let response = summarizer.summarize("DSK-0101", &evidence, &["systemd_analyze"]);
        assert!(!response.is_insufficient());
        assert!(response.answer.contains("7.7s"));
    }

    #[test]
    fn test_summarizer_missing_evidence() {
        let summarizer = FallbackSummarizer::new();
        let evidence = vec![ProbeEvidence::failed("systemd_analyze")];

        let response = summarizer.summarize("DSK-0101", &evidence, &["systemd_analyze"]);
        assert!(response.is_insufficient());
        assert!(response
            .missing_evidence
            .contains(&"systemd_analyze".to_string()));
    }

    #[test]
    fn test_fallback_reason() {
        assert_eq!(FallbackReason::Timeout.label(), "timeout");
        assert_eq!(
            FallbackReason::RetriesExhausted.label(),
            "retries_exhausted"
        );
    }
}
