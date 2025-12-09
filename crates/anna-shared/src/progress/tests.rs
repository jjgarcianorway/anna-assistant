//! Tests for progress module (v0.0.204).

#[cfg(test)]
mod tests {
    use crate::progress::{
        DiagnosticText, ProgressEvent, ProgressEventType, RequestStage, TimeoutConfig,
        MAX_DIAGNOSTIC_LENGTH,
    };

    #[test]
    fn test_progress_event_format() {
        let event = ProgressEvent::starting(RequestStage::Translator, 8, 0);
        assert!(event.format_debug().contains("translator"));
        assert!(event.format_debug().contains("8s"));
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.translator_secs, 8);
        assert_eq!(config.specialist_secs, 12);
    }

    /// GUARDRAIL: ProgressEventType must never carry user-facing content.
    /// Progress is telemetry only. Answers flow through ServiceDeskResult.
    #[test]
    fn test_progress_event_no_answer_content() {
        // Serialize all event variants and verify no "answer" or "content" fields
        let events = vec![
            ProgressEventType::Starting { timeout_secs: 10 },
            ProgressEventType::Complete,
            ProgressEventType::Timeout,
            ProgressEventType::Error {
                message: DiagnosticText::new("test error"),
            },
            ProgressEventType::Heartbeat,
            ProgressEventType::ProbeRunning {
                probe_id: "test".into(),
            },
            ProgressEventType::ProbeComplete {
                probe_id: "test".into(),
                exit_code: 0,
                timing_ms: 100,
            },
            ProgressEventType::Generation { tokens: 42 },
            ProgressEventType::InternalComms {
                from: "desktop_jr".into(),
                message: DiagnosticText::new("Looking at this now"),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            // These fields should NEVER appear in progress events
            assert!(
                !json.contains("\"answer\""),
                "Progress event must not contain 'answer' field"
            );
            assert!(
                !json.contains("\"content\""),
                "Progress event must not contain 'content' field"
            );
            assert!(
                !json.contains("\"response\""),
                "Progress event must not contain 'response' field"
            );
            // String payloads should be short diagnostics only (< 256 bytes)
            assert!(
                json.len() < 256,
                "Progress event JSON should be small (telemetry only)"
            );
        }
    }

    /// GUARDRAIL: ProgressEvent.detail is for short status, not content
    #[test]
    fn test_progress_detail_is_diagnostic_only() {
        let event = ProgressEvent::heartbeat(RequestStage::Specialist, "still thinking", 5000);
        let json = serde_json::to_string(&event).unwrap();
        // Detail should be short diagnostic text (DiagnosticText enforces this)
        assert!(
            event.detail.as_ref().map(|d| d.as_str().len()).unwrap_or(0) <= MAX_DIAGNOSTIC_LENGTH
        );
        assert!(json.len() < 256);
    }

    /// GUARDRAIL: Enforce size cap on worst-case progress event.
    /// This test FAILS if someone tries to stuff large content into progress events.
    const MAX_PROGRESS_EVENT_BYTES: usize = 512;

    #[test]
    fn test_progress_event_size_cap_enforced() {
        // Worst case: error message at max allowed length
        let max_error = ProgressEvent::error(
            RequestStage::Specialist,
            "E".repeat(MAX_DIAGNOSTIC_LENGTH),
            99999,
        );
        let json = serde_json::to_string(&max_error).unwrap();
        assert!(
            json.len() < MAX_PROGRESS_EVENT_BYTES,
            "Max-length error event ({} bytes) exceeds cap ({})",
            json.len(),
            MAX_PROGRESS_EVENT_BYTES
        );

        // Worst case: heartbeat with max detail
        let max_heartbeat = ProgressEvent::heartbeat(
            RequestStage::Specialist,
            "D".repeat(MAX_DIAGNOSTIC_LENGTH),
            99999,
        );
        let json = serde_json::to_string(&max_heartbeat).unwrap();
        assert!(
            json.len() < MAX_PROGRESS_EVENT_BYTES,
            "Max-length heartbeat ({} bytes) exceeds cap ({})",
            json.len(),
            MAX_PROGRESS_EVENT_BYTES
        );
    }

    /// GUARDRAIL: DiagnosticText truncates oversized input - enforced at type level
    #[test]
    fn test_diagnostic_text_truncates_oversized() {
        let oversized = "X".repeat(MAX_DIAGNOSTIC_LENGTH + 50);
        let text = DiagnosticText::new(oversized);

        // Must be truncated to MAX_DIAGNOSTIC_LENGTH
        assert!(
            text.as_str().len() <= MAX_DIAGNOSTIC_LENGTH,
            "DiagnosticText must enforce max length: got {} chars",
            text.as_str().len()
        );

        // Must end with "..." to indicate truncation
        assert!(
            text.as_str().ends_with("..."),
            "Truncated DiagnosticText must end with '...'"
        );
    }

    /// GUARDRAIL: DiagnosticText preserves short input unchanged
    #[test]
    fn test_diagnostic_text_preserves_short() {
        let short = "short message";
        let text = DiagnosticText::new(short);
        assert_eq!(text.as_str(), short);
    }

    /// GUARDRAIL: DiagnosticText exactly at limit is not truncated
    #[test]
    fn test_diagnostic_text_at_limit() {
        let at_limit = "X".repeat(MAX_DIAGNOSTIC_LENGTH);
        let text = DiagnosticText::new(at_limit.clone());
        assert_eq!(text.as_str(), at_limit);
    }
}
